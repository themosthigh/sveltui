use crossterm::event::KeyEvent;
use crossterm::style::Color;
use std::cell::{Cell as StdCell, RefCell};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::rc::Rc;
use svtui_layout::{Align, BorderStyle, Justify, Padding, Rect, Size};
use svtui_terminal::Buffer;

thread_local! {
    static CURRENT_FOCUS: StdCell<Option<FocusId>> = const { StdCell::new(None) };
    static FOCUS_WITHIN_SET: RefCell<HashSet<FocusId>> = RefCell::new(HashSet::new());
    static RENDER_FOCUS_COUNTER: StdCell<usize> = const { StdCell::new(0) };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FocusId(pub usize);

impl FocusId {
    pub fn new() -> Self {
        next_render_focus_id()
    }
}

impl Default for FocusId {
    fn default() -> Self {
        Self::new()
    }
}

pub fn reset_focus_counter() {
    RENDER_FOCUS_COUNTER.with(|c| c.set(0));
}

pub fn next_render_focus_id() -> FocusId {
    RENDER_FOCUS_COUNTER.with(|c| {
        let val = c.get();
        c.set(val + 1);
        FocusId(val)
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct FocusState {
    pub is_focused: bool,
    pub has_focus_within: bool,
}

impl FocusState {
    #[inline]
    pub fn is_focused(&self) -> bool {
        self.is_focused
    }

    #[inline]
    pub fn has_focus_within(&self) -> bool {
        self.has_focus_within
    }

    #[inline]
    pub fn is_active(&self) -> bool {
        self.is_focused || self.has_focus_within
    }
}

/// Completely opt-in hook to query focus state (both `:focus` and `:focus-within`) for any component or element.
pub fn use_focus(id: FocusId) -> FocusState {
    let is_focused = CURRENT_FOCUS.with(|f| f.get() == Some(id));
    let has_focus_within = FOCUS_WITHIN_SET.with(|s| s.borrow().contains(&id));
    FocusState {
        is_focused,
        has_focus_within,
    }
}

/// Reusable handle to manage and query focus state on custom components.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FocusHandle {
    id: FocusId,
}

impl FocusHandle {
    pub fn new() -> Self {
        Self { id: FocusId::new() }
    }

    pub fn from_id(id: FocusId) -> Self {
        Self { id }
    }

    pub fn id(&self) -> FocusId {
        self.id
    }

    pub fn state(&self) -> FocusState {
        use_focus(self.id)
    }

    pub fn is_focused(&self) -> bool {
        use_focus(self.id).is_focused()
    }

    pub fn has_focus_within(&self) -> bool {
        use_focus(self.id).has_focus_within()
    }

    pub fn is_active(&self) -> bool {
        use_focus(self.id).is_active()
    }
}

impl Default for FocusHandle {
    fn default() -> Self {
        Self::new()
    }
}

pub fn is_focused(id: FocusId) -> bool {
    CURRENT_FOCUS.with(|f| f.get() == Some(id))
}

pub fn get_focused() -> Option<FocusId> {
    CURRENT_FOCUS.with(|f| f.get())
}

pub fn set_focused(id: Option<FocusId>) {
    CURRENT_FOCUS.with(|f| f.set(id));
}

pub fn update_focus_context(focused_id: Option<FocusId>, hit_map: &HitMap) {
    CURRENT_FOCUS.with(|f| f.set(focused_id));
    FOCUS_WITHIN_SET.with(|s| {
        let mut set = s.borrow_mut();
        set.clear();
        if let Some(ancestors) = focused_id.and_then(|active| hit_map.ancestors.get(&active)) {
            for &anc in ancestors {
                set.insert(anc);
            }
        }
    });
}

impl From<usize> for FocusId {
    fn from(val: usize) -> Self {
        FocusId(val)
    }
}

#[derive(Clone)]
pub struct HitTarget {
    pub rect: Rect,
    pub on_click: Option<Rc<dyn Fn()>>,
    pub focus_id: Option<FocusId>,
    pub on_key: Option<Rc<dyn Fn(KeyEvent)>>,
}

#[derive(Default, Clone)]
pub struct HitMap {
    pub targets: Vec<HitTarget>,
    pub focus_stack: Vec<FocusId>,
    pub ancestors: HashMap<FocusId, HashSet<FocusId>>,
}

impl HitMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.targets.clear();
        self.focus_stack.clear();
        self.ancestors.clear();
    }

    pub fn register(&mut self, target: HitTarget) {
        self.targets.push(target);
    }

    /// Finds the innermost matching HitTarget containing (x, y).
    pub fn hit_test(&self, x: u16, y: u16) -> Option<&HitTarget> {
        self.targets
            .iter()
            .rev()
            .find(|target| target.rect.contains(x, y))
    }

    /// Returns all registered focusable IDs in DOM order.
    pub fn focusable_ids(&self) -> Vec<FocusId> {
        let mut ids = Vec::new();
        for target in &self.targets {
            if let Some(id) = target.focus_id.filter(|id| !ids.contains(id)) {
                ids.push(id);
            }
        }
        ids
    }
}

#[derive(Clone, Default)]
pub struct BoxElement {
    pub border: BorderStyle,
    pub border_color: Option<Color>,
    pub title: Option<String>,
    pub align_items: Align,
    pub justify_content: Justify,
    pub width: Size,
    pub height: Size,
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

pub fn measure_node(node: &VNode, max_w: u16, max_h: u16) -> (u16, u16) {
    match node {
        VNode::Empty => (0, 0),
        VNode::Text(text) => {
            let lines: Vec<&str> = text.content.lines().collect();
            let height = (lines.len().max(1) as u16).min(max_h);
            let width = lines
                .iter()
                .map(|line| line.chars().count() as u16)
                .max()
                .unwrap_or(0)
                .min(max_w);
            (width, height)
        }
        VNode::Fragment(children) => {
            let mut total_w: u16 = 0;
            let mut total_h: u16 = 0;
            for child in children {
                let (cw, ch) = measure_node(child, max_w, max_h);
                total_w = total_w.max(cw);
                total_h = total_h.saturating_add(ch);
            }
            (total_w.min(max_w), total_h.min(max_h))
        }
        VNode::Box(b) => {
            let border_w = if b.border != BorderStyle::None { 2 } else { 0 };
            let border_h = if b.border != BorderStyle::None { 2 } else { 0 };

            let pad_w = b.padding.left + b.padding.right;
            let pad_h = b.padding.top + b.padding.bottom;

            let extra_w = border_w + pad_w;
            let extra_h = border_h + pad_h;

            let avail_w = max_w.saturating_sub(extra_w);
            let avail_h = max_h.saturating_sub(extra_h);

            let mut content_w: u16 = 0;
            let mut content_h: u16 = 0;
            for child in &b.children {
                let (cw, ch) = measure_node(child, avail_w, avail_h);
                content_w = content_w.max(cw);
                content_h = content_h.saturating_add(ch);
            }

            if let Some(ref title) = b.title {
                let title_w = title.chars().count() as u16 + 4;
                content_w = content_w.max(title_w);
            }

            let measured_w = match b.width {
                Size::Fixed(w) => w,
                Size::Percent(p) => ((max_w as f32 * p / 100.0).round() as u16).min(max_w),
                Size::Fill => max_w,
                Size::Auto => content_w.saturating_add(extra_w),
            };

            let measured_h = match b.height {
                Size::Fixed(h) => h,
                Size::Percent(p) => ((max_h as f32 * p / 100.0).round() as u16).min(max_h),
                Size::Fill => max_h,
                Size::Auto => content_h.saturating_add(extra_h),
            };

            (measured_w.min(max_w), measured_h.min(max_h))
        }
    }
}

pub fn render_node(node: &VNode, rect: Rect, buffer: &mut Buffer) {
    let mut hit_map = HitMap::new();
    render_node_interactive(node, rect, buffer, &mut hit_map);
}

pub fn render_node_interactive(
    node: &VNode,
    rect: Rect,
    buffer: &mut Buffer,
    hit_map: &mut HitMap,
) {
    if rect.width == 0 || rect.height == 0 {
        return;
    }

    match node {
        VNode::Empty => {}
        VNode::Text(text) => {
            let lines: Vec<&str> = text.content.lines().collect();
            for (i, line) in lines.iter().enumerate() {
                let y = rect.y.saturating_add(i as u16);
                if y >= rect.y.saturating_add(rect.height) {
                    break;
                }
                buffer.set_string(rect.x, y, line, text.fg, text.bg, text.bold, rect.width);
            }
        }
        VNode::Fragment(children) => {
            let mut curr_y = rect.y;
            for child in children {
                let (_, ch) = measure_node(child, rect.width, rect.height);
                let child_rect = Rect::new(rect.x, curr_y, rect.width, ch.min(rect.height));
                render_node_interactive(child, child_rect, buffer, hit_map);
                curr_y = curr_y.saturating_add(ch);
                if curr_y >= rect.y + rect.height {
                    break;
                }
            }
        }
        VNode::Box(b) => {
            // Track focus ancestry hierarchy
            if let Some(id) = b.focus_id {
                for &anc in &hit_map.focus_stack {
                    hit_map.ancestors.entry(id).or_default().insert(anc);
                }
                hit_map.focus_stack.push(id);
            }

            // Register interaction if clickable or focusable or has key listener
            if b.on_click.is_some() || b.focus_id.is_some() || b.on_key.is_some() {
                hit_map.register(HitTarget {
                    rect,
                    on_click: b.on_click.clone(),
                    focus_id: b.focus_id,
                    on_key: b.on_key.clone(),
                });
            }

            // Fill background if specified
            if let Some(bg) = b.bg {
                for y in rect.y..(rect.y + rect.height) {
                    for x in rect.x..(rect.x + rect.width) {
                        buffer.set_char(x, y, ' ', b.fg, Some(bg), false);
                    }
                }
            }

            let border_w = if b.border != BorderStyle::None { 1 } else { 0 };
            let border_h = if b.border != BorderStyle::None { 1 } else { 0 };

            if let Some(chars) = b.border.chars() {
                buffer.draw_border(rect, chars, b.border_color);
            }

            if let Some(ref title) = b.title {
                buffer.draw_title(rect, title, b.border_color);
            }

            let inner = rect.inner(
                border_h + b.padding.top,
                border_w + b.padding.right,
                border_h + b.padding.bottom,
                border_w + b.padding.left,
            );

            if inner.width > 0 && inner.height > 0 && !b.children.is_empty() {
                let measured_children: Vec<(u16, u16)> = b
                    .children
                    .iter()
                    .map(|c| measure_node(c, inner.width, inner.height))
                    .collect();

                let total_children_h: u16 = measured_children.iter().map(|(_, h)| *h).sum();

                let start_y = match b.align_items {
                    Align::Center => inner.y + inner.height.saturating_sub(total_children_h) / 2,
                    Align::End => inner.y + inner.height.saturating_sub(total_children_h),
                    _ => inner.y,
                };

                let mut curr_y = start_y;
                for (child, &(cw, ch)) in b.children.iter().zip(measured_children.iter()) {
                    let start_x = match b.justify_content {
                        Justify::Center => inner.x + inner.width.saturating_sub(cw) / 2,
                        Justify::End => inner.x + inner.width.saturating_sub(cw),
                        _ => inner.x,
                    };

                    let child_rect =
                        Rect::new(start_x, curr_y, cw.min(inner.width), ch.min(inner.height));
                    render_node_interactive(child, child_rect, buffer, hit_map);

                    curr_y = curr_y.saturating_add(ch);
                    if curr_y >= inner.y + inner.height {
                        break;
                    }
                }
            }

            // Pop from focus ancestry stack
            if b.focus_id.is_some() {
                hit_map.focus_stack.pop();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_measure_text() {
        let text = TextElement::new("Hello, world!").into_node();
        let (w, h) = measure_node(&text, 80, 24);
        assert_eq!(w, 13);
        assert_eq!(h, 1);
    }

    #[test]
    fn test_hit_map() {
        let mut hit_map = HitMap::new();
        hit_map.register(HitTarget {
            rect: Rect::new(5, 5, 10, 3),
            on_click: Some(Rc::new(|| {})),
            focus_id: Some(FocusId(1)),
            on_key: None,
        });

        assert!(hit_map.hit_test(6, 6).is_some());
        assert!(hit_map.hit_test(0, 0).is_none());
        assert_eq!(hit_map.focusable_ids(), vec![FocusId(1)]);
    }

    #[test]
    fn test_stable_focus_ids_across_renders() {
        reset_focus_counter();
        let id1 = FocusId::new();
        let id2 = FocusId::new();
        assert_eq!(id1, FocusId(0));
        assert_eq!(id2, FocusId(1));

        set_focused(Some(id2));

        reset_focus_counter();
        let frame2_id1 = FocusId::new();
        let frame2_id2 = FocusId::new();
        assert_eq!(frame2_id1, FocusId(0));
        assert_eq!(frame2_id2, FocusId(1));

        assert!(!is_focused(frame2_id1));
        assert!(is_focused(frame2_id2));
    }

    #[test]
    fn test_use_focus_and_focus_within() {
        let mut buffer = Buffer::new(40, 10);
        let mut hit_map = HitMap::new();

        let parent_id = FocusId(100);
        let child_id = FocusId(101);

        let tree = BoxElement::new()
            .focus_id(parent_id)
            .child(
                BoxElement::new()
                    .focus_id(child_id)
                    .child(TextElement::new("Child button")),
            )
            .into_node();

        // Render to establish DOM ancestry
        render_node_interactive(&tree, Rect::new(0, 0, 40, 10), &mut buffer, &mut hit_map);

        // When child is focused:
        update_focus_context(Some(child_id), &hit_map);

        let child_state = use_focus(child_id);
        assert!(child_state.is_focused());
        assert!(!child_state.has_focus_within());
        assert!(child_state.is_active());

        // Parent should contain focus (:focus-within)
        let parent_state = use_focus(parent_id);
        assert!(!parent_state.is_focused());
        assert!(parent_state.has_focus_within());
        assert!(parent_state.is_active());

        // When parent is focused directly:
        update_focus_context(Some(parent_id), &hit_map);
        let parent_state_direct = use_focus(parent_id);
        assert!(parent_state_direct.is_focused());
        assert!(!parent_state_direct.has_focus_within());

        // When nothing is focused:
        update_focus_context(None, &hit_map);
        assert!(!use_focus(parent_id).is_active());
        assert!(!use_focus(child_id).is_active());
    }
}
