use crossterm::event::KeyEvent;
use crossterm::style::Color;
use std::rc::Rc;
use svtui_element::{BoxElement, IntoNode, OverlayElement, VNode};
use svtui_element::{DismissLayer, FocusId, InitialFocus, Role, Semantics};
use svtui_element::{EventContext, EventResult, KeyHandler};
use svtui_layout::{BorderStyle, Padding, Placement, Size};

/// Modal dialog component with top-layer deferred rendering, scrim backdrop,
/// and keyboard focus trapping.
#[derive(Clone)]
pub struct Dialog {
    open: bool,
    id: Option<FocusId>,
    opener: Option<FocusId>,
    initial_focus: InitialFocus,
    title: Option<String>,
    border: BorderStyle,
    border_color: Option<Color>,
    width: Size,
    height: Size,
    padding: Padding,
    on_close: Option<Rc<dyn Fn()>>,
    on_open: Option<Rc<dyn Fn()>>,
    on_key: Option<KeyHandler>,
    children: Vec<VNode>,
    placement: Placement,
    backdrop: bool,
}

impl Default for Dialog {
    fn default() -> Self {
        Self {
            open: false,
            id: None,
            opener: None,
            initial_focus: InitialFocus::First,
            title: None,
            border: BorderStyle::Single,
            border_color: Some(Color::White),
            width: Size::Fixed(50),
            height: Size::Auto,
            padding: Padding::all(1),
            on_close: None,
            on_open: None,
            on_key: None,
            children: Vec::new(),
            placement: Placement::Center,
            backdrop: true,
        }
    }
}

impl Dialog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn id(mut self, id: FocusId) -> Self {
        self.id = Some(id);
        self
    }
    pub fn opener(mut self, id: FocusId) -> Self {
        self.opener = Some(id);
        self
    }
    pub fn initial_focus(mut self, target: InitialFocus) -> Self {
        self.initial_focus = target;
        self
    }

    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn border(mut self, border: BorderStyle) -> Self {
        self.border = border;
        self
    }

    pub fn border_color(mut self, color: Color) -> Self {
        self.border_color = Some(color);
        self
    }

    pub fn width(mut self, width: impl Into<Size>) -> Self {
        self.width = width.into();
        self
    }

    pub fn height(mut self, height: impl Into<Size>) -> Self {
        self.height = height.into();
        self
    }

    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = padding.into();
        self
    }

    pub fn on_key(
        mut self,
        handler: impl Fn(KeyEvent, &mut EventContext) -> EventResult + 'static,
    ) -> Self {
        self.on_key = Some(Rc::new(handler));
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

    pub fn placement(mut self, placement: Placement) -> Self {
        self.placement = placement;
        self
    }

    pub fn backdrop(mut self, backdrop: bool) -> Self {
        self.backdrop = backdrop;
        self
    }

    pub fn child(mut self, child: impl IntoNode) -> Self {
        self.children.push(child.into_node());
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = impl IntoNode>) -> Self {
        self.children
            .extend(children.into_iter().map(|c| c.into_node()));
        self
    }
}

impl IntoNode for Dialog {
    fn into_node(self) -> VNode {
        let semantics = Semantics {
            modal: Some(true),
            ..Semantics::new(Role::Dialog, self.title.clone().unwrap_or_default())
        };
        let mut b = BoxElement::new()
            .border(self.border)
            .width(self.width)
            .height(self.height)
            .padding(self.padding);

        if let Some(color) = self.border_color {
            b = b.border_color(color);
        }
        if let Some(title) = self.title {
            b = b.title(title);
        }
        for child in self.children {
            b = b.child(child);
        }

        let mut layer = DismissLayer::dialog();
        layer.scope.initial_focus = self.initial_focus;
        let mut overlay = OverlayElement::new()
            .id(self.id.unwrap_or_default())
            .semantics(semantics)
            .open(self.open)
            .placement(self.placement)
            .layer(layer)
            .backdrop(self.backdrop)
            .content(b);

        overlay.opener = self.opener;
        overlay.on_key = self.on_key;
        overlay.on_open = self.on_open;
        if let Some(h) = self.on_close {
            overlay.on_close = Some(h);
        }

        overlay.into_node()
    }
}
