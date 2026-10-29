//! Per-application focus and keyboard dispatch; no component or terminal runtime dependencies.
use crate::element::FocusId;
use crate::element::interaction::{
    Activation, EventCommand, EventContext, EventResult, Focusability, InitialFocus,
    InteractionTree, MenuInteraction, MenuTriggerKind, Orientation, Role, TabDismiss,
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
    typeahead: HashMap<FocusId, crate::element::Typeahead>,
    clock: Option<std::time::Instant>,
    pending_tab_exit: Option<(Vec<FocusId>, Option<FocusId>)>,
    modal_scope: Option<FocusId>,
    requested_initial: HashMap<FocusId, InitialFocus>,
    requested_focus: Option<FocusId>,
    anchors: HashMap<FocusId, (u16, u16)>,
    hovered: Option<FocusId>,
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
        crate::element::focus::with_context(self.focused, &self.tree, &self.open, &self.anchors, f)
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
        self.anchors.retain(|id, _| {
            tree.layers
                .iter()
                .any(|layer| layer.id == *id && layer.active)
        });
        self.hovered = self.hovered.filter(|id| tree.node(*id).is_some());
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
                InitialFocus::Preserve => self.focused,
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
    fn close_branch(&mut self, id: FocusId) {
        let closing: Vec<_> = self
            .active_layers
            .iter()
            .rev()
            .copied()
            .filter(|&child| child == id || self.tree.contains(child, id))
            .collect();
        for child in closing {
            self.dismiss(child);
        }
    }
    fn close_competing(&mut self, item: FocusId) {
        let group = self.tree.group(item).map(|g| g.id);
        let closing: Vec<_> = self
            .active_layers
            .iter()
            .copied()
            .filter(|&id| {
                self.tree
                    .layers
                    .iter()
                    .find(|l| l.id == id)
                    .is_some_and(|l| {
                        l.config.chain
                            && l.opener.is_some_and(|opener| {
                                opener != item
                                    && group.is_some()
                                    && self.tree.group(opener).map(|g| g.id) == group
                            })
                    })
            })
            .collect();
        let bar_switch = self
            .tree
            .group(item)
            .is_some_and(|g| g.interaction.semantics.role == Role::MenuBar);
        for id in closing {
            if bar_switch {
                // The bar's new value closes the previous root atomically.
                // Only descendants need separate close notifications.
                let children: Vec<_> = self
                    .active_layers
                    .iter()
                    .rev()
                    .copied()
                    .filter(|&child| child != id && self.tree.contains(child, id))
                    .collect();
                for child in children {
                    self.dismiss(child);
                }
            } else {
                self.close_branch(id);
            }
        }
    }
    fn open_layer(&mut self, id: FocusId) {
        let opener = self
            .tree
            .layers
            .iter()
            .find(|l| l.id == id)
            .and_then(|l| l.opener);
        if let Some(opener) = opener {
            self.close_competing(opener);
        }
        if let Some(layer) = self.tree.layers.iter().find(|l| l.id == id) {
            if layer.active || self.open.contains(&id) {
                return;
            }
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
                    // Keyboard invocation replaces any previous pointer anchor.
                    self.anchors.remove(&id);
                    self.open_layer(id);
                    if self.active_layers.contains(&id) {
                        let target = match initial {
                            InitialFocus::Last => self.initial_last(id),
                            InitialFocus::First => self
                                .tab_order()
                                .into_iter()
                                .find(|&n| self.tree.contains(n, id)),
                            _ => None,
                        };
                        if let Some(target) = target {
                            self.focus(target);
                        }
                    } else {
                        self.requested_initial.insert(id, initial);
                    }
                }
                EventCommand::OpenAtPoint(id, x, y) => {
                    self.anchors.insert(id, (x, y));
                    self.open_layer(id);
                    if self.active_layers.contains(&id) {
                        let target = self
                            .tab_order()
                            .into_iter()
                            .find(|&n| self.tree.contains(n, id))
                            .unwrap_or(id);
                        self.focus(target);
                    } else {
                        self.requested_initial.insert(id, InitialFocus::First);
                    }
                }
                EventCommand::Dismiss(id) => self.close_branch(id),
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
    fn pointer_allowed(&self, target: &crate::element::HitTarget) -> bool {
        target
            .layer
            .or(target.focus_id)
            .map_or(!self.has_modal(), |id| self.allowed(id))
            && !target
                .focus_id
                .and_then(|id| self.tree.node(id))
                .is_some_and(|n| n.interaction.disabled)
    }

    pub fn pointer_down(
        &mut self,
        target: &crate::element::HitTarget,
        x: u16,
        y: u16,
    ) -> DispatchOutcome {
        if !self.pointer_allowed(target) {
            return DispatchOutcome::default();
        }
        if let Some(id) = target.dismiss {
            let mut context = EventContext::default();
            let root = self
                .tree
                .layers
                .iter()
                .find(|l| l.id == id)
                .filter(|l| l.config.chain)
                .map(|_| {
                    self.active_layers
                        .iter()
                        .copied()
                        .find(|&ancestor| {
                            self.tree
                                .layers
                                .iter()
                                .any(|l| l.id == ancestor && l.config.chain)
                                && self.tree.contains(id, ancestor)
                        })
                        .unwrap_or(id)
                })
                .unwrap_or(id);
            context.dismiss(root);
            let mut outcome = self.apply(context);
            outcome.handled = true;
            return outcome;
        }
        if let Some(id) = target.focus_id
            && let Some(MenuInteraction::Trigger { layer, kind }) =
                self.tree.node(id).and_then(|n| n.interaction.menu)
        {
            let before = self.focused;
            self.focus(id);
            if kind == MenuTriggerKind::Context {
                return DispatchOutcome {
                    handled: true,
                    changed: before != self.focused,
                    exit: false,
                };
            }
            let mut context = self.event_context();
            if self.active_layers.contains(&layer) {
                context.dismiss(layer);
            } else {
                context.open_at(layer, InitialFocus::First);
            }
            let mut outcome = self.apply(context);
            outcome.changed |= before != self.focused;
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

    fn switch_bar(&mut self, layer_id: FocusId, key: KeyEvent) -> bool {
        if key
            .modifiers
            .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT)
        {
            return false;
        }
        let Some(layer) = self
            .tree
            .layers
            .iter()
            .find(|l| l.id == layer_id && l.active)
        else {
            return false;
        };
        let Some(opener) = layer.opener else {
            return false;
        };
        let Some(group) = self
            .tree
            .group(opener)
            .filter(|g| g.interaction.semantics.role == Role::MenuBar)
            .map(|g| g.id)
        else {
            return false;
        };
        self.focus(opener);
        self.navigate(group, key);
        let Some(next) = self.focused else {
            return false;
        };
        let Some(MenuInteraction::Trigger { layer, .. }) =
            self.tree.node(next).and_then(|n| n.interaction.menu)
        else {
            return false;
        };
        if layer != layer_id {
            self.open_layer(layer);
            self.requested_initial.insert(layer, InitialFocus::First);
        }
        true
    }

    /// The top painted surface wins. Dismiss backdrops are transparent only to
    /// menu pointer routing, allowing travel back into an ancestor menu.
    fn menu_pointer_target(
        &self,
        hits: &crate::element::HitMap,
        x: u16,
        y: u16,
    ) -> Option<FocusId> {
        let target = hits
            .targets
            .iter()
            .rev()
            .find(|t| t.rect.contains(x, y) && t.dismiss.is_none())?;
        if !target
            .layer
            .or(target.focus_id)
            .is_none_or(|id| self.allowed(id))
        {
            return None;
        }
        let mut current = target.focus_id;
        while let Some(id) = current {
            let node = self.tree.node(id)?;
            if node.interaction.menu.is_some() {
                return Some(id);
            }
            if node.interaction.scope.is_some() {
                return None;
            }
            current = node.parent;
        }
        // A content surface or unrelated control blocks underlying menus.
        None
    }

    pub fn pointer_down_at(
        &mut self,
        hits: &crate::element::HitMap,
        x: u16,
        y: u16,
    ) -> DispatchOutcome {
        if let Some(id) = self.menu_pointer_target(hits, x, y)
            && let Some(target) = hits
                .targets
                .iter()
                .rev()
                .find(|t| t.focus_id == Some(id) && t.rect.contains(x, y))
        {
            if self
                .tree
                .node(id)
                .is_some_and(|node| !node.interaction.disabled)
            {
                self.close_competing(id);
            }
            return self.pointer_down(target, x, y);
        }
        hits.hit_test(x, y)
            .map(|target| self.pointer_down(target, x, y))
            .unwrap_or_default()
    }

    pub fn pointer_move(
        &mut self,
        hits: &crate::element::HitMap,
        x: u16,
        y: u16,
    ) -> DispatchOutcome {
        let next = self.menu_pointer_target(hits, x, y);
        if self.hovered == next {
            return DispatchOutcome::default();
        }
        self.hovered = next;
        let Some(id) = next else {
            return DispatchOutcome::default();
        };
        let Some(node) = self.tree.node(id) else {
            return DispatchOutcome::default();
        };
        if node.interaction.disabled {
            return DispatchOutcome::default();
        }
        let policy = node.interaction.menu;
        let before = self.focused;
        match policy {
            Some(MenuInteraction::Item) => {
                self.close_competing(id);
                self.focus(id);
            }
            Some(MenuInteraction::Trigger {
                layer,
                kind: MenuTriggerKind::Submenu,
            }) => {
                self.close_competing(id);
                self.focus(id);
                if !self.active_layers.contains(&layer) {
                    self.open_layer(layer);
                    self.requested_initial.insert(layer, InitialFocus::Preserve);
                    return DispatchOutcome {
                        handled: true,
                        changed: true,
                        exit: false,
                    };
                }
            }
            Some(MenuInteraction::Trigger {
                layer,
                kind: MenuTriggerKind::MenuBar,
            }) => {
                let bar = self.tree.group(id).map(|g| g.id);
                let any_open = self.active_layers.iter().any(|&active| {
                    self.tree
                        .layers
                        .iter()
                        .find(|l| l.id == active)
                        .and_then(|l| l.opener)
                        .and_then(|opener| self.tree.group(opener))
                        .map(|g| g.id)
                        == bar
                });
                if any_open && !self.active_layers.contains(&layer) {
                    self.focus(id);
                    self.open_layer(layer);
                    self.requested_initial.insert(layer, InitialFocus::Preserve);
                    return DispatchOutcome {
                        handled: true,
                        changed: true,
                        exit: false,
                    };
                }
            }
            _ => {}
        }
        DispatchOutcome {
            handled: true,
            changed: before != self.focused || next.is_some(),
            exit: false,
        }
    }

    pub fn context_menu(
        &mut self,
        hits: &crate::element::HitMap,
        x: u16,
        y: u16,
    ) -> DispatchOutcome {
        let Some(id) = self.menu_pointer_target(hits, x, y) else {
            return DispatchOutcome::default();
        };
        let Some(node) = self.tree.node(id) else {
            return DispatchOutcome::default();
        };
        if node.interaction.disabled {
            return DispatchOutcome::default();
        }
        let Some(MenuInteraction::Trigger {
            layer,
            kind: MenuTriggerKind::Context,
        }) = node.interaction.menu
        else {
            return DispatchOutcome::default();
        };
        self.focus(id);
        let mut context = self.event_context();
        context.open_at_point(layer, x, y);
        self.apply(context)
    }

    pub fn scroll(
        &mut self,
        target: &crate::element::HitTarget,
        dx: i16,
        dy: i16,
    ) -> DispatchOutcome {
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
        self.hovered = None;
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
                if node.interaction.menu.is_none()
                    && matches!(key.code, KeyCode::Down | KeyCode::Up)
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
                if node.interaction.semantics.role == Role::MenuBar {
                    let open = self.tree.layers.iter().any(|layer| {
                        layer.active
                            && layer
                                .opener
                                .and_then(|opener| self.tree.group(opener))
                                .is_some_and(|group| group.id == id)
                    });
                    if open
                        && let Some(MenuInteraction::Trigger { layer, .. }) = self
                            .focused
                            .and_then(|focused| self.tree.node(focused))
                            .and_then(|n| n.interaction.menu)
                        && !self.active_layers.contains(&layer)
                    {
                        self.open_layer(layer);
                        self.requested_initial.insert(layer, InitialFocus::First);
                    }
                }
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
            if matches!(key.code, KeyCode::Left | KeyCode::Right) && self.switch_bar(id, key) {
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
