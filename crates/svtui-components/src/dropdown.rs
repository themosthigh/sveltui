use crossterm::event::KeyCode;
use crossterm::style::Color;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use svtui_element::{FocusId, IntoNode, VNode, set_focused, use_focus};
use svtui_layout::{BorderStyle, FlexDirection, Justify, Overflow, Padding, Placement, Size};
use svtui_macros::view;
use svtui_reactive::Runtime;

thread_local! {
    static MENU_ITEM_IDS: RefCell<HashSet<FocusId>> = RefCell::new(HashSet::new());
    static SUBMENU_OPEN_STATES: RefCell<HashMap<FocusId, bool>> = RefCell::new(HashMap::new());
    static SUBMENU_STACK: RefCell<Vec<FocusId>> = const { RefCell::new(Vec::new()) };
}

pub fn register_menu_item(id: FocusId) {
    MENU_ITEM_IDS.with(|s| s.borrow_mut().insert(id));
}

pub fn is_menu_item(id: FocusId) -> bool {
    MENU_ITEM_IDS.with(|s| s.borrow().contains(&id))
}

pub fn is_submenu_open(id: FocusId) -> bool {
    SUBMENU_OPEN_STATES.with(|m| *m.borrow().get(&id).unwrap_or(&false))
}

pub fn open_submenu(id: FocusId) {
    SUBMENU_OPEN_STATES.with(|m| {
        m.borrow_mut().insert(id, true);
    });
    SUBMENU_STACK.with(|s| {
        let mut stack = s.borrow_mut();
        if !stack.contains(&id) {
            stack.push(id);
        }
    });
    Runtime::set_dirty(true);
}

pub fn close_submenu(id: FocusId) {
    SUBMENU_OPEN_STATES.with(|m| {
        m.borrow_mut().insert(id, false);
    });
    SUBMENU_STACK.with(|s| {
        s.borrow_mut().retain(|&x| x != id);
    });
    Runtime::set_dirty(true);
}

pub fn close_topmost_submenu() -> Option<FocusId> {
    let popped = SUBMENU_STACK.with(|s| s.borrow_mut().pop());
    if let Some(id) = popped {
        SUBMENU_OPEN_STATES.with(|m| {
            m.borrow_mut().insert(id, false);
        });
        set_focused(Some(id));
        Runtime::set_dirty(true);
        Some(id)
    } else {
        None
    }
}

pub fn has_open_submenus() -> bool {
    SUBMENU_STACK.with(|s| !s.borrow().is_empty())
}

pub fn close_all_submenus() {
    SUBMENU_OPEN_STATES.with(|m| {
        m.borrow_mut().clear();
    });
    SUBMENU_STACK.with(|s| {
        s.borrow_mut().clear();
    });
    Runtime::set_dirty(true);
}

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
        register_menu_item(id);
        let focus = use_focus(id);
        let is_active = focus.is_active();

        let focus_bg = self.focus_bg.unwrap_or(Color::White);
        let focus_fg = self.focus_fg.unwrap_or(Color::Black);

        let label_text = format!(" {}", self.label);

        let label_fg = if self.disabled {
            Color::DarkGrey
        } else if is_active {
            focus_fg
        } else {
            Color::White
        };

        let row_bg = if is_active && !self.disabled {
            Some(focus_bg)
        } else {
            None
        };

        let on_select = self.on_select.clone();
        let on_select_click = self.on_select.clone();

        let click_handler: Option<Rc<dyn Fn()>> = if !self.disabled {
            Some(Rc::new(move || {
                close_all_submenus();
                if let Some(ref sel) = on_select_click {
                    sel();
                }
                Runtime::set_dirty(true);
            }))
        } else {
            None
        };

        let key_handler: Option<Rc<dyn Fn(crossterm::event::KeyEvent)>> = if !self.disabled {
            Some(Rc::new(move |key: crossterm::event::KeyEvent| {
                if key.code == KeyCode::Enter || key.code == KeyCode::Char(' ') {
                    close_all_submenus();
                    if let Some(ref sel) = on_select {
                        sel();
                    }
                    Runtime::set_dirty(true);
                } else if key.code == KeyCode::Left {
                    close_topmost_submenu();
                }
            }))
        } else {
            None
        };

        view! {
            <Box
                width=Size::Fill
                padding=Padding::zero()
                flex_direction=FlexDirection::Row
                overflow=Overflow::Hidden
                justify_content=Justify::SpaceBetween
                focus_id=id
                opt_bg=row_bg
                opt_on_click=click_handler
                opt_on_key=key_handler
            >
                <Text fg=label_fg bold=is_active>
                    {label_text}
                </Text>
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
    bg: Option<Color>,
    on_close: Option<Rc<dyn Fn()>>,
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
            bg: None,
            on_close: None,
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
        register_menu_item(id);
        let is_open = self.open.unwrap_or_else(|| is_submenu_open(id));
        let focus = use_focus(id);
        let is_active = focus.is_active() || is_open;

        let focus_bg = self.focus_bg.unwrap_or(Color::White);
        let focus_fg = self.focus_fg.unwrap_or(Color::Black);

        let label_text = format!(" {}", self.label);

        let label_fg = if self.disabled {
            Color::DarkGrey
        } else if is_active {
            focus_fg
        } else {
            Color::White
        };

        let arrow_fg = if is_active && !self.disabled {
            focus_fg
        } else {
            Color::DarkGrey
        };

        let row_bg = if is_active && !self.disabled {
            Some(focus_bg)
        } else {
            None
        };

        let id_click = id;
        let cur_open = is_open;
        let on_close_click = self.on_close.clone();

        let click_handler: Option<Rc<dyn Fn()>> = if !self.disabled {
            Some(Rc::new(move || {
                let next_open = !cur_open;
                if next_open {
                    open_submenu(id_click);
                } else {
                    close_submenu(id_click);
                    if let Some(ref close) = on_close_click {
                        close();
                    }
                }
                Runtime::set_dirty(true);
            }))
        } else {
            None
        };

        let id_key = id;
        let key_handler: Option<Rc<dyn Fn(crossterm::event::KeyEvent)>> = if !self.disabled {
            Some(Rc::new(move |key: crossterm::event::KeyEvent| {
                match key.code {
                    KeyCode::Right | KeyCode::Enter | KeyCode::Char(' ') => {
                        open_submenu(id_key);
                    }
                    KeyCode::Left => {
                        close_topmost_submenu();
                    }
                    _ => {}
                }
            }))
        } else {
            None
        };

        let on_close_handler = self.on_close.clone();
        let submenu_box_id = FocusId::new();
        let menu_bg = self.bg.unwrap_or(Color::Black);

        view! {
            <Overlay
                open=is_open
                placement=self.placement
                modal=false
                backdrop=false
                trigger={
                    view! {
                        <Box
                            width=Size::Fill
                            padding=Padding::zero()
                            flex_direction=FlexDirection::Row
                            overflow=Overflow::Hidden
                            justify_content=Justify::SpaceBetween
                            focus_id=id
                            opt_bg=row_bg
                            opt_on_click=click_handler
                            opt_on_key=key_handler
                        >
                            <Box width=Size::Fill overflow=Overflow::Hidden>
                                <Text fg=label_fg bold=is_active>
                                    {label_text}
                                </Text>
                            </Box>
                            <Text fg=arrow_fg bold=is_active>
                                "▶ "
                            </Text>
                        </Box>
                    }
                }
                on_close={move || {
                    close_submenu(id);
                    if let Some(ref close) = on_close_handler {
                        close();
                    }
                    Runtime::set_dirty(true);
                }}
            >
                <Box
                    focus_id=submenu_box_id
                    border=BorderStyle::Single
                    border_color=Color::White
                    bg=menu_bg
                    width=self.width
                    padding=Padding::zero()
                >
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
    open: bool,
    placement: Placement,
    width: Size,
    bg: Option<Color>,
    on_close: Option<Rc<dyn Fn()>>,
    children: Vec<VNode>,
}

impl Default for DropdownMenu {
    fn default() -> Self {
        Self {
            trigger: None,
            open: false,
            placement: Placement::BottomStart,
            width: Size::Fixed(24),
            bg: None,
            on_close: None,
            children: Vec::new(),
        }
    }
}

impl DropdownMenu {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn trigger(mut self, trigger: impl IntoNode) -> Self {
        self.trigger = Some(trigger.into_node());
        self
    }

    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
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
        let menu_box_id = FocusId::new();
        let menu_bg = self.bg.unwrap_or(Color::Black);
        let on_close = self.on_close.clone();

        view! {
            <Overlay
                open=self.open
                placement=self.placement
                modal=false
                backdrop=false
                opt_trigger=self.trigger
                opt_on_close={on_close.map(|h| {
                    Rc::new(move || {
                        close_all_submenus();
                        h();
                    }) as Rc<dyn Fn()>
                })}
            >
                <Box
                    focus_id=menu_box_id
                    border=BorderStyle::Single
                    border_color=Color::White
                    bg=menu_bg
                    width=self.width
                    padding=Padding::zero()
                >
                    {self.children}
                </Box>
            </Overlay>
        }
    }
}
