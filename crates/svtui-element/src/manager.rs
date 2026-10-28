//! Per-application focus and keyboard dispatch; no component or terminal runtime dependencies.
use crate::FocusId;
use crate::interaction::{
    Activation, EventCommand, EventContext, EventResult, Focusability, InitialFocus,
    InteractionTree, Orientation, TabDismiss,
};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::collections::{HashMap, HashSet};

#[derive(Default, Debug)]
pub struct DispatchOutcome {
    pub handled: bool,
    pub changed: bool,
    pub exit: bool,
}

/// Authoritative state for one application. No terminal or component dependency.
#[derive(Default)]
pub struct FocusManager {
    tree: InteractionTree,
    focused: Option<FocusId>,
    open: HashSet<FocusId>,
    active_layers: Vec<FocusId>,
    restored: HashMap<FocusId, Option<FocusId>>,
    group_active: HashMap<FocusId, FocusId>,
    typeahead: HashMap<FocusId, crate::Typeahead>,
    clock: Option<std::time::Instant>,
    pending_tab_exit: Option<(Vec<FocusId>, Option<FocusId>)>,
    modal_scope: Option<FocusId>,
    requested_initial: HashMap<FocusId, InitialFocus>,
    requested_focus: Option<FocusId>,
}

impl FocusManager {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn focused(&self) -> Option<FocusId> {
        self.focused
    }
    pub fn tree(&self) -> &InteractionTree {
        &self.tree
    }
    pub fn event_context(&self) -> EventContext {
        EventContext {
            focused: self.focused,
            ..EventContext::default()
        }
    }
    pub fn is_open(&self, id: FocusId) -> bool {
        self.open.contains(&id)
    }
    pub fn with_context<T>(&self, f: impl FnOnce() -> T) -> T {
        crate::focus::with_context(self.focused, &self.tree, &self.open, f)
    }
    pub fn focus(&mut self, id: FocusId) -> bool {
        if self.eligible(id) && self.allowed(id) {
            self.focused = Some(id);
            self.remember_group();
            true
        } else {
            false
        }
    }
    fn eligible(&self, id: FocusId) -> bool {
        self.tree.node(id).is_some_and(|n| {
            n.interaction.focusability != Focusability::None
                && (!n.interaction.disabled
                    || self
                        .tree
                        .group(id)
                        .is_some_and(|g| g.interaction.group.unwrap().disabled_focusable))
        })
    }
    pub fn allowed(&self, id: FocusId) -> bool {
        self.modal()
            .is_none_or(|scope| self.tree.contains(id, scope))
    }
    pub fn has_modal(&self) -> bool {
        self.modal().is_some()
    }
    fn modal(&self) -> Option<FocusId> {
        self.modal_scope
    }
    fn remember_group(&mut self) {
        if let Some(id) = self.focused
            && let Some(group) = self.tree.group(id)
        {
            self.group_active.insert(group.id, id);
        }
    }
    pub fn tab_order(&self) -> Vec<FocusId> {
        self.tab_order_excluding(&[])
    }
    fn tab_order_excluding(&self, excluded: &[FocusId]) -> Vec<FocusId> {
        let candidates: Vec<_> = self
            .tree
            .nodes
            .iter()
            .filter(|n| {
                self.eligible(n.id)
                    && self.allowed(n.id)
                    && !excluded.iter().any(|&id| self.tree.contains(n.id, id))
            })
            .collect();
        let mut group_heads = HashMap::new();
        for node in &candidates {
            if let Some(group) = self.tree.group(node.id) {
                let active = self.group_active.get(&group.id).copied().filter(|&id| {
                    self.eligible(id)
                        && self.allowed(id)
                        && self.tree.group(id).is_some_and(|g| g.id == group.id)
                        && !excluded
                            .iter()
                            .any(|&ancestor| self.tree.contains(id, ancestor))
                });
                group_heads
                    .entry(group.id)
                    .or_insert(active.unwrap_or(node.id));
            }
        }
        candidates
            .into_iter()
            .filter(|node| {
                self.tree.group(node.id).map_or(
                    node.interaction.focusability == Focusability::TabStop,
                    |group| group_heads.get(&group.id) == Some(&node.id),
                )
            })
            .map(|node| node.id)
            .collect()
    }
    fn initial_last(&self, scope: FocusId) -> Option<FocusId> {
        self.tree
            .nodes
            .iter()
            .rev()
            .find(|node| {
                self.eligible(node.id)
                    && self.allowed(node.id)
                    && self.tree.contains(node.id, scope)
                    && (node.interaction.focusability == Focusability::TabStop
                        || self.tree.group(node.id).is_some())
            })
            .map(|node| node.id)
    }
    pub fn reconcile(&mut self, tree: InteractionTree) -> bool {
        let before = self.focused;
        if let Some((closing, previous)) = self.pending_tab_exit.take()
            && let Some(rejected) = closing
                .iter()
                .find(|&&id| tree.layers.iter().any(|l| l.id == id && l.active))
        {
            self.focused = previous.filter(|&id| tree.node(id).is_some()).or_else(|| {
                tree.nodes
                    .iter()
                    .find(|n| {
                        tree.contains(n.id, *rejected)
                            && n.interaction.focusability == Focusability::TabStop
                            && !n.interaction.disabled
                    })
                    .map(|n| n.id)
            });
        }
        let old_order: Vec<_> = self.tree.nodes.iter().map(|n| n.id).collect();
        let new_scopes: Vec<_> = tree
            .nodes
            .iter()
            .filter(|n| {
                n.interaction.scope.is_some()
                    && !tree.layers.iter().any(|l| l.id == n.id)
                    && self.tree.node(n.id).is_none()
            })
            .map(|n| n.id)
            .collect();
        let old_index = before.and_then(|id| old_order.iter().position(|&n| n == id));
        let next_layers: Vec<_> = tree
            .layers
            .iter()
            .filter(|l| l.active)
            .map(|l| l.id)
            .collect();
        for &id in self.active_layers.iter().rev() {
            if !next_layers.contains(&id) {
                let mut opener = self.restored.remove(&id).flatten();
                if opener.is_none_or(|id| tree.node(id).is_none_or(|n| n.interaction.disabled)) {
                    let mut parent = self
                        .tree
                        .layers
                        .iter()
                        .find(|l| l.id == id)
                        .and_then(|l| l.parent);
                    while let Some(ancestor) = parent {
                        if tree
                            .node(ancestor)
                            .is_some_and(|n| n.interaction.scope.is_some())
                        {
                            opener = Some(ancestor);
                            break;
                        }
                        parent = self.tree.node(ancestor).and_then(|n| n.parent);
                    }
                }
                if self
                    .focused
                    .is_none_or(|focused| self.tree.contains(focused, id))
                {
                    self.focused = opener;
                }
            }
        }
        let newly_opened: Vec<_> = next_layers
            .iter()
            .filter(|id| !self.active_layers.contains(id))
            .copied()
            .collect();
        self.active_layers.retain(|id| next_layers.contains(id));
        self.active_layers.extend(&newly_opened);
        self.open
            .retain(|id| tree.layers.iter().any(|l| l.id == *id));
        for layer in &tree.layers {
            if layer.active {
                self.open.insert(layer.id);
            } else {
                self.open.remove(&layer.id);
            }
        }
        self.tree = tree;
        self.modal_scope = self
            .active_layers
            .iter()
            .rev()
            .copied()
            .find(|&id| {
                self.tree
                    .node(id)
                    .is_some_and(|n| n.interaction.scope.is_some_and(|s| s.contain_tab))
            })
            .or_else(|| {
                self.tree
                    .nodes
                    .iter()
                    .rev()
                    .find(|n| n.interaction.scope.is_some_and(|s| s.contain_tab))
                    .map(|n| n.id)
            });
        self.group_active.retain(|group, member| {
            self.tree
                .group(*member)
                .is_some_and(|owner| owner.id == *group)
        });
        self.typeahead.retain(|group, _| {
            self.tree
                .node(*group)
                .is_some_and(|n| n.interaction.group.is_some())
        });
        for id in new_scopes {
            if !self.allowed(id) {
                continue;
            }
            let scope = self.tree.node(id).unwrap().interaction.scope.unwrap();
            let first = self
                .tab_order()
                .into_iter()
                .find(|&child| self.tree.contains(child, id));
            self.focused = match scope.initial_focus {
                InitialFocus::Target(target) => Some(target)
                    .filter(|&t| self.eligible(t) && self.tree.contains(t, id))
                    .or(first),
                InitialFocus::Container => Some(id),
                InitialFocus::First => first.or_else(|| scope.contain_tab.then_some(id)),
                InitialFocus::Last => self
                    .initial_last(id)
                    .or_else(|| scope.contain_tab.then_some(id)),
                InitialFocus::Preserve => self.focused,
            }
            .or(self.focused);
        }
        for id in newly_opened {
            let layer = self.tree.layers.iter().find(|l| l.id == id).unwrap();
            self.restored.insert(id, layer.opener.or(self.focused));
            let initial = self
                .requested_initial
                .remove(&id)
                .unwrap_or(layer.config.scope.initial_focus);
            let target = match initial {
                InitialFocus::Target(target) => Some(target)
                    .filter(|&n| self.eligible(n) && self.tree.contains(n, id))
                    .or_else(|| {
                        self.tab_order()
                            .into_iter()
                            .find(|&n| self.tree.contains(n, id))
                    }),
                InitialFocus::Last => self.initial_last(id),
                InitialFocus::Container => Some(id),
                InitialFocus::Preserve => self.focused.filter(|&f| self.tree.contains(f, id)),
                InitialFocus::First => self
                    .tab_order()
                    .into_iter()
                    .find(|&n| self.tree.contains(n, id)),
            };
            self.focused = target
                .or_else(|| {
                    (layer.config.scope.contain_tab || layer.config.tab == TabDismiss::Any)
                        .then_some(id)
                })
                .or(self.focused);
        }
        if let Some(id) = self.requested_focus.take() {
            self.focus(id);
        }
        self.requested_initial
            .retain(|id, _| self.tree.layers.iter().any(|l| l.id == *id && l.active));
        if self
            .focused
            .is_some_and(|id| !self.eligible(id) || !self.allowed(id))
        {
            let candidates = self.tab_order();
            self.focused = old_index
                .and_then(|index| {
                    old_order[index..]
                        .iter()
                        .chain(old_order[..index].iter().rev())
                        .copied()
                        .find(|id| candidates.contains(id))
                })
                .or_else(|| candidates.first().copied())
                .or_else(|| self.modal());
        }
        if self.focused.is_none() {
            self.focused = self.modal();
        }
        self.remember_group();
        before != self.focused
    }
    fn owning_layers(&self) -> Vec<FocusId> {
        let Some(focused) = self.focused else {
            return Vec::new();
        };
        self.active_layers
            .iter()
            .rev()
            .copied()
            .filter(|&id| self.tree.contains(focused, id))
            .collect()
    }
    fn open_layer(&mut self, id: FocusId) {
        if let Some(layer) = self.tree.layers.iter().find(|l| l.id == id) {
            let callback = layer.on_open.clone();
            self.open.insert(id);
            if let Some(callback) = callback {
                self.with_context(|| callback());
            }
        }
    }
    fn dismiss(&mut self, id: FocusId) {
        if let Some(layer) = self.tree.layers.iter().find(|l| l.id == id) {
            let managed = layer.managed;
            let close = layer.on_dismiss.clone();
            if managed {
                self.open.remove(&id);
            }
            if let Some(close) = close {
                self.with_context(|| close());
            }
        }
    }
    fn tab(&mut self, back: bool) {
        let layers = self.owning_layers();
        let nearest = layers
            .first()
            .and_then(|id| self.tree.layers.iter().find(|l| l.id == *id));
        let exit = nearest.is_some_and(|l| {
            l.config.tab == TabDismiss::Any
                || (l.config.tab == TabDismiss::Boundary && {
                    let ids: Vec<_> = self
                        .tab_order()
                        .into_iter()
                        .filter(|&id| self.tree.contains(id, l.id))
                        .collect();
                    self.focused
                        == if back {
                            ids.first().copied()
                        } else {
                            ids.last().copied()
                        }
                })
        });
        let (order, start) = if exit {
            let mut closing = vec![nearest.unwrap().id];
            if nearest.unwrap().config.tab == TabDismiss::Any {
                closing = layers
                    .into_iter()
                    .take_while(|id| {
                        self.tree
                            .layers
                            .iter()
                            .find(|l| l.id == *id)
                            .is_some_and(|l| l.config.tab == TabDismiss::Any)
                    })
                    .collect();
            }
            let root = *closing.last().unwrap();
            let opener = self
                .tree
                .layers
                .iter()
                .find(|l| l.id == root)
                .and_then(|l| l.opener)
                .or(self.restored.get(&root).copied().flatten());
            let order = self.tab_order_excluding(&closing);
            self.pending_tab_exit = Some((closing.clone(), self.focused));
            for id in closing {
                self.dismiss(id);
            }
            (order, opener)
        } else {
            (self.tab_order(), self.focused)
        };
        if order.is_empty() {
            self.focused = self.modal();
            return;
        }
        let index = start.and_then(|id| order.iter().position(|&n| n == id));
        let next = match index {
            Some(i) if back => (i + order.len() - 1) % order.len(),
            Some(i) => (i + 1) % order.len(),
            None if back => order.len() - 1,
            None => 0,
        };
        self.focused = Some(order[next]);
        self.remember_group();
    }
    fn navigate(&mut self, group_id: FocusId, key: KeyEvent) -> bool {
        if key
            .modifiers
            .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT)
        {
            return false;
        }
        let group = self.tree.node(group_id).unwrap().interaction.group.unwrap();
        let members: Vec<_> = self
            .tree
            .nodes
            .iter()
            .filter(|n| {
                self.eligible(n.id) && self.tree.group(n.id).is_some_and(|g| g.id == group_id)
            })
            .collect();
        if members.is_empty() {
            return false;
        }
        let current = members
            .iter()
            .position(|n| Some(n.id) == self.focused)
            .unwrap_or(0);
        let next = match key.code {
            KeyCode::Home => Some(0),
            KeyCode::End => Some(members.len() - 1),
            KeyCode::Up if group.orientation == Orientation::Vertical => Some(if current > 0 {
                current - 1
            } else if group.wrap {
                members.len() - 1
            } else {
                0
            }),
            KeyCode::Left if group.orientation == Orientation::Horizontal => Some(if current > 0 {
                current - 1
            } else if group.wrap {
                members.len() - 1
            } else {
                0
            }),
            KeyCode::Down if group.orientation == Orientation::Vertical => {
                Some(if current + 1 < members.len() {
                    current + 1
                } else if group.wrap {
                    0
                } else {
                    current
                })
            }
            KeyCode::Right if group.orientation == Orientation::Horizontal => {
                Some(if current + 1 < members.len() {
                    current + 1
                } else if group.wrap {
                    0
                } else {
                    current
                })
            }
            KeyCode::Char(c)
                if group.typeahead
                    && !key
                        .modifiers
                        .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
            {
                let time = self
                    .clock
                    .get_or_insert_with(std::time::Instant::now)
                    .elapsed();
                let prefix = self
                    .typeahead
                    .entry(group_id)
                    .or_default()
                    .push(c, time)
                    .to_lowercase();
                (1..=members.len())
                    .map(|offset| (current + offset) % members.len())
                    .find(|&i| {
                        members[i]
                            .interaction
                            .semantics
                            .label
                            .to_lowercase()
                            .starts_with(&prefix)
                    })
            }
            _ => return false,
        };
        if let Some(index) = next {
            self.focused = Some(members[index].id);
            self.remember_group();
        }
        true
    }
    pub fn apply(&mut self, context: EventContext) -> DispatchOutcome {
        let mut result = DispatchOutcome::default();
        for command in context.commands {
            result.changed = true;
            match command {
                EventCommand::Focus(id) => {
                    if !self.focus(id) {
                        self.requested_focus = Some(id);
                    }
                }
                EventCommand::Open(id) => self.open_layer(id),
                EventCommand::OpenAt(id, initial) => {
                    self.open_layer(id);
                    self.requested_initial.insert(id, initial);
                }
                EventCommand::Dismiss(id) => self.dismiss(id),
                EventCommand::DismissChain => {
                    let owners = self.owning_layers();
                    let closing: Vec<_> = if owners.first().is_some_and(|id| {
                        self.tree
                            .layers
                            .iter()
                            .any(|l| l.id == *id && l.config.chain)
                    }) {
                        owners
                            .into_iter()
                            .take_while(|id| {
                                self.tree
                                    .layers
                                    .iter()
                                    .any(|l| l.id == *id && l.config.chain)
                            })
                            .collect()
                    } else {
                        owners.into_iter().take(1).collect()
                    };
                    for id in closing {
                        self.dismiss(id);
                    }
                }
                EventCommand::Exit => result.exit = true,
            }
        }
        result
    }
    fn pointer_allowed(&self, target: &crate::HitTarget) -> bool {
        target
            .layer
            .or(target.focus_id)
            .map_or(!self.has_modal(), |id| self.allowed(id))
            && !target
                .focus_id
                .and_then(|id| self.tree.node(id))
                .is_some_and(|n| n.interaction.disabled)
    }

    pub fn pointer_down(&mut self, target: &crate::HitTarget, x: u16, y: u16) -> DispatchOutcome {
        if !self.pointer_allowed(target) {
            return DispatchOutcome::default();
        }
        if let Some(id) = target.dismiss {
            let mut context = EventContext::default();
            context.dismiss(id);
            let mut outcome = self.apply(context);
            outcome.handled = true;
            return outcome;
        }
        let mut outcome = DispatchOutcome {
            handled: true,
            ..DispatchOutcome::default()
        };
        let before = self.focused;
        let mut activated = false;
        if let Some(id) = target.focus_id {
            self.focus(id);
            if let Some(node) = self.tree.node(id) {
                let code = match node.interaction.activation {
                    Activation::Press => Some(KeyCode::Enter),
                    Activation::Toggle => Some(KeyCode::Char(' ')),
                    Activation::None => None,
                };
                if let Some(code) = code {
                    activated = true;
                    outcome = self.dispatch(KeyEvent::new(code, KeyModifiers::NONE));
                }
            }
        }
        if !activated && let Some(action) = &target.on_click {
            action();
            outcome.changed = true;
        }
        if let Some(action) = &target.on_click_at {
            action(
                x.saturating_sub(target.rect.x),
                y.saturating_sub(target.rect.y),
            );
            outcome.changed = true;
        }
        outcome.changed |= before != self.focused;
        outcome
    }

    pub fn scroll(&mut self, target: &crate::HitTarget, dx: i16, dy: i16) -> DispatchOutcome {
        if self.pointer_allowed(target)
            && let Some(action) = &target.on_scroll
        {
            action(dx, dy);
            return DispatchOutcome {
                handled: true,
                changed: true,
                exit: false,
            };
        }
        DispatchOutcome::default()
    }

    pub fn dispatch(&mut self, key: KeyEvent) -> DispatchOutcome {
        if key.kind == KeyEventKind::Release {
            return DispatchOutcome::default();
        }
        let before = self.focused;
        let mut context = self.event_context();
        let mut handled = false;
        let mut current = self.focused;
        while let Some(id) = current {
            context.current_target = Some(id);
            let Some(node) = self.tree.node(id).cloned() else {
                break;
            };
            if let Some(handler) = &node.on_key
                && !node.interaction.disabled
                && self.with_context(|| handler(key, &mut context)) == EventResult::Handled
            {
                handled = true;
                break;
            }
            if Some(id) == self.focused
                && node.interaction.disabled
                && matches!(key.code, KeyCode::Enter | KeyCode::Char(' '))
            {
                handled = true;
                break;
            }
            if Some(id) == self.focused && !node.interaction.disabled {
                let activate = match node.interaction.activation {
                    Activation::Press => matches!(key.code, KeyCode::Enter | KeyCode::Char(' ')),
                    Activation::Toggle => key.code == KeyCode::Char(' '),
                    Activation::None => false,
                };
                if activate {
                    if let Some(layer) = self.tree.layers.iter().find(|l| {
                        l.opener == Some(id)
                            && (l.managed || (l.on_open.is_some() && node.on_activate.is_none()))
                    }) {
                        if layer.active {
                            context.dismiss(layer.id);
                        } else {
                            context.open(layer.id);
                        }
                    }
                    if let Some(action) = &node.on_activate {
                        self.with_context(|| action());
                    }
                    handled = true;
                    break;
                }
                if matches!(key.code, KeyCode::Down | KeyCode::Up)
                    && let Some(layer) = self.tree.layers.iter().find(|l| {
                        l.opener == Some(id)
                            && l.config.tab == TabDismiss::Any
                            && !l.config.dismiss_on_left
                    })
                {
                    if layer.active {
                        let order: Vec<_> = self
                            .tree
                            .nodes
                            .iter()
                            .filter(|n| {
                                n.interaction.focusability == Focusability::TabStop
                                    && self.eligible(n.id)
                                    && self.tree.contains(n.id, layer.id)
                            })
                            .map(|n| n.id)
                            .collect();
                        if let Some(target) = if key.code == KeyCode::Up {
                            order.last()
                        } else {
                            order.first()
                        } {
                            context.focus(*target);
                        }
                    } else {
                        let initial = if key.code == KeyCode::Up {
                            InitialFocus::Last
                        } else {
                            InitialFocus::First
                        };
                        context.open_at(layer.id, initial);
                        if !layer.managed
                            && layer.on_open.is_none()
                            && let Some(action) = &node.on_activate
                        {
                            self.with_context(|| action());
                        }
                    }
                    handled = true;
                    break;
                }
                if key.code == KeyCode::Right
                    && let Some(layer) = self
                        .tree
                        .layers
                        .iter()
                        .find(|l| l.opener == Some(id) && l.config.dismiss_on_left)
                {
                    context.open(layer.id);
                    handled = true;
                    break;
                }
            }
            if node.interaction.group.is_some() && self.navigate(id, key) {
                handled = true;
                break;
            }
            if node.interaction.scope.is_some()
                && matches!(key.code, KeyCode::Tab | KeyCode::BackTab)
            {
                self.tab(
                    key.code == KeyCode::BackTab || key.modifiers.contains(KeyModifiers::SHIFT),
                );
                handled = true;
                break;
            }
            if let Some(layer) = self.tree.layers.iter().find(|l| l.active && l.id == id)
                && (key.code == KeyCode::Esc
                    || (key.code == KeyCode::Left && layer.config.dismiss_on_left))
            {
                context.dismiss(id);
                handled = true;
                break;
            }
            if let Some(scroll) = &node.on_scroll {
                let delta = match key.code {
                    KeyCode::Up => Some((0, -1)),
                    KeyCode::Down => Some((0, 1)),
                    KeyCode::Left => Some((-2, 0)),
                    KeyCode::Right => Some((2, 0)),
                    KeyCode::PageUp => Some((0, -5)),
                    KeyCode::PageDown => Some((0, 5)),
                    _ => None,
                };
                if let Some((x, y)) = delta {
                    scroll(x, y);
                    handled = true;
                    break;
                }
            }
            if node.interaction.scope.is_some_and(|s| s.contain_tab) {
                handled = true;
                break;
            }
            current = node.parent;
        }
        if !handled
            && matches!(key.code, KeyCode::Tab | KeyCode::BackTab)
            && !self.tab_order().is_empty()
        {
            self.tab(key.code == KeyCode::BackTab || key.modifiers.contains(KeyModifiers::SHIFT));
            handled = true;
        }
        if !handled
            && key.code == KeyCode::Esc
            && let Some(layer) = self
                .active_layers
                .last()
                .and_then(|id| self.tree.layers.iter().find(|l| l.id == *id))
            && (self.focused.is_none() || layer.opener == self.focused)
        {
            context.dismiss(layer.id);
            handled = true;
        }
        let mut outcome = self.apply(context);
        outcome.changed |= before != self.focused || handled;
        outcome.handled = handled;
        outcome
    }
}
