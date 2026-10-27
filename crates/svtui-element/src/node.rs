use crate::focus::FocusId;
use crossterm::event::KeyEvent;
use crossterm::style::Color;
use std::fmt;
use std::rc::Rc;
use svtui_layout::{
    Align, BorderStyle, Justify, Overflow, Padding, ScrollbarStyle, ScrollbarVisibility, Size,
};

#[derive(Clone, Default)]
pub struct BoxElement {
    pub border: BorderStyle,
    pub border_color: Option<Color>,
    pub title: Option<String>,
    pub align_items: Align,
    pub justify_content: Justify,
    pub width: Size,
    pub height: Size,
    pub min_width: Option<Size>,
    pub max_width: Option<Size>,
    pub min_height: Option<Size>,
    pub max_height: Option<Size>,
    pub overflow_x: Overflow,
    pub overflow_y: Overflow,
    pub scroll_x: Option<u16>,
    pub scroll_y: Option<u16>,
    pub scrollbar_visibility: ScrollbarVisibility,
    pub scrollbar_style: ScrollbarStyle,
    pub padding: Padding,
    pub bg: Option<Color>,
    pub fg: Option<Color>,
    pub on_click: Option<Rc<dyn Fn()>>,
    pub focus_id: Option<FocusId>,
    pub on_key: Option<Rc<dyn Fn(KeyEvent)>>,
    pub children: Vec<VNode>,
}

impl fmt::Debug for BoxElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BoxElement")
            .field("border", &self.border)
            .field("border_color", &self.border_color)
            .field("title", &self.title)
            .field("align_items", &self.align_items)
            .field("justify_content", &self.justify_content)
            .field("width", &self.width)
            .field("height", &self.height)
            .field("min_width", &self.min_width)
            .field("max_width", &self.max_width)
            .field("min_height", &self.min_height)
            .field("max_height", &self.max_height)
            .field("overflow_x", &self.overflow_x)
            .field("overflow_y", &self.overflow_y)
            .field("scroll_x", &self.scroll_x)
            .field("scroll_y", &self.scroll_y)
            .field("scrollbar_visibility", &self.scrollbar_visibility)
            .field("scrollbar_style", &self.scrollbar_style)
            .field("padding", &self.padding)
            .field("bg", &self.bg)
            .field("fg", &self.fg)
            .field("focus_id", &self.focus_id)
            .field("has_click_handler", &self.on_click.is_some())
            .field("children", &self.children)
            .finish()
    }
}

impl BoxElement {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn border(mut self, border: BorderStyle) -> Self {
        self.border = border;
        self
    }

    pub fn border_color(mut self, color: Color) -> Self {
        self.border_color = Some(color);
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn align_items(mut self, align: Align) -> Self {
        self.align_items = align;
        self
    }

    pub fn justify_content(mut self, justify: Justify) -> Self {
        self.justify_content = justify;
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

    pub fn min_width(mut self, min_width: impl Into<Size>) -> Self {
        self.min_width = Some(min_width.into());
        self
    }

    pub fn max_width(mut self, max_width: impl Into<Size>) -> Self {
        self.max_width = Some(max_width.into());
        self
    }

    pub fn min_height(mut self, min_height: impl Into<Size>) -> Self {
        self.min_height = Some(min_height.into());
        self
    }

    pub fn max_height(mut self, max_height: impl Into<Size>) -> Self {
        self.max_height = Some(max_height.into());
        self
    }

    pub fn full_width(mut self) -> Self {
        self.width = Size::Fill;
        self
    }

    pub fn full_height(mut self) -> Self {
        self.height = Size::Fill;
        self
    }

    pub fn overflow(mut self, overflow: Overflow) -> Self {
        self.overflow_x = overflow;
        self.overflow_y = overflow;
        self
    }

    pub fn overflow_x(mut self, overflow: Overflow) -> Self {
        self.overflow_x = overflow;
        self
    }

    pub fn overflow_y(mut self, overflow: Overflow) -> Self {
        self.overflow_y = overflow;
        self
    }

    pub fn scroll_x(mut self, x: u16) -> Self {
        self.scroll_x = Some(x);
        self
    }

    pub fn scroll_y(mut self, y: u16) -> Self {
        self.scroll_y = Some(y);
        self
    }

    pub fn scrollbar_visibility(mut self, vis: ScrollbarVisibility) -> Self {
        self.scrollbar_visibility = vis;
        self
    }

    pub fn scrollbar_style(mut self, style: ScrollbarStyle) -> Self {
        self.scrollbar_style = style;
        self
    }

    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = padding.into();
        self
    }

    pub fn bg(mut self, color: Color) -> Self {
        self.bg = Some(color);
        self
    }

    pub fn fg(mut self, color: Color) -> Self {
        self.fg = Some(color);
        self
    }

    pub fn on_click(mut self, handler: impl Fn() + 'static) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }

    pub fn focus_id(mut self, id: impl Into<FocusId>) -> Self {
        self.focus_id = Some(id.into());
        self
    }

    pub fn on_key(mut self, handler: impl Fn(KeyEvent) + 'static) -> Self {
        self.on_key = Some(Rc::new(handler));
        self
    }

    pub fn child(mut self, child: impl IntoNode) -> Self {
        self.children.push(child.into_node());
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = impl IntoNode>) -> Self {
        for c in children {
            self.children.push(c.into_node());
        }
        self
    }
}

#[derive(Clone, Debug, Default)]
pub struct TextElement {
    pub content: String,
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

impl TextElement {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            ..Default::default()
        }
    }

    pub fn color(mut self, color: Color) -> Self {
        self.fg = Some(color);
        self
    }

    pub fn fg(mut self, color: Color) -> Self {
        self.fg = Some(color);
        self
    }

    pub fn bg(mut self, color: Color) -> Self {
        self.bg = Some(color);
        self
    }

    pub fn bold(mut self, bold: bool) -> Self {
        self.bold = bold;
        self
    }

    pub fn italic(mut self, italic: bool) -> Self {
        self.italic = italic;
        self
    }

    pub fn underline(mut self, underline: bool) -> Self {
        self.underline = underline;
        self
    }
}

#[derive(Clone, Debug)]
pub enum VNode {
    Box(BoxElement),
    Text(TextElement),
    Fragment(Vec<VNode>),
    Empty,
}

pub trait IntoNode {
    fn into_node(self) -> VNode;
}

impl IntoNode for VNode {
    fn into_node(self) -> VNode {
        self
    }
}

impl IntoNode for BoxElement {
    fn into_node(self) -> VNode {
        VNode::Box(self)
    }
}

impl IntoNode for TextElement {
    fn into_node(self) -> VNode {
        VNode::Text(self)
    }
}

impl IntoNode for &str {
    fn into_node(self) -> VNode {
        VNode::Text(TextElement::new(self))
    }
}

impl IntoNode for String {
    fn into_node(self) -> VNode {
        VNode::Text(TextElement::new(self))
    }
}

impl IntoNode for &String {
    fn into_node(self) -> VNode {
        VNode::Text(TextElement::new(self.as_str()))
    }
}

impl<T: IntoNode> IntoNode for Option<T> {
    fn into_node(self) -> VNode {
        match self {
            Some(v) => v.into_node(),
            None => VNode::Empty,
        }
    }
}

impl<T: IntoNode> IntoNode for Vec<T> {
    fn into_node(self) -> VNode {
        VNode::Fragment(self.into_iter().map(|item| item.into_node()).collect())
    }
}

impl IntoNode for () {
    fn into_node(self) -> VNode {
        VNode::Empty
    }
}
