use crossterm::event::KeyEvent;
use crossterm::style::Color;
use std::cell::{Cell as StdCell, RefCell};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::rc::Rc;
use svtui_layout::{
    Align, BorderStyle, Justify, Overflow, Padding, Rect, ScrollbarStyle, ScrollbarVisibility,
    Size, clamp_dimension,
};
use svtui_terminal::Buffer;

thread_local! {
    static CURRENT_FOCUS: StdCell<Option<FocusId>> = const { StdCell::new(None) };
    static FOCUS_WITHIN_SET: RefCell<HashSet<FocusId>> = RefCell::new(HashSet::new());
    static RENDER_FOCUS_COUNTER: StdCell<usize> = const { StdCell::new(0) };
    static SCROLL_MAP: RefCell<HashMap<FocusId, (u16, u16)>> = RefCell::new(HashMap::new());
}

pub fn get_scroll(id: FocusId) -> (u16, u16) {
    SCROLL_MAP.with(|m| *m.borrow().get(&id).unwrap_or(&(0, 0)))
}

pub fn set_scroll(id: FocusId, pos: (u16, u16)) {
    SCROLL_MAP.with(|m| {
        m.borrow_mut().insert(id, pos);
    });
}

use std::sync::atomic::{AtomicUsize, Ordering};

static USER_FOCUS_COUNTER: AtomicUsize = AtomicUsize::new(100_000);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FocusId(pub usize);

impl FocusId {
    /// Allocates a globally unique FocusId for application-level state handles.
    pub fn new() -> Self {
        FocusId(USER_FOCUS_COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for FocusId {
    fn default() -> Self {
        next_render_focus_id()
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
    pub on_scroll: Option<Rc<dyn Fn(i16, i16)>>,
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

    /// Finds the innermost matching HitTarget containing (x, y) that supports scrolling.
    pub fn scroll_target(&self, x: u16, y: u16) -> Option<&HitTarget> {
        self.targets
            .iter()
            .rev()
            .find(|target| target.rect.contains(x, y) && target.on_scroll.is_some())
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

            let avail_w = if b.overflow_x == Overflow::Scroll || b.overflow_x == Overflow::Auto {
                u16::MAX / 2
            } else {
                max_w.saturating_sub(extra_w)
            };

            let avail_h = if b.overflow_y == Overflow::Scroll || b.overflow_y == Overflow::Auto {
                u16::MAX / 2
            } else {
                max_h.saturating_sub(extra_h)
            };

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
            let clamped_w = clamp_dimension(measured_w, b.min_width, b.max_width, max_w);

            let measured_h = match b.height {
                Size::Fixed(h) => h,
                Size::Percent(p) => ((max_h as f32 * p / 100.0).round() as u16).min(max_h),
                Size::Fill => max_h,
                Size::Auto => content_h.saturating_add(extra_h),
            };
            let clamped_h = clamp_dimension(measured_h, b.min_height, b.max_height, max_h);

            (clamped_w.min(max_w), clamped_h.min(max_h))
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
    render_node_offset(
        node,
        rect.x as i32,
        rect.y as i32,
        rect.width,
        rect.height,
        buffer,
        hit_map,
    );
}

fn render_node_offset(
    node: &VNode,
    x: i32,
    y: i32,
    width: u16,
    height: u16,
    buffer: &mut Buffer,
    hit_map: &mut HitMap,
) {
    if width == 0 || height == 0 {
        return;
    }
    if x >= buffer.width as i32 || y >= buffer.height as i32 {
        return;
    }
    if x + width as i32 <= 0 || y + height as i32 <= 0 {
        return;
    }

    match node {
        VNode::Empty => {}
        VNode::Text(text) => {
            let lines: Vec<&str> = text.content.lines().collect();
            for (i, line) in lines.iter().enumerate() {
                let curr_y = y + i as i32;
                if curr_y < 0 || curr_y >= (y + height as i32) || curr_y >= buffer.height as i32 {
                    continue;
                }
                let clip = buffer.active_clip();
                if curr_y < clip.y as i32 || curr_y >= (clip.y as i32 + clip.height as i32) {
                    continue;
                }
                for (char_idx, ch) in line.chars().enumerate() {
                    let curr_x = x + char_idx as i32;
                    if curr_x >= x + width as i32 || curr_x >= buffer.width as i32 {
                        break;
                    }
                    if curr_x >= 0 {
                        buffer.set_char(
                            curr_x as u16,
                            curr_y as u16,
                            ch,
                            text.fg,
                            text.bg,
                            text.bold,
                        );
                    }
                }
            }
        }
        VNode::Fragment(children) => {
            let mut curr_y = y;
            for child in children {
                let (_, ch) = measure_node(child, width, height);
                render_node_offset(child, x, curr_y, width, ch.min(height), buffer, hit_map);
                curr_y = curr_y.saturating_add(ch as i32);
                if curr_y >= y + height as i32 {
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

            // Fill background if specified
            if let Some(bg) = b.bg {
                for curr_y in y..(y + height as i32) {
                    for curr_x in x..(x + width as i32) {
                        if curr_x >= 0 && curr_y >= 0 {
                            buffer.set_char(
                                curr_x as u16,
                                curr_y as u16,
                                ' ',
                                b.fg,
                                Some(bg),
                                false,
                            );
                        }
                    }
                }
            }

            let border_w: u16 = if b.border != BorderStyle::None { 1 } else { 0 };
            let border_h: u16 = if b.border != BorderStyle::None { 1 } else { 0 };

            if let Some(chars) = b.border.chars().filter(|_| width >= 2 && height >= 2) {
                let right = x + width as i32 - 1;
                let bottom = y + height as i32 - 1;

                let mut draw_ch = |cx: i32, cy: i32, ch: char| {
                    if cx >= 0 && cy >= 0 {
                        buffer.set_char(cx as u16, cy as u16, ch, b.border_color, None, false);
                    }
                };

                draw_ch(x, y, chars.top_left);
                draw_ch(right, y, chars.top_right);
                draw_ch(x, bottom, chars.bottom_left);
                draw_ch(right, bottom, chars.bottom_right);

                for cx in (x + 1)..right {
                    draw_ch(cx, y, chars.horizontal);
                    draw_ch(cx, bottom, chars.horizontal);
                }

                for cy in (y + 1)..bottom {
                    draw_ch(x, cy, chars.vertical);
                    draw_ch(right, cy, chars.vertical);
                }
            }

            if let Some(title) = b.title.as_deref().filter(|t| width > 4 && !t.is_empty()) {
                let title_text = format!(" {} ", title);
                let max_len = (width.saturating_sub(4)) as usize;
                let displayed_title: String = title_text.chars().take(max_len).collect();
                let start_x = x + 2;
                for (idx, ch) in displayed_title.chars().enumerate() {
                    let cx = start_x + idx as i32;
                    if cx >= 0 && y >= 0 {
                        buffer.set_char(cx as u16, y as u16, ch, b.border_color, None, true);
                    }
                }
            }

            let inner_x = x + border_w as i32 + b.padding.left as i32;
            let inner_y = y + border_h as i32 + b.padding.top as i32;
            let inner_w = width.saturating_sub(border_w * 2 + b.padding.left + b.padding.right);
            let inner_h = height.saturating_sub(border_h * 2 + b.padding.top + b.padding.bottom);

            if inner_w > 0 && inner_h > 0 {
                let is_scroll_x =
                    b.overflow_x == Overflow::Scroll || b.overflow_x == Overflow::Auto;
                let is_scroll_y =
                    b.overflow_y == Overflow::Scroll || b.overflow_y == Overflow::Auto;

                let avail_w = if is_scroll_x { u16::MAX / 2 } else { inner_w };
                let avail_h = if is_scroll_y { u16::MAX / 2 } else { inner_h };

                let measured_children: Vec<(u16, u16)> = b
                    .children
                    .iter()
                    .map(|c| measure_node(c, avail_w, avail_h))
                    .collect();

                let total_children_w: u16 =
                    measured_children.iter().map(|(w, _)| *w).max().unwrap_or(0);
                let total_children_h: u16 = measured_children.iter().map(|(_, h)| *h).sum();

                let show_v_scrollbar = match b.scrollbar_visibility {
                    ScrollbarVisibility::Always => is_scroll_y,
                    ScrollbarVisibility::Auto => is_scroll_y && total_children_h > inner_h,
                    ScrollbarVisibility::Never => false,
                };

                let show_h_scrollbar = match b.scrollbar_visibility {
                    ScrollbarVisibility::Always => is_scroll_x,
                    ScrollbarVisibility::Auto => is_scroll_x && total_children_w > inner_w,
                    ScrollbarVisibility::Never => false,
                };

                let mut viewport_w = inner_w;
                let mut viewport_h = inner_h;
                if show_v_scrollbar {
                    viewport_w = viewport_w.saturating_sub(1);
                }
                if show_h_scrollbar {
                    viewport_h = viewport_h.saturating_sub(1);
                }

                let is_scrollable = is_scroll_x || is_scroll_y;
                let max_scroll_x = if is_scroll_x {
                    total_children_w.saturating_sub(viewport_w)
                } else {
                    0
                };
                let max_scroll_y = if is_scroll_y {
                    total_children_h.saturating_sub(viewport_h)
                } else {
                    0
                };

                // Resolve current scroll offsets (ensure stable ID for any scrollable container)
                let (scroll_x, scroll_y, scroll_fid) = if is_scrollable {
                    let fid = b.focus_id.unwrap_or_else(next_render_focus_id);
                    let (mut sx, mut sy) = get_scroll(fid);

                    if let Some(x) = b.scroll_x {
                        sx = x;
                    }
                    if let Some(y) = b.scroll_y {
                        sy = y;
                    }

                    sx = sx.min(max_scroll_x);
                    sy = sy.min(max_scroll_y);
                    set_scroll(fid, (sx, sy));
                    (sx, sy, Some(fid))
                } else {
                    (0, 0, b.focus_id)
                };

                // Draw vertical scrollbar
                if show_v_scrollbar && inner_h > 0 && inner_w > 0 {
                    let sb_x = inner_x + inner_w as i32 - 1;
                    let track_h = viewport_h;
                    let thumb_len = if total_children_h > 0 {
                        ((track_h as f32 / total_children_h as f32) * track_h as f32)
                            .round()
                            .clamp(1.0, track_h as f32) as u16
                    } else {
                        track_h
                    };

                    let thumb_travel = track_h.saturating_sub(thumb_len);
                    let thumb_start_y = if max_scroll_y > 0 {
                        inner_y
                            + ((scroll_y as f32 / max_scroll_y as f32) * thumb_travel as f32)
                                .round() as i32
                    } else {
                        inner_y
                    };

                    if sb_x >= 0 {
                        for row in 0..track_h {
                            let cy = inner_y + row as i32;
                            if cy >= 0 {
                                if cy >= thumb_start_y && cy < thumb_start_y + thumb_len as i32 {
                                    buffer.set_char(
                                        sb_x as u16,
                                        cy as u16,
                                        b.scrollbar_style.chars.vertical_thumb,
                                        b.scrollbar_style.thumb_color,
                                        None,
                                        true,
                                    );
                                } else {
                                    buffer.set_char(
                                        sb_x as u16,
                                        cy as u16,
                                        b.scrollbar_style.chars.vertical_track,
                                        b.scrollbar_style.track_color,
                                        None,
                                        false,
                                    );
                                }
                            }
                        }
                    }
                }

                // Draw horizontal scrollbar
                if show_h_scrollbar && inner_h > 0 && inner_w > 0 {
                    let sb_y = inner_y + inner_h as i32 - 1;
                    let track_w = viewport_w;
                    let thumb_len = if total_children_w > 0 {
                        ((track_w as f32 / total_children_w as f32) * track_w as f32)
                            .round()
                            .clamp(1.0, track_w as f32) as u16
                    } else {
                        track_w
                    };

                    let thumb_travel = track_w.saturating_sub(thumb_len);
                    let thumb_start_x = if max_scroll_x > 0 {
                        inner_x
                            + ((scroll_x as f32 / max_scroll_x as f32) * thumb_travel as f32)
                                .round() as i32
                    } else {
                        inner_x
                    };

                    if sb_y >= 0 {
                        for col in 0..track_w {
                            let cx = inner_x + col as i32;
                            if cx >= 0 {
                                if cx >= thumb_start_x && cx < thumb_start_x + thumb_len as i32 {
                                    buffer.set_char(
                                        cx as u16,
                                        sb_y as u16,
                                        b.scrollbar_style.chars.horizontal_thumb,
                                        b.scrollbar_style.thumb_color,
                                        None,
                                        true,
                                    );
                                } else {
                                    buffer.set_char(
                                        cx as u16,
                                        sb_y as u16,
                                        b.scrollbar_style.chars.horizontal_track,
                                        b.scrollbar_style.track_color,
                                        None,
                                        false,
                                    );
                                }
                            }
                        }
                    }
                }

                // Draw corner intersection if both scrollbars are active
                if show_v_scrollbar && show_h_scrollbar && inner_h > 0 && inner_w > 0 {
                    let corner_x = inner_x + inner_w as i32 - 1;
                    let corner_y = inner_y + inner_h as i32 - 1;
                    if corner_x >= 0 && corner_y >= 0 {
                        buffer.set_char(
                            corner_x as u16,
                            corner_y as u16,
                            b.scrollbar_style.chars.corner,
                            b.scrollbar_style.track_color,
                            None,
                            false,
                        );
                    }
                }

                // Register interaction
                let on_scroll_handler: Option<Rc<dyn Fn(i16, i16)>> = if is_scrollable {
                    scroll_fid.map(|fid| {
                        Rc::new(move |dx: i16, dy: i16| {
                            let (cur_x, cur_y) = get_scroll(fid);
                            let (actual_dx, actual_dy) =
                                if max_scroll_y == 0 && max_scroll_x > 0 && dx == 0 && dy != 0 {
                                    (dy * 2, 0)
                                } else {
                                    (dx, dy)
                                };
                            let new_x = if actual_dx < 0 {
                                cur_x.saturating_sub((-actual_dx) as u16)
                            } else {
                                cur_x.saturating_add(actual_dx as u16).min(max_scroll_x)
                            };
                            let new_y = if actual_dy < 0 {
                                cur_y.saturating_sub((-actual_dy) as u16)
                            } else {
                                cur_y.saturating_add(actual_dy as u16).min(max_scroll_y)
                            };
                            set_scroll(fid, (new_x, new_y));
                        }) as Rc<dyn Fn(i16, i16)>
                    })
                } else {
                    None
                };

                if b.on_click.is_some()
                    || b.focus_id.is_some()
                    || b.on_key.is_some()
                    || on_scroll_handler.is_some()
                {
                    let target_rect = Rect::new(x.max(0) as u16, y.max(0) as u16, width, height);
                    hit_map.register(HitTarget {
                        rect: target_rect,
                        on_click: b.on_click.clone(),
                        focus_id: scroll_fid,
                        on_key: b.on_key.clone(),
                        on_scroll: on_scroll_handler,
                    });
                }

                // Render children clipped to viewport
                let needs_clip =
                    b.overflow_x != Overflow::Visible || b.overflow_y != Overflow::Visible;
                if needs_clip {
                    let clip_rect = Rect::new(
                        inner_x.max(0) as u16,
                        inner_y.max(0) as u16,
                        viewport_w,
                        viewport_h,
                    );
                    buffer.push_clip(clip_rect);
                }

                if !b.children.is_empty() {
                    let start_y = match b.align_items {
                        Align::Center => {
                            inner_y + (viewport_h as i32 - total_children_h as i32) / 2
                        }
                        Align::End => inner_y + (viewport_h as i32 - total_children_h as i32),
                        _ => inner_y,
                    } - scroll_y as i32;

                    let mut curr_y = start_y;
                    for (child, &(cw, ch)) in b.children.iter().zip(measured_children.iter()) {
                        let start_x = match b.justify_content {
                            Justify::Center => inner_x + (viewport_w as i32 - cw as i32) / 2,
                            Justify::End => inner_x + (viewport_w as i32 - cw as i32),
                            _ => inner_x,
                        } - scroll_x as i32;

                        render_node_offset(child, start_x, curr_y, cw, ch, buffer, hit_map);
                        curr_y += ch as i32;
                    }
                }

                if needs_clip {
                    buffer.pop_clip();
                }
            } else if b.on_click.is_some() || b.focus_id.is_some() || b.on_key.is_some() {
                let target_rect = Rect::new(x.max(0) as u16, y.max(0) as u16, width, height);
                hit_map.register(HitTarget {
                    rect: target_rect,
                    on_click: b.on_click.clone(),
                    focus_id: b.focus_id,
                    on_key: b.on_key.clone(),
                    on_scroll: None,
                });
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
            on_scroll: None,
        });

        assert!(hit_map.hit_test(6, 6).is_some());
        assert!(hit_map.hit_test(0, 0).is_none());
        assert_eq!(hit_map.focusable_ids(), vec![FocusId(1)]);
    }

    #[test]
    fn test_stable_focus_ids_across_renders() {
        reset_focus_counter();
        let id1 = next_render_focus_id();
        let id2 = next_render_focus_id();
        assert_eq!(id1, FocusId(0));
        assert_eq!(id2, FocusId(1));

        set_focused(Some(id2));

        reset_focus_counter();
        let frame2_id1 = next_render_focus_id();
        let frame2_id2 = next_render_focus_id();
        assert_eq!(frame2_id1, FocusId(0));
        assert_eq!(frame2_id2, FocusId(1));

        assert!(!is_focused(frame2_id1));
        assert!(is_focused(frame2_id2));

        // User-created IDs are globally unique and never collide with render-counter IDs
        let u1 = FocusId::new();
        let u2 = FocusId::new();
        assert!(u1.0 >= 100_000);
        assert_eq!(u2.0, u1.0 + 1);
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

    #[test]
    fn test_box_min_max_dimensions() {
        let b = BoxElement::new()
            .width(10)
            .min_width(20)
            .max_width(50)
            .into_node();
        let (w, _) = measure_node(&b, 100, 100);
        assert_eq!(w, 20); // Clamped to min_width 20

        let b2 = BoxElement::new()
            .width(60)
            .min_width(20)
            .max_width(50)
            .into_node();
        let (w2, _) = measure_node(&b2, 100, 100);
        assert_eq!(w2, 50); // Clamped to max_width 50
    }

    #[test]
    fn test_box_overflow_scroll() {
        let mut buffer = Buffer::new(20, 5);
        let mut hit_map = HitMap::new();

        let node = BoxElement::new()
            .width(20)
            .height(5)
            .overflow(Overflow::Scroll)
            .child(TextElement::new("Line 1"))
            .child(TextElement::new("Line 2"))
            .child(TextElement::new("Line 3"))
            .child(TextElement::new("Line 4"))
            .child(TextElement::new("Line 5"))
            .child(TextElement::new("Line 6"))
            .child(TextElement::new("Line 7"))
            .child(TextElement::new("Line 8"))
            .into_node();

        render_node_interactive(&node, Rect::new(0, 0, 20, 5), &mut buffer, &mut hit_map);

        // Content should render within bounds, and scrollbar track/thumb should be on column 19
        assert_eq!(buffer.get(0, 0).unwrap().ch, 'L');
        assert_eq!(buffer.get(1, 0).unwrap().ch, 'i');
        // Scrollbar thumb or track is drawn at x=19 with full block █
        let cell = buffer.get(19, 0).unwrap();
        assert_eq!(cell.ch, '█');
    }

    #[test]
    fn test_horizontal_scrollbar_ui_parity() {
        let mut buffer = Buffer::new(10, 5);
        let mut hit_map = HitMap::new();

        let node = BoxElement::new()
            .width(10)
            .height(5)
            .overflow_x(Overflow::Scroll)
            .child(TextElement::new(
                "A very long horizontal sentence exceeding 10 cols",
            ))
            .into_node();

        render_node_interactive(&node, Rect::new(0, 0, 10, 5), &mut buffer, &mut hit_map);

        // Horizontal scrollbar thumb or track is drawn at y=4 with matching lower half block ▄
        let cell = buffer.get(0, 4).unwrap();
        assert_eq!(cell.ch, '▄');
    }

    #[test]
    fn test_horizontal_text_scrolling_offset() {
        let mut buffer = Buffer::new(10, 3);
        let mut hit_map = HitMap::new();

        let node = BoxElement::new()
            .width(10)
            .height(3)
            .border(BorderStyle::None)
            .overflow_x(Overflow::Scroll)
            .scroll_x(5)
            .child(TextElement::new("0123456789ABCDEF"))
            .into_node();

        render_node_interactive(&node, Rect::new(0, 0, 10, 3), &mut buffer, &mut hit_map);

        // At x=0, we expect character '5' because scroll_x = 5
        assert_eq!(buffer.get(0, 0).unwrap().ch, '5');
        assert_eq!(buffer.get(1, 0).unwrap().ch, '6');
    }

    #[test]
    fn test_mouse_scroll_handler_horizontal_and_fallback() {
        let mut buffer = Buffer::new(20, 5);
        let mut hit_map = HitMap::new();

        let node = BoxElement::new()
            .width(20)
            .height(5)
            .overflow_x(Overflow::Scroll)
            .child(TextElement::new("0123456789012345678901234567890123456789"))
            .into_node();

        reset_focus_counter();
        render_node_interactive(&node, Rect::new(0, 0, 20, 5), &mut buffer, &mut hit_map);

        // Even without explicit focus_id, on_scroll handler should be registered
        let target = hit_map.hit_test(5, 2).expect("target should be hit");
        let on_scroll = target.on_scroll.as_ref().expect("on_scroll should exist");

        // Scroll horizontally by 4
        on_scroll(4, 0);

        // Re-render and verify text shifted (App calls reset_focus_counter each frame)
        reset_focus_counter();
        let mut buffer2 = Buffer::new(20, 5);
        let mut hit_map2 = HitMap::new();
        render_node_interactive(&node, Rect::new(0, 0, 20, 5), &mut buffer2, &mut hit_map2);
        assert_eq!(buffer2.get(0, 0).unwrap().ch, '4');

        // Test vertical wheel translation on horizontal-only container (dx=0, dy=1 -> horizontal scroll)
        let target2 = hit_map2.hit_test(5, 2).expect("target2 should be hit");
        let on_scroll2 = target2.on_scroll.as_ref().expect("on_scroll2 should exist");
        on_scroll2(0, 1);

        reset_focus_counter();
        let mut buffer3 = Buffer::new(20, 5);
        let mut hit_map3 = HitMap::new();
        render_node_interactive(&node, Rect::new(0, 0, 20, 5), &mut buffer3, &mut hit_map3);
        // 4 + 2 = 6
        assert_eq!(buffer3.get(0, 0).unwrap().ch, '6');
    }

    #[test]
    fn test_nested_scroll_box_scroll() {
        let scroll_id = FocusId::new(); // User calls FocusId::new() in main()
        let tree = BoxElement::new() // Root Box
            .child(
                BoxElement::new() // Dialog Box
                    .child(
                        BoxElement::new() // Scrollable Box
                            .focus_id(scroll_id)
                            .height(3)
                            .overflow_y(Overflow::Scroll)
                            .child(TextElement::new("Line 1"))
                            .child(TextElement::new("Line 2"))
                            .child(TextElement::new("Line 3"))
                            .child(TextElement::new("Line 4"))
                            .child(TextElement::new("Line 5"))
                            .child(TextElement::new("Line 6")),
                    ),
            )
            .into_node();

        reset_focus_counter();
        let mut buffer = Buffer::new(20, 10);
        let mut hit_map = HitMap::new();
        render_node_interactive(&tree, Rect::new(0, 0, 20, 10), &mut buffer, &mut hit_map);

        // Hit test scroll container
        let target = hit_map
            .targets
            .iter()
            .find(|t| t.focus_id == Some(scroll_id))
            .expect("target should be found");
        let on_scroll = target.on_scroll.as_ref().expect("on_scroll should exist");
        on_scroll(0, 2);

        // Next frame render
        reset_focus_counter();
        let mut buffer2 = Buffer::new(20, 10);
        let mut hit_map2 = HitMap::new();
        render_node_interactive(&tree, Rect::new(0, 0, 20, 10), &mut buffer2, &mut hit_map2);

        assert_eq!(get_scroll(scroll_id), (0, 2));
    }
}
