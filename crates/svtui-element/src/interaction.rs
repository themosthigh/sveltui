//! Opt-in keyboard participation and logical interaction ownership.
//!
//! This model is independent of paint clipping and terminal event polling.
pub use crate::manager::{DispatchOutcome, FocusManager};
use crate::{FocusId, VNode};
use crossterm::event::KeyEvent;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Focusability {
    #[default]
    None,
    Programmatic,
    TabStop,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EventResult {
    Handled,
    #[default]
    Ignored,
}

pub type KeyHandler = Rc<dyn Fn(KeyEvent, &mut EventContext) -> EventResult>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventCommand {
    Focus(FocusId),
    Open(FocusId),
    OpenAt(FocusId, InitialFocus),
    Dismiss(FocusId),
    DismissChain,
    Exit,
}

#[derive(Default)]
pub struct EventContext {
    pub(crate) focused: Option<FocusId>,
    pub(crate) current_target: Option<FocusId>,
    pub(crate) commands: Vec<EventCommand>,
}

impl EventContext {
    pub fn focused(&self) -> Option<FocusId> {
        self.focused
    }
    pub fn current_target(&self) -> Option<FocusId> {
        self.current_target
    }
    pub fn focus(&mut self, id: FocusId) {
        self.commands.push(EventCommand::Focus(id));
    }
    pub fn open(&mut self, id: FocusId) {
        self.commands.push(EventCommand::Open(id));
    }
    pub fn open_at(&mut self, id: FocusId, initial: InitialFocus) {
        self.commands.push(EventCommand::OpenAt(id, initial));
    }
    pub fn dismiss(&mut self, id: FocusId) {
        self.commands.push(EventCommand::Dismiss(id));
    }
    pub fn dismiss_chain(&mut self) {
        self.commands.push(EventCommand::DismissChain);
    }
    pub fn exit(&mut self) {
        self.commands.push(EventCommand::Exit);
    }
    pub fn commands(&self) -> &[EventCommand] {
        &self.commands
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Role {
    #[default]
    Generic,
    Button,
    Checkbox,
    Switch,
    Textbox,
    Dialog,
    Menu,
    MenuItem,
    List,
    Listbox,
    Option,
    Table,
    Grid,
    Row,
    Cell,
    Tree,
    TreeItem,
    Treegrid,
    TabList,
    Tab,
    Toolbar,
    RadioGroup,
    Radio,
    Separator,
    ProgressBar,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Semantics {
    pub role: Role,
    pub label: String,
    pub description: Option<String>,
    pub checked: Option<bool>,
    pub selected: Option<bool>,
    pub expanded: Option<bool>,
    pub controls: Option<FocusId>,
    pub has_popup: Option<Role>,
    pub modal: Option<bool>,
    pub parent_item: Option<FocusId>,
    pub row: Option<usize>,
    pub column: Option<usize>,
    pub value: Option<f64>,
}

impl Semantics {
    pub fn new(role: Role, label: impl Into<String>) -> Self {
        Self {
            role,
            label: label.into(),
            ..Self::default()
        }
    }
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = Some(checked);
        self
    }
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = Some(expanded);
        self
    }
    pub fn controls(mut self, id: FocusId) -> Self {
        self.controls = Some(id);
        self
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InitialFocus {
    #[default]
    First,
    Last,
    Target(FocusId),
    Container,
    Preserve,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FocusScope {
    pub contain_tab: bool,
    pub initial_focus: InitialFocus,
}

impl FocusScope {
    pub fn modal() -> Self {
        Self {
            contain_tab: true,
            initial_focus: InitialFocus::First,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Orientation {
    Horizontal,
    #[default]
    Vertical,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FocusGroup {
    pub orientation: Orientation,
    pub wrap: bool,
    pub typeahead: bool,
    pub disabled_focusable: bool,
}

impl FocusGroup {
    pub fn menu() -> Self {
        Self {
            orientation: Orientation::Vertical,
            wrap: true,
            typeahead: true,
            disabled_focusable: true,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TabDismiss {
    #[default]
    None,
    Boundary,
    Any,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DismissLayer {
    pub scope: FocusScope,
    pub tab: TabDismiss,
    pub dismiss_on_left: bool,
    pub chain: bool,
}

impl Default for DismissLayer {
    fn default() -> Self {
        Self::popover()
    }
}

impl DismissLayer {
    pub fn dialog() -> Self {
        Self {
            scope: FocusScope::modal(),
            tab: TabDismiss::None,
            dismiss_on_left: false,
            chain: false,
        }
    }
    pub fn popover() -> Self {
        Self {
            scope: FocusScope::modal(),
            tab: TabDismiss::None,
            dismiss_on_left: false,
            chain: false,
        }
    }
    pub fn menu() -> Self {
        Self {
            scope: FocusScope::default(),
            tab: TabDismiss::Any,
            dismiss_on_left: false,
            chain: true,
        }
    }
    pub fn submenu() -> Self {
        Self {
            dismiss_on_left: true,
            ..Self::menu()
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Activation {
    #[default]
    None,
    Press,
    Toggle,
}

/// Stored only when explicitly attached to an element.
#[derive(Clone, Debug, Default)]
pub struct Interaction {
    pub focusability: Focusability,
    pub disabled: bool,
    pub hidden: bool,
    pub activation: Activation,
    pub scope: Option<FocusScope>,
    pub group: Option<FocusGroup>,
    pub semantics: Semantics,
}

#[derive(Clone)]
pub struct InteractionNode {
    pub id: FocusId,
    pub parent: Option<FocusId>,
    pub ancestors: Vec<FocusId>,
    pub interaction: Interaction,
    pub on_key: Option<KeyHandler>,
    pub on_activate: Option<Rc<dyn Fn()>>,
    pub on_scroll: Option<Rc<dyn Fn(i16, i16)>>,
}

#[derive(Clone)]
pub struct LayerNode {
    pub id: FocusId,
    pub parent: Option<FocusId>,
    pub opener: Option<FocusId>,
    pub config: DismissLayer,
    pub active: bool,
    pub managed: bool,
    pub on_dismiss: Option<Rc<dyn Fn()>>,
    pub on_open: Option<Rc<dyn Fn()>>,
}

#[derive(Clone, Default)]
pub struct InteractionTree {
    pub(crate) nodes: Vec<InteractionNode>,
    pub(crate) layers: Vec<LayerNode>,
    index: HashMap<FocusId, usize>,
}

impl InteractionTree {
    pub fn from_node(root: &VNode) -> Self {
        fn visit(
            node: &VNode,
            parent: Option<FocusId>,
            ancestry: &mut Vec<FocusId>,
            tree: &mut InteractionTree,
        ) {
            match node {
                VNode::Box(b) => {
                    if b.interaction.as_ref().is_some_and(|i| i.hidden) {
                        return;
                    }
                    let registered = b.interaction.is_some() || b.on_key.is_some();
                    let id = b.focus_id;
                    if registered {
                        let id = id.expect("interactive elements require an identity");
                        tree.push(InteractionNode {
                            id,
                            parent,
                            ancestors: ancestry.clone(),
                            interaction: b.interaction.as_deref().cloned().unwrap_or_default(),
                            on_key: b.on_key.clone(),
                            on_activate: b.on_click.clone(),
                            on_scroll: b.on_scroll.clone(),
                        });
                    }
                    if let Some(id) = id {
                        ancestry.push(id);
                    }
                    for child in &b.children {
                        visit(child, if registered { id } else { parent }, ancestry, tree);
                    }
                    if id.is_some() {
                        ancestry.pop();
                    }
                }
                VNode::Overlay(o) => {
                    if let Some(trigger) = &o.trigger {
                        visit(trigger, parent, ancestry, tree);
                    }
                    let opener = o
                        .opener
                        .or_else(|| o.trigger.as_deref().and_then(first_focus));
                    let owner = opener.filter(|&id| tree.node(id).is_some()).or(parent);
                    tree.layers.push(LayerNode {
                        id: o.id,
                        parent: owner,
                        opener,
                        config: o.layer,
                        active: o.open,
                        managed: o.managed,
                        on_dismiss: o.on_close.clone(),
                        on_open: o.on_open.clone(),
                    });
                    if o.open {
                        let pushed_opener = opener.filter(|id| !ancestry.contains(id));
                        if let Some(id) = pushed_opener {
                            ancestry.push(id);
                        }
                        tree.push(InteractionNode {
                            id: o.id,
                            parent: owner,
                            ancestors: ancestry.clone(),
                            interaction: Interaction {
                                focusability: Focusability::Programmatic,
                                scope: Some(o.layer.scope),
                                group: o.focus_group,
                                semantics: o.semantics.clone(),
                                ..Interaction::default()
                            },
                            on_key: o.on_key.clone(),
                            on_activate: None,
                            on_scroll: None,
                        });
                        ancestry.push(o.id);
                        if let Some(content) = &o.content {
                            visit(content, Some(o.id), ancestry, tree);
                        }
                        ancestry.pop();
                        if pushed_opener.is_some() {
                            ancestry.pop();
                        }
                    }
                }
                VNode::Fragment(items) => {
                    for item in items {
                        visit(item, parent, ancestry, tree);
                    }
                }
                _ => {}
            }
        }
        let mut tree = Self::default();
        visit(root, None, &mut Vec::new(), &mut tree);
        for layer in &tree.layers {
            if let Some(id) = layer.opener
                && let Some(node) = tree.nodes.iter_mut().find(|n| n.id == id)
            {
                node.interaction.semantics.controls = Some(layer.id);
                node.interaction.semantics.expanded = Some(layer.active);
                node.interaction.semantics.has_popup = Some(if layer.config.scope.contain_tab {
                    Role::Dialog
                } else if layer.config.tab == TabDismiss::Any {
                    Role::Menu
                } else {
                    Role::Generic
                });
            }
        }
        tree
    }
    fn push(&mut self, node: InteractionNode) {
        assert!(
            self.index.insert(node.id, self.nodes.len()).is_none(),
            "duplicate interaction identity {:?}",
            node.id
        );
        self.nodes.push(node);
    }
    pub(crate) fn node_mut(&mut self, id: FocusId) -> Option<&mut InteractionNode> {
        let index = *self.index.get(&id)?;
        self.nodes.get_mut(index)
    }
    pub fn nodes(&self) -> &[InteractionNode] {
        &self.nodes
    }
    pub fn layers(&self) -> &[LayerNode] {
        &self.layers
    }
    pub fn node(&self, id: FocusId) -> Option<&InteractionNode> {
        self.index.get(&id).and_then(|&index| self.nodes.get(index))
    }
    pub fn contains(&self, id: FocusId, ancestor: FocusId) -> bool {
        id == ancestor
            || self
                .node(id)
                .is_some_and(|n| n.ancestors.contains(&ancestor))
    }
    pub(crate) fn group(&self, id: FocusId) -> Option<&InteractionNode> {
        let node = self.node(id)?;
        if node.interaction.scope.is_some() {
            return None;
        }
        for &ancestor in node.ancestors.iter().rev() {
            if let Some(owner) = self.node(ancestor) {
                if owner.interaction.group.is_some() {
                    return Some(owner);
                }
                if owner.interaction.scope.is_some() {
                    return None;
                }
            }
        }
        None
    }
}

fn first_focus(node: &VNode) -> Option<FocusId> {
    match node {
        VNode::Box(b) => b
            .focus_id
            .filter(|_| {
                b.interaction
                    .as_ref()
                    .is_some_and(|i| i.focusability != Focusability::None)
            })
            .or_else(|| b.children.iter().find_map(first_focus)),
        VNode::Fragment(items) => items.iter().find_map(first_focus),
        VNode::Overlay(o) => o.trigger.as_deref().and_then(first_focus),
        _ => None,
    }
}
