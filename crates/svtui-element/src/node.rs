use crate::focus::FocusId;
use crate::interaction::{
    Activation, DismissLayer, EventContext, EventResult, FocusGroup, FocusScope, Focusability,
    Interaction, KeyHandler, Semantics,
};
use crossterm::event::KeyEvent;
use crossterm::style::Color;
use std::fmt;
use std::rc::Rc;
use svtui_layout::{
    Align, BorderStyle, FlexDirection, Justify, Overflow, Padding, Placement, ScrollbarStyle,
    ScrollbarVisibility, Size,
};

/// A single-line flex container with border-box sizing in terminal cells.
///
/// Column is the default: `justify_content` controls vertical placement and
/// `align_items` controls horizontal placement. Row exchanges those axes.
/// `gap` adds spacing between children without changing their source order.
///
/// On the main axis, Fill children share the space left by fixed, percentage,
/// and content-sized children. On the cross axis, Fill occupies the available
/// dimension. Stretch expands Auto cross sizes only. Both honor min/max bounds;
/// if a minimum exceeds a maximum, the minimum wins. Content never implicitly
/// shrinks below its minimum: use overflow settings to clip or scroll it.
///
/// Widths/heights include this box's border and padding. Percentages refer to
/// the parent's content viewport after scrollbar reservation. During intrinsic
/// measurement of an Auto parent, dependent percentages/Fill use content sizes
/// until the parent has a definite size. Wrapping and weighted grow/shrink are
/// not currently supported.
#[derive(Clone, Default)]
pub struct BoxElement {
    pub border: BorderStyle,
    pub border_color: Option<Color>,
    pub title: Option<String>,
    pub flex_direction: FlexDirection,
    pub gap: u16,
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
    pub on_click_at: Option<Rc<dyn Fn(u16, u16)>>,
    pub focus_id: Option<FocusId>,
    pub on_key: Option<KeyHandler>,
    pub interaction: Option<Box<Interaction>>,
    pub on_scroll: Option<Rc<dyn Fn(i16, i16)>>,
    pub cursor_offset: Option<(u16, u16)>,
    pub children: Vec<VNode>,
}

impl fmt::Debug for BoxElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BoxElement")
            .field("border", &self.border)
            .field("border_color", &self.border_color)
            .field("title", &self.title)
            .field("flex_direction", &self.flex_direction)
            .field("gap", &self.gap)
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

    /// Select the main axis. Column is the default.
    pub fn flex_direction(mut self, direction: FlexDirection) -> Self {
        self.flex_direction = direction;
        self
    }

    /// Space between adjacent children along the main axis, in cells.
    pub fn gap(mut self, gap: u16) -> Self {
        self.gap = gap;
        self
    }

    /// Align each child along the cross axis.
    pub fn align_items(mut self, align: Align) -> Self {
        self.align_items = align;
        self
    }

    /// Distribute children along the main axis.
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

    pub fn opt_bg(mut self, color: Option<Color>) -> Self {
        self.bg = color;
        self
    }

    pub fn fg(mut self, color: Color) -> Self {
        self.fg = Some(color);
        self
    }

    pub fn opt_fg(mut self, color: Option<Color>) -> Self {
        self.fg = color;
        self
    }

    pub fn opt_border_color(mut self, color: Option<Color>) -> Self {
        self.border_color = color;
        self
    }

    pub fn opt_focus_id(mut self, id: Option<FocusId>) -> Self {
        self.focus_id = id;
        self
    }

    pub fn opt_on_click(mut self, handler: Option<Rc<dyn Fn()>>) -> Self {
        self.on_click = handler;
        self
    }

    pub fn opt_on_key(mut self, handler: Option<KeyHandler>) -> Self {
        if handler.is_some() {
            self.ensure_interaction_id();
        }
        self.on_key = handler;
        self
    }

    pub fn on_click(mut self, handler: impl Fn() + 'static) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }

    pub fn on_click_at<F>(mut self, handler: F) -> Self
    where
        F: Fn(u16, u16) + 'static,
    {
        self.on_click_at = Some(Rc::new(handler));
        self
    }

    pub fn cursor_offset(mut self, x: u16, y: u16) -> Self {
        self.cursor_offset = Some((x, y));
        self
    }

    /// Assign identity without opting into Tab navigation.
    pub fn focus_id(mut self, id: impl Into<FocusId>) -> Self {
        self.focus_id = Some(id.into());
        self
    }

    pub fn on_key(
        mut self,
        handler: impl Fn(KeyEvent, &mut EventContext) -> EventResult + 'static,
    ) -> Self {
        self.ensure_interaction_id();
        self.on_key = Some(Rc::new(handler));
        self
    }

    fn ensure_interaction_id(&mut self) {
        if self.focus_id.is_none() {
            self.focus_id = Some(FocusId::default());
        }
    }

    fn interaction_mut(&mut self) -> &mut Interaction {
        self.ensure_interaction_id();
        self.interaction
            .get_or_insert_with(|| Box::new(Interaction::default()))
            .as_mut()
    }

    /// Opt into keyboard focus independently of identity and pointer handlers.
    pub fn focusability(mut self, value: Focusability) -> Self {
        self.interaction_mut().focusability = value;
        self
    }

    pub fn tab_stop(self, value: bool) -> Self {
        if !value && self.interaction.is_none() {
            return self;
        }
        self.focusability(if value {
            Focusability::TabStop
        } else {
            Focusability::None
        })
    }

    pub fn disabled(mut self, value: bool) -> Self {
        self.interaction_mut().disabled = value;
        self
    }

    pub fn hidden(mut self, value: bool) -> Self {
        self.interaction_mut().hidden = value;
        self
    }

    pub fn activation(mut self, value: Activation) -> Self {
        self.interaction_mut().activation = value;
        self
    }

    pub fn focus_scope(mut self, scope: FocusScope) -> Self {
        let interaction = self.interaction_mut();
        interaction.scope = Some(scope);
        if interaction.focusability == Focusability::None {
            interaction.focusability = Focusability::Programmatic;
        }
        self
    }

    pub fn focus_group(mut self, group: FocusGroup) -> Self {
        self.interaction_mut().group = Some(group);
        self
    }

    pub fn semantics(mut self, semantics: Semantics) -> Self {
        self.interaction_mut().semantics = semantics;
        self
    }

    pub fn on_scroll(mut self, handler: impl Fn(i16, i16) + 'static) -> Self {
        self.on_scroll = Some(Rc::new(handler));
        self
    }

    pub fn child(mut self, child: impl IntoNode) -> Self {
        match child.into_node() {
            VNode::Fragment(items) => self.children.extend(items),
            other => self.children.push(other),
        }
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
    /// Repeat this character after each line's content to its allocated width.
    /// This does not add intrinsic width; use a stretching parent to expand it.
    pub fill: Option<char>,
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

    /// Fill the unused cells of each rendered line with a character.
    /// An empty string fills one line. Content is clipped to the allocation.
    pub fn fill(mut self, character: char) -> Self {
        self.fill = Some(character);
        self
    }

    pub fn fg(mut self, color: Color) -> Self {
        self.fg = Some(color);
        self
    }

    pub fn opt_fg(mut self, color: Option<Color>) -> Self {
        self.fg = color;
        self
    }

    pub fn bg(mut self, color: Color) -> Self {
        self.bg = Some(color);
        self
    }

    pub fn opt_bg(mut self, color: Option<Color>) -> Self {
        self.bg = color;
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

/// Floating overlay element rendered in the top layer.
#[derive(Clone, Default)]
pub struct OverlayElement {
    pub id: FocusId,
    pub opener: Option<FocusId>,
    pub layer: DismissLayer,
    pub focus_group: Option<FocusGroup>,
    pub managed: bool,
    pub on_key: Option<KeyHandler>,
    pub semantics: Semantics,
    pub trigger: Option<Box<VNode>>,
    pub content: Option<Box<VNode>>,
    pub open: bool,
    pub placement: Placement,
    pub backdrop: bool,
    pub modal: bool,
    pub on_close: Option<Rc<dyn Fn()>>,
    pub on_open: Option<Rc<dyn Fn()>>,
}

impl fmt::Debug for OverlayElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OverlayElement")
            .field("trigger", &self.trigger)
            .field("content", &self.content)
            .field("open", &self.open)
            .field("placement", &self.placement)
            .field("backdrop", &self.backdrop)
            .field("modal", &self.modal)
            .field("has_close_handler", &self.on_close.is_some())
            .finish()
    }
}

impl OverlayElement {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn id(mut self, id: FocusId) -> Self {
        self.id = id;
        self
    }
    pub fn opener(mut self, id: FocusId) -> Self {
        self.opener = Some(id);
        self
    }
    pub fn layer(mut self, layer: DismissLayer) -> Self {
        self.modal = layer.scope.contain_tab;
        self.layer = layer;
        self
    }
    pub fn managed(mut self, managed: bool) -> Self {
        self.managed = managed;
        self
    }
    pub fn semantics(mut self, semantics: Semantics) -> Self {
        self.semantics = semantics;
        self
    }

    pub fn focus_group(mut self, group: FocusGroup) -> Self {
        self.focus_group = Some(group);
        self
    }
    pub fn opt_on_key(mut self, handler: Option<KeyHandler>) -> Self {
        self.on_key = handler;
        self
    }

    pub fn on_key(
        mut self,
        handler: impl Fn(KeyEvent, &mut EventContext) -> EventResult + 'static,
    ) -> Self {
        self.on_key = Some(Rc::new(handler));
        self
    }

    pub fn trigger(mut self, trigger: impl IntoNode) -> Self {
        let node = trigger.into_node();
        if matches!(node, VNode::Empty) {
            self.trigger = None;
        } else {
            self.trigger = Some(Box::new(node));
        }
        self
    }

    pub fn opt_trigger(mut self, trigger: Option<impl IntoNode>) -> Self {
        self.trigger = trigger.and_then(|t| {
            let node = t.into_node();
            if matches!(node, VNode::Empty) {
                None
            } else {
                Some(Box::new(node))
            }
        });
        self
    }

    pub fn content(mut self, content: impl IntoNode) -> Self {
        let node = content.into_node();
        if matches!(node, VNode::Empty) {
            self.content = None;
        } else {
            self.content = Some(Box::new(node));
        }
        self
    }

    pub fn child(self, child: impl IntoNode) -> Self {
        self.content(child)
    }

    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
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

    pub fn modal(mut self, modal: bool) -> Self {
        self.modal = modal;
        self.layer = if modal {
            DismissLayer::dialog()
        } else {
            DismissLayer::popover()
        };
        self
    }

    pub fn on_open(mut self, handler: impl Fn() + 'static) -> Self {
        self.on_open = Some(Rc::new(handler));
        self
    }
    pub fn opt_on_open(mut self, handler: Option<Rc<dyn Fn()>>) -> Self {
        self.on_open = handler;
        self
    }

    pub fn on_close(mut self, f: impl Fn() + 'static) -> Self {
        self.on_close = Some(Rc::new(f));
        self
    }

    pub fn opt_on_close(mut self, f: Option<Rc<dyn Fn()>>) -> Self {
        self.on_close = f;
        self
    }
}

// BoxElement is the fundamental layout container in svtui; keeping it unboxed in VNode
// avoids unnecessary heap allocation indirection on every node creation and pattern match.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug)]
pub enum VNode {
    Box(BoxElement),
    Text(TextElement),
    Overlay(OverlayElement),
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

impl IntoNode for OverlayElement {
    fn into_node(self) -> VNode {
        VNode::Overlay(self)
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
