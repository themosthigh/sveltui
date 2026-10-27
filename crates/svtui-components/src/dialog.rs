use crossterm::style::Color;
use std::rc::Rc;
use svtui_element::{BoxElement, IntoNode, OverlayElement, VNode};
use svtui_layout::{BorderStyle, Padding, Placement, Size};

/// Modal dialog component with top-layer deferred rendering, scrim backdrop,
/// and keyboard focus trapping.
#[derive(Clone)]
pub struct Dialog {
    open: bool,
    title: Option<String>,
    border: BorderStyle,
    border_color: Option<Color>,
    width: Size,
    height: Size,
    padding: Padding,
    on_close: Option<Rc<dyn Fn()>>,
    children: Vec<VNode>,
    placement: Placement,
    backdrop: bool,
}

impl Default for Dialog {
    fn default() -> Self {
        Self {
            open: false,
            title: None,
            border: BorderStyle::Single,
            border_color: Some(Color::White),
            width: Size::Fixed(50),
            height: Size::Auto,
            padding: Padding::all(1),
            on_close: None,
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

        let mut overlay = OverlayElement::new()
            .open(self.open)
            .placement(self.placement)
            .modal(true)
            .backdrop(self.backdrop)
            .content(b);

        if let Some(h) = self.on_close {
            overlay.on_close = Some(h);
        }

        overlay.into_node()
    }
}
