use crate::focus::FocusId;
use crate::interaction::{
    Activation, DismissLayer, EventContext, EventResult, FocusGroup, FocusScope, Focusability,
    Interaction, KeyHandler, Semantics,
};
use crate::snippet::Snippet;
use crossterm::event::KeyEvent;
use crossterm::style::Color;
use std::fmt;
use std::rc::Rc;
use sveltui_layout::{
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
pub struct FlexBoxElement {
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
    pub focus_scope: Option<FocusScope>,
    pub children: Vec<VNode>,
}

impl fmt::Debug for FlexBoxElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FlexBoxElement")
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

impl FlexBoxElement {
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
    /// Wrap paragraphs at word boundaries within their allocated width.
    pub wrap: bool,
    /// Maximum visible lines; truncated content ends with an ellipsis.
    pub max_lines: Option<usize>,
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

    pub fn wrap(mut self, wrap: bool) -> Self {
        self.wrap = wrap;
        self
    }

    pub fn max_lines(mut self, max_lines: usize) -> Self {
        self.max_lines = Some(max_lines);
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
    /// Layout identity used for positioning, independently of focus restoration.
    pub anchor: Option<FocusId>,
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
    pub fn anchor(mut self, id: FocusId) -> Self {
        self.anchor = Some(id);
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

// FlexBoxElement is the fundamental layout container in sveltui; keeping it unboxed in VNode
// avoids unnecessary heap allocation indirection on every node creation and pattern match.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug)]
pub enum VNode {
    FlexBox(FlexBoxElement),
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

impl IntoNode for FlexBoxElement {
    fn into_node(self) -> VNode {
        VNode::FlexBox(self)
    }
}

pub trait IntoClickHandler {
    fn into_click_handler(self) -> Rc<dyn Fn()>;
}
impl<F: Fn() + 'static> IntoClickHandler for F {
    fn into_click_handler(self) -> Rc<dyn Fn()> {
        Rc::new(self)
    }
}
impl IntoClickHandler for Rc<dyn Fn()> {
    fn into_click_handler(self) -> Rc<dyn Fn()> {
        self
    }
}

pub trait IntoClickHandlerAt {
    fn into_click_handler_at(self) -> Rc<dyn Fn(u16, u16)>;
}
impl<F: Fn(u16, u16) + 'static> IntoClickHandlerAt for F {
    fn into_click_handler_at(self) -> Rc<dyn Fn(u16, u16)> {
        Rc::new(self)
    }
}
impl IntoClickHandlerAt for Rc<dyn Fn(u16, u16)> {
    fn into_click_handler_at(self) -> Rc<dyn Fn(u16, u16)> {
        self
    }
}

pub trait IntoKeyHandler {
    fn into_key_handler(self) -> KeyHandler;
}
impl<F: Fn(KeyEvent, &mut EventContext) -> EventResult + 'static> IntoKeyHandler for F {
    fn into_key_handler(self) -> KeyHandler {
        Rc::new(self)
    }
}
impl IntoKeyHandler for KeyHandler {
    fn into_key_handler(self) -> KeyHandler {
        self
    }
}

pub trait IntoScrollHandler {
    fn into_scroll_handler(self) -> Rc<dyn Fn(i16, i16)>;
}
impl<F: Fn(i16, i16) + 'static> IntoScrollHandler for F {
    fn into_scroll_handler(self) -> Rc<dyn Fn(i16, i16)> {
        Rc::new(self)
    }
}
impl IntoScrollHandler for Rc<dyn Fn(i16, i16)> {
    fn into_scroll_handler(self) -> Rc<dyn Fn(i16, i16)> {
        self
    }
}

#[derive(Clone, Default)]
pub struct FlexBoxProps {
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
    pub tab_stop: bool,
    pub disabled: bool,
    pub activation: Activation,
    pub semantics: Option<Semantics>,
    pub interaction: Option<std::boxed::Box<Interaction>>,
    pub on_scroll: Option<Rc<dyn Fn(i16, i16)>>,
    pub cursor_offset: Option<(u16, u16)>,
    pub focus_scope: Option<FocusScope>,
    pub children: Vec<VNode>,
}

#[derive(Clone, Default)]
pub struct FlexBoxPropsBuilder {
    props: FlexBoxProps,
}

impl FlexBoxProps {
    pub fn builder() -> FlexBoxPropsBuilder {
        FlexBoxPropsBuilder::default()
    }
}

impl FlexBoxPropsBuilder {
    pub fn border(mut self, border: BorderStyle) -> Self {
        self.props.border = border;
        self
    }
    pub fn border_color(mut self, color: Color) -> Self {
        self.props.border_color = Some(color);
        self
    }
    pub fn opt_border_color(mut self, color: Option<Color>) -> Self {
        self.props.border_color = color;
        self
    }
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.props.title = Some(title.into());
        self
    }
    pub fn opt_title(mut self, title: Option<String>) -> Self {
        self.props.title = title;
        self
    }
    pub fn flex_direction(mut self, direction: FlexDirection) -> Self {
        self.props.flex_direction = direction;
        self
    }
    pub fn gap(mut self, gap: u16) -> Self {
        self.props.gap = gap;
        self
    }
    pub fn align_items(mut self, align: Align) -> Self {
        self.props.align_items = align;
        self
    }
    pub fn justify_content(mut self, justify: Justify) -> Self {
        self.props.justify_content = justify;
        self
    }
    pub fn width(mut self, width: impl Into<Size>) -> Self {
        self.props.width = width.into();
        self
    }
    pub fn height(mut self, height: impl Into<Size>) -> Self {
        self.props.height = height.into();
        self
    }
    pub fn min_width(mut self, width: impl Into<Size>) -> Self {
        self.props.min_width = Some(width.into());
        self
    }
    pub fn opt_min_width(mut self, width: Option<Size>) -> Self {
        self.props.min_width = width;
        self
    }
    pub fn max_width(mut self, width: impl Into<Size>) -> Self {
        self.props.max_width = Some(width.into());
        self
    }
    pub fn opt_max_width(mut self, width: Option<Size>) -> Self {
        self.props.max_width = width;
        self
    }
    pub fn min_height(mut self, height: impl Into<Size>) -> Self {
        self.props.min_height = Some(height.into());
        self
    }
    pub fn opt_min_height(mut self, height: Option<Size>) -> Self {
        self.props.min_height = height;
        self
    }
    pub fn max_height(mut self, height: impl Into<Size>) -> Self {
        self.props.max_height = Some(height.into());
        self
    }
    pub fn opt_max_height(mut self, height: Option<Size>) -> Self {
        self.props.max_height = height;
        self
    }
    pub fn overflow_x(mut self, overflow: Overflow) -> Self {
        self.props.overflow_x = overflow;
        self
    }
    pub fn overflow_y(mut self, overflow: Overflow) -> Self {
        self.props.overflow_y = overflow;
        self
    }
    pub fn overflow(mut self, overflow: Overflow) -> Self {
        self.props.overflow_x = overflow;
        self.props.overflow_y = overflow;
        self
    }
    pub fn scroll_x(mut self, scroll: u16) -> Self {
        self.props.scroll_x = Some(scroll);
        self
    }
    pub fn scroll_y(mut self, scroll: u16) -> Self {
        self.props.scroll_y = Some(scroll);
        self
    }
    pub fn scrollbar_visibility(mut self, vis: ScrollbarVisibility) -> Self {
        self.props.scrollbar_visibility = vis;
        self
    }
    pub fn scrollbar_style(mut self, style: ScrollbarStyle) -> Self {
        self.props.scrollbar_style = style;
        self
    }
    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.props.padding = padding.into();
        self
    }
    pub fn bg(mut self, bg: Color) -> Self {
        self.props.bg = Some(bg);
        self
    }
    pub fn opt_bg(mut self, bg: Option<Color>) -> Self {
        self.props.bg = bg;
        self
    }
    pub fn fg(mut self, fg: Color) -> Self {
        self.props.fg = Some(fg);
        self
    }
    pub fn opt_fg(mut self, fg: Option<Color>) -> Self {
        self.props.fg = fg;
        self
    }
    pub fn color(self, color: Color) -> Self {
        self.fg(color)
    }
    pub fn opt_color(self, color: Option<Color>) -> Self {
        self.opt_fg(color)
    }
    pub fn on_click(mut self, handler: impl Fn() + 'static) -> Self {
        self.props.on_click = Some(Rc::new(handler));
        self
    }
    pub fn opt_on_click(mut self, handler: Option<Rc<dyn Fn()>>) -> Self {
        self.props.on_click = handler;
        self
    }
    pub fn on_click_at(mut self, handler: impl IntoClickHandlerAt) -> Self {
        self.props.on_click_at = Some(handler.into_click_handler_at());
        self
    }
    pub fn opt_on_click_at(mut self, handler: Option<Rc<dyn Fn(u16, u16)>>) -> Self {
        self.props.on_click_at = handler;
        self
    }
    pub fn focus_id(mut self, id: FocusId) -> Self {
        self.props.focus_id = Some(id);
        self
    }
    pub fn id(self, id: FocusId) -> Self {
        self.focus_id(id)
    }
    pub fn on_key(mut self, handler: impl IntoKeyHandler) -> Self {
        self.props.on_key = Some(handler.into_key_handler());
        self
    }
    pub fn opt_on_key(mut self, handler: Option<KeyHandler>) -> Self {
        self.props.on_key = handler;
        self
    }
    pub fn tab_stop(mut self, tab_stop: bool) -> Self {
        self.props.tab_stop = tab_stop;
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.props.disabled = disabled;
        self
    }
    pub fn activation(mut self, activation: Activation) -> Self {
        self.props.activation = activation;
        self
    }
    pub fn semantics(mut self, semantics: Semantics) -> Self {
        self.props.semantics = Some(semantics);
        self
    }
    pub fn interaction(mut self, interaction: Interaction) -> Self {
        self.props.interaction = Some(std::boxed::Box::new(interaction));
        self
    }
    pub fn on_scroll(mut self, handler: impl IntoScrollHandler) -> Self {
        self.props.on_scroll = Some(handler.into_scroll_handler());
        self
    }
    pub fn cursor_offset(mut self, offset: (u16, u16)) -> Self {
        self.props.cursor_offset = Some(offset);
        self
    }
    pub fn opt_cursor_offset(mut self, offset: Option<(u16, u16)>) -> Self {
        self.props.cursor_offset = offset;
        self
    }
    pub fn focus_scope(mut self, scope: FocusScope) -> Self {
        self.props.focus_scope = Some(scope);
        self
    }
    pub fn opt_focus_scope(mut self, scope: Option<FocusScope>) -> Self {
        self.props.focus_scope = scope;
        self
    }
    pub fn slot_children(mut self, node: impl IntoNode) -> Self {
        match node.into_node() {
            VNode::Fragment(items) => self.props.children.extend(items),
            VNode::Empty => {}
            other => self.props.children.push(other),
        }
        self
    }
    pub fn children(self, node: impl IntoNode) -> Self {
        self.slot_children(node)
    }
    pub fn build(self) -> FlexBoxProps {
        self.props
    }
}

#[allow(non_snake_case)]
pub fn FlexBox(props: FlexBoxProps) -> FlexBoxElement {
    let mut el = FlexBoxElement {
        border: props.border,
        border_color: props.border_color,
        title: props.title,
        flex_direction: props.flex_direction,
        gap: props.gap,
        align_items: props.align_items,
        justify_content: props.justify_content,
        width: props.width,
        height: props.height,
        min_width: props.min_width,
        max_width: props.max_width,
        min_height: props.min_height,
        max_height: props.max_height,
        overflow_x: props.overflow_x,
        overflow_y: props.overflow_y,
        scroll_x: props.scroll_x,
        scroll_y: props.scroll_y,
        scrollbar_visibility: props.scrollbar_visibility,
        scrollbar_style: props.scrollbar_style,
        padding: props.padding,
        bg: props.bg,
        fg: props.fg,
        on_click: props.on_click,
        on_click_at: props.on_click_at,
        focus_id: props.focus_id,
        on_key: props.on_key,
        interaction: props.interaction,
        on_scroll: props.on_scroll,
        cursor_offset: props.cursor_offset,
        focus_scope: props.focus_scope,
        children: props.children,
    };
    if props.tab_stop {
        el = el.tab_stop(true);
    }
    if props.disabled {
        el = el.disabled(true);
    }
    if props.activation != Activation::default() {
        el = el.activation(props.activation);
    }
    if let Some(sem) = props.semantics {
        el = el.semantics(sem);
    }
    if let Some(scope) = props.focus_scope {
        el = el.focus_scope(scope);
    }
    el
}

#[derive(Clone, Default)]
pub struct TextSpanProps {
    pub content: Option<String>,
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub children: Option<Snippet<()>>,
}

#[derive(Clone, Default)]
pub struct TextSpanPropsBuilder {
    props: TextSpanProps,
}

impl TextSpanProps {
    pub fn builder() -> TextSpanPropsBuilder {
        TextSpanPropsBuilder::default()
    }
}

impl TextSpanPropsBuilder {
    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.props.content = Some(content.into());
        self
    }
    pub fn color(mut self, color: Color) -> Self {
        self.props.fg = Some(color);
        self
    }
    pub fn fg(mut self, fg: Color) -> Self {
        self.props.fg = Some(fg);
        self
    }
    pub fn opt_fg(mut self, fg: Option<Color>) -> Self {
        self.props.fg = fg;
        self
    }
    pub fn opt_color(self, color: Option<Color>) -> Self {
        self.opt_fg(color)
    }
    pub fn bg(mut self, bg: Color) -> Self {
        self.props.bg = Some(bg);
        self
    }
    pub fn opt_bg(mut self, bg: Option<Color>) -> Self {
        self.props.bg = bg;
        self
    }
    pub fn bold(mut self, bold: bool) -> Self {
        self.props.bold = bold;
        self
    }
    pub fn italic(mut self, italic: bool) -> Self {
        self.props.italic = italic;
        self
    }
    pub fn underline(mut self, underline: bool) -> Self {
        self.props.underline = underline;
        self
    }
    pub fn slot_children(mut self, node: impl IntoNode) -> Self {
        self.props.children = Some(Snippet::from(node.into_node()));
        self
    }
    pub fn children(self, node: impl IntoNode) -> Self {
        self.slot_children(node)
    }
    pub fn build(self) -> TextSpanProps {
        self.props
    }
}

#[allow(non_snake_case)]
pub fn TextSpan(props: TextSpanProps) -> TextElement {
    let mut text_content = props.content.unwrap_or_default();
    if let Some(children) = props.children {
        match children.render(()) {
            VNode::Text(t) => {
                if text_content.is_empty() {
                    text_content = t.content;
                } else {
                    text_content.push_str(&t.content);
                }
            }
            VNode::Fragment(nodes) => {
                for node in nodes {
                    if let VNode::Text(t) = node {
                        text_content.push_str(&t.content);
                    }
                }
            }
            _ => {}
        }
    }
    let mut el = TextElement::new(text_content);
    if let Some(fg) = props.fg {
        el = el.fg(fg);
    }
    if let Some(bg) = props.bg {
        el = el.bg(bg);
    }
    if props.bold {
        el = el.bold(true);
    }
    if props.italic {
        el = el.italic(true);
    }
    if props.underline {
        el = el.underline(true);
    }
    el
}

#[derive(Clone, Default)]
pub struct ParagraphProps {
    pub content: Option<String>,
    /// Limit wrapped lines, adding `…` when content remains. Zero hides all text.
    /// Omit this property to show every line.
    pub max_lines: Option<usize>,
    pub fill: Option<char>,
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub children: Option<Snippet<()>>,
}

#[derive(Clone, Default)]
pub struct ParagraphPropsBuilder {
    props: ParagraphProps,
}

impl ParagraphProps {
    pub fn builder() -> ParagraphPropsBuilder {
        ParagraphPropsBuilder::default()
    }
}

impl ParagraphPropsBuilder {
    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.props.content = Some(content.into());
        self
    }
    pub fn max_lines(mut self, max_lines: usize) -> Self {
        self.props.max_lines = Some(max_lines);
        self
    }
    pub fn fill(mut self, fill: char) -> Self {
        self.props.fill = Some(fill);
        self
    }
    pub fn color(mut self, color: Color) -> Self {
        self.props.fg = Some(color);
        self
    }
    pub fn fg(mut self, fg: Color) -> Self {
        self.props.fg = Some(fg);
        self
    }
    pub fn opt_fg(mut self, fg: Option<Color>) -> Self {
        self.props.fg = fg;
        self
    }
    pub fn opt_color(self, color: Option<Color>) -> Self {
        self.opt_fg(color)
    }
    pub fn bg(mut self, bg: Color) -> Self {
        self.props.bg = Some(bg);
        self
    }
    pub fn opt_bg(mut self, bg: Option<Color>) -> Self {
        self.props.bg = bg;
        self
    }
    pub fn bold(mut self, bold: bool) -> Self {
        self.props.bold = bold;
        self
    }
    pub fn italic(mut self, italic: bool) -> Self {
        self.props.italic = italic;
        self
    }
    pub fn underline(mut self, underline: bool) -> Self {
        self.props.underline = underline;
        self
    }
    pub fn slot_children(mut self, node: impl IntoNode) -> Self {
        self.props.children = Some(Snippet::from(node.into_node()));
        self
    }
    pub fn children(self, node: impl IntoNode) -> Self {
        self.slot_children(node)
    }
    pub fn build(self) -> ParagraphProps {
        self.props
    }
}

#[allow(non_snake_case)]
pub fn Paragraph(props: ParagraphProps) -> TextElement {
    let mut text_content = props.content.unwrap_or_default();
    if let Some(children) = props.children {
        match children.render(()) {
            VNode::Text(t) => {
                if text_content.is_empty() {
                    text_content = t.content;
                } else {
                    text_content.push_str(&t.content);
                }
            }
            VNode::Fragment(nodes) => {
                for node in nodes {
                    if let VNode::Text(t) = node {
                        text_content.push_str(&t.content);
                    }
                }
            }
            _ => {}
        }
    }
    // Filled text is used for single-line rules such as Divider labels.
    let mut el = TextElement::new(text_content).wrap(props.fill.is_none());
    if let Some(max_lines) = props.max_lines {
        el = el.max_lines(max_lines);
    }
    if let Some(fill) = props.fill {
        el = el.fill(fill);
    }
    if let Some(fg) = props.fg {
        el = el.fg(fg);
    }
    if let Some(bg) = props.bg {
        el = el.bg(bg);
    }
    if props.bold {
        el = el.bold(true);
    }
    if props.italic {
        el = el.italic(true);
    }
    if props.underline {
        el = el.underline(true);
    }
    el
}

#[derive(Clone, Default)]
pub struct OverlayProps {
    pub id: Option<FocusId>,
    pub opener: Option<FocusId>,
    /// Layout identity used for positioning, independently of focus restoration.
    pub anchor: Option<FocusId>,
    pub layer: Option<DismissLayer>,
    pub focus_group: Option<FocusGroup>,
    pub managed: bool,
    pub semantics: Option<Semantics>,
    pub open: bool,
    pub placement: Placement,
    pub backdrop: bool,
    pub modal: bool,
    pub on_key: Option<KeyHandler>,
    pub on_close: Option<Rc<dyn Fn()>>,
    pub on_open: Option<Rc<dyn Fn()>>,
    pub trigger: Option<VNode>,
    pub slot_trigger: Option<Snippet<()>>,
    pub children: Option<Snippet<()>>,
}

#[derive(Clone, Default)]
pub struct OverlayPropsBuilder {
    props: OverlayProps,
}

impl OverlayProps {
    pub fn builder() -> OverlayPropsBuilder {
        OverlayPropsBuilder::default()
    }
}

impl OverlayPropsBuilder {
    pub fn id(mut self, id: FocusId) -> Self {
        self.props.id = Some(id);
        self
    }
    pub fn opener(mut self, opener: FocusId) -> Self {
        self.props.opener = Some(opener);
        self
    }
    pub fn opt_opener(mut self, opener: Option<FocusId>) -> Self {
        self.props.opener = opener;
        self
    }
    pub fn anchor(mut self, id: FocusId) -> Self {
        self.props.anchor = Some(id);
        self
    }
    pub fn layer(mut self, layer: DismissLayer) -> Self {
        self.props.layer = Some(layer);
        self
    }
    pub fn focus_group(mut self, group: FocusGroup) -> Self {
        self.props.focus_group = Some(group);
        self
    }
    pub fn managed(mut self, managed: bool) -> Self {
        self.props.managed = managed;
        self
    }
    pub fn semantics(mut self, semantics: Semantics) -> Self {
        self.props.semantics = Some(semantics);
        self
    }
    pub fn open(mut self, open: bool) -> Self {
        self.props.open = open;
        self
    }
    pub fn placement(mut self, placement: Placement) -> Self {
        self.props.placement = placement;
        self
    }
    pub fn backdrop(mut self, backdrop: bool) -> Self {
        self.props.backdrop = backdrop;
        self
    }
    pub fn modal(mut self, modal: bool) -> Self {
        self.props.modal = modal;
        self
    }
    pub fn on_key(
        mut self,
        handler: impl Fn(KeyEvent, &mut EventContext) -> EventResult + 'static,
    ) -> Self {
        self.props.on_key = Some(Rc::new(handler));
        self
    }
    pub fn opt_on_key(mut self, handler: Option<KeyHandler>) -> Self {
        self.props.on_key = handler;
        self
    }
    pub fn on_close(mut self, handler: impl Fn() + 'static) -> Self {
        self.props.on_close = Some(Rc::new(handler));
        self
    }
    pub fn opt_on_close(mut self, handler: Option<Rc<dyn Fn()>>) -> Self {
        self.props.on_close = handler;
        self
    }
    pub fn on_open(mut self, handler: impl Fn() + 'static) -> Self {
        self.props.on_open = Some(Rc::new(handler));
        self
    }
    pub fn opt_on_open(mut self, handler: Option<Rc<dyn Fn()>>) -> Self {
        self.props.on_open = handler;
        self
    }
    pub fn trigger(mut self, trigger: impl IntoNode) -> Self {
        self.props.trigger = Some(trigger.into_node());
        self
    }
    pub fn opt_trigger(mut self, trigger: Option<VNode>) -> Self {
        self.props.trigger = trigger;
        self
    }
    pub fn slot_trigger(mut self, trigger: impl IntoNode) -> Self {
        self.props.slot_trigger = Some(Snippet::from(trigger.into_node()));
        self
    }
    pub fn opt_slot_trigger(mut self, trigger: Option<Snippet<()>>) -> Self {
        self.props.slot_trigger = trigger;
        self
    }
    pub fn slot_children(mut self, node: impl IntoNode) -> Self {
        self.props.children = Some(Snippet::from(node.into_node()));
        self
    }
    pub fn children(self, node: impl IntoNode) -> Self {
        self.slot_children(node)
    }
    pub fn build(self) -> OverlayProps {
        self.props
    }
}

#[allow(non_snake_case)]
pub fn Overlay(props: OverlayProps) -> OverlayElement {
    let mut el = OverlayElement::new();
    if let Some(id) = props.id {
        el = el.id(id);
    }
    if let Some(opener) = props.opener {
        el = el.opener(opener);
    }
    if let Some(anchor) = props.anchor {
        el = el.anchor(anchor);
    }
    if let Some(layer) = props.layer {
        el = el.layer(layer);
    }
    if let Some(fg) = props.focus_group {
        el = el.focus_group(fg);
    }
    if props.managed {
        el = el.managed(true);
    }
    if let Some(sem) = props.semantics {
        el = el.semantics(sem);
    }
    el = el.open(props.open);
    el = el.placement(props.placement);
    el = el.backdrop(props.backdrop);
    if props.modal {
        el = el.modal(true);
    }
    if let Some(ok) = props.on_key {
        el = el.opt_on_key(Some(ok));
    }
    if let Some(oc) = props.on_close {
        el = el.opt_on_close(Some(oc));
    }
    if let Some(oo) = props.on_open {
        el = el.opt_on_open(Some(oo));
    }
    if let Some(trigger) = props.trigger {
        el = el.trigger(trigger);
    } else if let Some(slot_trigger) = props.slot_trigger {
        el = el.trigger(slot_trigger.render(()));
    }
    if let Some(children) = props.children {
        el = el.child(children.render(()));
    }
    el
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
