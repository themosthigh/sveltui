use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::style::Color;
use std::rc::Rc;
use svtui_element::{
    Activation, DismissLayer, EventResult, FocusGroup, FocusId, IntoNode, Role, Semantics, VNode,
    is_layer_open, use_focus,
};
use svtui_element::{EventContext, KeyHandler};
use svtui_layout::{BorderStyle, FlexDirection, Overflow, Padding, Placement, Size};
use svtui_macros::view;

/// Interactive leaf item inside a dropdown menu.
#[derive(Clone, Default)]
pub struct DropdownItem {
    label: String,
    disabled: bool,
    id: Option<FocusId>,
    on_select: Option<Rc<dyn Fn()>>,
    focus_bg: Option<Color>,
    focus_fg: Option<Color>,
}

impl DropdownItem {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            disabled: false,
            id: None,
            on_select: None,
            focus_bg: None,
            focus_fg: None,
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    pub fn child(mut self, child: impl IntoNode) -> Self {
        if let VNode::Text(t) = child.into_node() {
            self.label = t.content;
        }
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn id(mut self, id: FocusId) -> Self {
        self.id = Some(id);
        self
    }

    pub fn focus_bg(mut self, color: Color) -> Self {
        self.focus_bg = Some(color);
        self
    }

    pub fn focus_fg(mut self, color: Color) -> Self {
        self.focus_fg = Some(color);
        self
    }

    pub fn on_select(mut self, handler: impl Fn() + 'static) -> Self {
        self.on_select = Some(Rc::new(handler));
        self
    }

    pub fn on_click(mut self, handler: impl Fn() + 'static) -> Self {
        self.on_select = Some(Rc::new(handler));
        self
    }
}

impl IntoNode for DropdownItem {
    fn into_node(self) -> VNode {
        let id = self.id.unwrap_or_default();
        let active = use_focus(id).is_active();
        let label_fg = if self.disabled {
            Color::DarkGrey
        } else if active {
            self.focus_fg.unwrap_or(Color::Black)
        } else {
            Color::White
        };
        let row_bg = active.then_some(self.focus_bg.unwrap_or(Color::White));
        let label_text = format!(" {}", self.label);
        let semantics = Semantics::new(Role::MenuItem, self.label);
        let select = self.on_select;
        view! {
            <Box width=Size::Fill padding=Padding::zero() flex_direction=FlexDirection::Row
                overflow=Overflow::Hidden focus_id=id tab_stop=true disabled={self.disabled}
                activation=Activation::Press semantics=semantics opt_bg=row_bg
                on_key={move |key, context| {
                    if matches!(key.code, KeyCode::Enter | KeyCode::Char(' ')) {
                        if let Some(select) = &select { select(); }
                        context.dismiss_chain();
                        EventResult::Handled
                    } else { EventResult::Ignored }
                }}>
                <Text fg=label_fg bold=active>{label_text}</Text>
            </Box>
        }
    }
}

/// Interactive submenu item inside a dropdown menu that reveals another menu layer.
#[derive(Clone)]
pub struct DropdownSubmenu {
    label: String,
    disabled: bool,
    id: Option<FocusId>,
    width: Size,
    placement: Placement,
    open: Option<bool>,
    layer_id: Option<FocusId>,
    bg: Option<Color>,
    on_close: Option<Rc<dyn Fn()>>,
    on_open: Option<Rc<dyn Fn()>>,
    children: Vec<VNode>,
    focus_bg: Option<Color>,
    focus_fg: Option<Color>,
}

impl Default for DropdownSubmenu {
    fn default() -> Self {
        Self {
            label: String::new(),
            disabled: false,
            id: None,
            width: Size::Fixed(24),
            placement: Placement::RightStart,
            open: None,
            layer_id: None,
            bg: None,
            on_close: None,
            on_open: None,
            children: Vec::new(),
            focus_bg: None,
            focus_fg: None,
        }
    }
}

impl DropdownSubmenu {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            ..Default::default()
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn id(mut self, id: FocusId) -> Self {
        self.id = Some(id);
        self
    }

    /// Supply a persistent layer ID when this item is dynamically reordered.
    pub fn layer_id(mut self, id: FocusId) -> Self {
        self.layer_id = Some(id);
        self
    }

    pub fn width(mut self, width: impl Into<Size>) -> Self {
        self.width = width.into();
        self
    }

    pub fn placement(mut self, placement: Placement) -> Self {
        self.placement = placement;
        self
    }

    pub fn open(mut self, open: bool) -> Self {
        self.open = Some(open);
        self
    }

    pub fn bg(mut self, color: Color) -> Self {
        self.bg = Some(color);
        self
    }

    pub fn on_open(mut self, handler: impl Fn() + 'static) -> Self {
        self.on_open = Some(Rc::new(handler));
        self
    }

    pub fn on_close(mut self, handler: impl Fn() + 'static) -> Self {
        self.on_close = Some(Rc::new(handler));
        self
    }

    pub fn focus_bg(mut self, color: Color) -> Self {
        self.focus_bg = Some(color);
        self
    }

    pub fn focus_fg(mut self, color: Color) -> Self {
        self.focus_fg = Some(color);
        self
    }

    pub fn child(mut self, child: impl IntoNode) -> Self {
        self.children.push(child.into_node());
        self
    }

    pub fn item(mut self, item: impl IntoNode) -> Self {
        self.children.push(item.into_node());
        self
    }
}

impl IntoNode for DropdownSubmenu {
    fn into_node(self) -> VNode {
        let id = self.id.unwrap_or_default();
        let layer_id = self.layer_id.unwrap_or_default();
        let managed = self.open.is_none();
        let is_open = self.open.unwrap_or_else(|| is_layer_open(layer_id));
        let active = use_focus(id).is_active() || is_open;
        let label_fg = if self.disabled {
            Color::DarkGrey
        } else if active {
            self.focus_fg.unwrap_or(Color::Black)
        } else {
            Color::White
        };
        let arrow_fg = if active && !self.disabled {
            label_fg
        } else {
            Color::DarkGrey
        };
        let row_bg = active.then_some(self.focus_bg.unwrap_or(Color::White));
        let label_text = format!(" {}", self.label);
        let semantics = Semantics::new(Role::MenuItem, self.label.clone())
            .expanded(is_open)
            .controls(layer_id);
        let layer_semantics = Semantics::new(Role::Menu, self.label);
        let menu_bg = self.bg.unwrap_or(Color::Black);
        view! {
            <Overlay id=layer_id opener=id open=is_open managed=managed layer=DismissLayer::submenu() focus_group=FocusGroup::menu()
                placement={self.placement} semantics=layer_semantics opt_on_close={self.on_close} opt_on_open={self.on_open}
                trigger={view! {
                    <Box width=Size::Fill padding=Padding::zero() flex_direction=FlexDirection::Row
                        overflow=Overflow::Hidden focus_id=id tab_stop=true disabled={self.disabled}
                        activation=Activation::Press semantics=semantics opt_bg=row_bg
                        on_key={move |key, context| {
                            if matches!(key.code, KeyCode::Right | KeyCode::Enter | KeyCode::Char(' ')) {
                                if is_open && key.code != KeyCode::Right { context.dismiss(layer_id); }
                                else { context.open(layer_id); }
                                EventResult::Handled
                            } else { EventResult::Ignored }
                        }}>
                        <Box width=Size::Fill overflow=Overflow::Hidden>
                            <Text fg=label_fg bold=active>{label_text}</Text>
                        </Box>
                        <Text fg=arrow_fg bold=active>"▶ "</Text>
                    </Box>
                }}>
                <Box border=BorderStyle::Single border_color=Color::White bg=menu_bg width={self.width}
                    padding=Padding::zero()>
                    {self.children}
                </Box>
            </Overlay>
        }
    }
}

/// Popover dropdown menu container.
#[derive(Clone)]
pub struct DropdownMenu {
    trigger: Option<VNode>,
    id: Option<FocusId>,
    open: Option<bool>,
    placement: Placement,
    width: Size,
    bg: Option<Color>,
    on_close: Option<Rc<dyn Fn()>>,
    on_open: Option<Rc<dyn Fn()>>,
    on_key: Option<KeyHandler>,
    children: Vec<VNode>,
}

impl Default for DropdownMenu {
    fn default() -> Self {
        Self {
            trigger: None,
            id: None,
            open: None,
            placement: Placement::BottomStart,
            width: Size::Fixed(24),
            bg: None,
            on_close: None,
            on_open: None,
            on_key: None,
            children: Vec::new(),
        }
    }
}

impl DropdownMenu {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn id(mut self, id: FocusId) -> Self {
        self.id = Some(id);
        self
    }

    pub fn trigger(mut self, trigger: impl IntoNode) -> Self {
        self.trigger = Some(trigger.into_node());
        self
    }

    pub fn open(mut self, open: bool) -> Self {
        self.open = Some(open);
        self
    }

    pub fn placement(mut self, placement: Placement) -> Self {
        self.placement = placement;
        self
    }

    pub fn width(mut self, width: impl Into<Size>) -> Self {
        self.width = width.into();
        self
    }

    pub fn bg(mut self, color: Color) -> Self {
        self.bg = Some(color);
        self
    }

    pub fn on_open(mut self, handler: impl Fn() + 'static) -> Self {
        self.on_open = Some(Rc::new(handler));
        self
    }

    pub fn on_key(
        mut self,
        handler: impl Fn(KeyEvent, &mut EventContext) -> EventResult + 'static,
    ) -> Self {
        self.on_key = Some(Rc::new(handler));
        self
    }

    pub fn on_close(mut self, handler: impl Fn() + 'static) -> Self {
        self.on_close = Some(Rc::new(handler));
        self
    }

    pub fn child(mut self, child: impl IntoNode) -> Self {
        self.children.push(child.into_node());
        self
    }

    pub fn item(mut self, item: impl IntoNode) -> Self {
        self.children.push(item.into_node());
        self
    }
}

impl IntoNode for DropdownMenu {
    fn into_node(self) -> VNode {
        let id = self.id.unwrap_or_default();
        let menu_bg = self.bg.unwrap_or(Color::Black);
        let managed = self.open.is_none();
        let open = self.open.unwrap_or_else(|| is_layer_open(id));
        view! {
            <Overlay id=id open=open managed=managed layer=DismissLayer::menu() focus_group=FocusGroup::menu() placement={self.placement}
                semantics={Semantics::new(Role::Menu, "Menu")} opt_trigger={self.trigger} opt_on_close={self.on_close} opt_on_key={self.on_key} opt_on_open={self.on_open}>
                <Box border=BorderStyle::Single border_color=Color::White bg=menu_bg width={self.width}
                    padding=Padding::zero()>
                    {self.children}
                </Box>
            </Overlay>
        }
    }
}
