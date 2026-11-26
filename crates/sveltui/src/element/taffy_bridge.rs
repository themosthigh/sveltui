use crate::element::node::{TextElement, VNode, ViewElement};
use crate::layout::geometry::Rect;
use crate::layout::{BorderStyle, Padding};
use taffy::prelude::*;

#[derive(Clone, Debug, Default)]
pub struct ComputedLayoutNode {
    pub rect: Rect,
    /// Border plus padding on each side: where the children's area starts.
    pub content_inset: Padding,
    /// Just the border on each side. Scrollbars sit just inside it, outside
    /// the padding, the way a browser draws them.
    pub border_inset: Padding,
    pub viewport_size: (u16, u16),
    /// Whether layout set aside room for a scrollbar on the right and at the
    /// bottom. It does so only for views whose size is bounded; a view sized
    /// by flex gets no room, so the renderer takes the bars' cells itself.
    pub scrollbar_reserved: (bool, bool),
    pub content_size: (u16, u16),
    pub scroll_size: (u16, u16),
    /// Where a grid's rules go, when it draws them (see `grid_rules`).
    pub rules: Option<Box<RuleLines>>,
    pub children: Vec<ComputedLayoutNode>,
}

/// The middle of each gap between a grid's tracks, relative to the grid's
/// own top-left corner, in content coordinates (before scrolling).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuleLines {
    /// x of each line between two columns.
    pub columns: Vec<u16>,
    /// y of each line between two rows.
    pub rows: Vec<u16>,
}

/// The middle cell of each gap between tracks. Taffy lists a grid's
/// tracks as gutter, track, gutter, ..., gutter, starting at the content
/// edge; the outer gutters aren't between two tracks, so they get no line.
fn rule_positions(info: &taffy::DetailedGridTracksInfo, start: f32) -> Vec<u16> {
    let mut offset = start + info.gutters.first().copied().unwrap_or(0.0);
    let mut lines = Vec::new();
    for (index, size) in info.sizes.iter().enumerate() {
        offset += size;
        if index + 1 < info.sizes.len() {
            let gutter = info.gutters.get(index + 1).copied().unwrap_or(0.0);
            if gutter >= 1.0 {
                lines.push((offset + (gutter - 1.0) / 2.0).round().max(0.0) as u16);
            }
            offset += gutter;
        }
    }
    lines
}

/// Measured sizes for one text node, keyed by the width it was measured
/// against. Taffy asks about the same text several times per pass (min,
/// max and definite widths), and wrapping is the expensive part.
#[derive(Default)]
struct TextSizes(Vec<(Option<u16>, (f32, f32))>);

impl TextSizes {
    fn get(&mut self, text: &TextElement, width: Option<u16>) -> (f32, f32) {
        if let Some(&(_, size)) = self.0.iter().find(|(w, _)| *w == width) {
            return size;
        }
        let (widest, count) = crate::element::text::size(text, width);
        let size = (widest as f32, count as f32);
        self.0.push((width, size));
        size
    }
}

/// The taffy style for a view: its stored style, plus the parts that depend
/// on its other settings (the border it draws and whether it shows a
/// scrollbar).
pub fn view_to_taffy_style(view: &ViewElement) -> taffy::style::Style {
    let mut style = view.style.clone();
    style.align_items = style.align_items.or(Some(AlignItems::STRETCH));

    let border = view
        .visual
        .as_ref()
        .is_some_and(|v| v.border != BorderStyle::None);
    let inset = LengthPercentage::length(if border { 1.0 } else { 0.0 });
    style.border = taffy::geometry::Rect {
        left: inset,
        right: inset,
        top: inset,
        bottom: inset,
    };

    // A scrollbar only takes up room when the view scrolls and has a bound
    // on both axes; an unbounded view just grows to fit instead.
    let scrolls = style.overflow.x == taffy::style::Overflow::Scroll
        || style.overflow.y == taffy::style::Overflow::Scroll;
    let bounded = |size: Dimension, max: Dimension| !size.is_auto() || !max.is_auto();
    let shows_scrollbar = view
        .scroll
        .as_ref()
        .is_none_or(|s| s.scrollbar_visibility != crate::layout::ScrollbarVisibility::Never);
    // A rule needs a gap to sit in.
    if let Some(visual) = view
        .visual
        .as_ref()
        .filter(|v| v.grid_rules != BorderStyle::None)
    {
        let one = LengthPercentage::length(1.0);
        let none = LengthPercentage::length(0.0);
        if visual.grid_rule_axes.columns() && style.gap.width == none {
            style.gap.width = one;
        }
        if visual.grid_rule_axes.rows() && style.gap.height == none {
            style.gap.height = one;
        }
    }

    style.scrollbar_width = if scrolls
        && bounded(style.size.width, style.max_size.width)
        && bounded(style.size.height, style.max_size.height)
        && shows_scrollbar
    {
        1.0
    } else {
        0.0
    };
    style
}

/// What a laid-out node is, which decides what can be reused for it.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    View,
    Text,
    Fragment,
    /// Takes no room: empty fragments, empty overlays, anything unresolved.
    Hidden,
}

/// A node of the tree we kept from the last frame: its taffy node, and what
/// we told taffy about it, so the next frame can tell what changed.
struct Kept {
    node: NodeId,
    kind: Kind,
    style: Style,
    /// For text, its place in the engine's list of texts.
    text: Option<usize>,
    children: Vec<Kept>,
}

/// A text node's content and the sizes measured for it.
struct TextSlot {
    text: TextElement,
    sizes: TextSizes,
}

/// Lays out VNode trees with taffy, and remembers the last tree it laid
/// out. The next call walks the new tree against the remembered one and
/// tells taffy only about what differs, so taffy's own caches still hold
/// for everything that did not change: typing in one field costs that field
/// and its ancestors, not every row beside it.
pub struct LayoutEngine {
    /// Text leaves carry their index into `texts`.
    tree: TaffyTree<usize>,
    root: Option<Kept>,
    texts: Vec<Option<TextSlot>>,
    free_texts: Vec<usize>,
}

#[cfg(test)]
thread_local! {
    /// How many layout passes have run on this thread, so tests can check a
    /// frame doesn't lay out more than it needs to.
    pub(crate) static LAYOUT_PASSES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    /// How many nodes taffy was told changed (made, restyled or marked dirty)
    /// in the last pass.
    pub(crate) static LAST_CHANGES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

thread_local! {
    static SHARED_ENGINE: std::cell::RefCell<LayoutEngine> =
        std::cell::RefCell::new(LayoutEngine::new());
}

/// Lays out `root` with this thread's shared engine, so each frame is
/// compared with the one before. If the shared engine is already busy (a
/// layout running inside another), we fall back to a fresh one.
pub fn layout(root: &VNode, available_w: u16, available_h: u16) -> ComputedLayoutNode {
    SHARED_ENGINE.with(|engine| match engine.try_borrow_mut() {
        Ok(mut engine) => engine.compute_layout(root, available_w, available_h),
        Err(_) => LayoutEngine::new().compute_layout(root, available_w, available_h),
    })
}

impl Default for LayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}

fn hidden_style() -> Style {
    Style {
        display: Display::None,
        ..Default::default()
    }
}

fn fragment_style() -> Style {
    Style {
        display: Display::Flex,
        flex_direction: FlexDirection::Column,
        size: taffy::geometry::Size {
            width: Dimension::auto(),
            height: Dimension::auto(),
        },
        ..Default::default()
    }
}

fn text_style(text: &TextElement) -> Style {
    // Unwrapped text keeps its intrinsic minimum so stretching a
    // narrow parent doesn't silently truncate visible overflow.
    let min_size = if !text.wrap && text.fill.is_none() {
        let lines = crate::element::text::lines(text, None);
        taffy::geometry::Size {
            width: Dimension::length(
                lines
                    .iter()
                    .map(|line| line.chars().count())
                    .max()
                    .unwrap_or(0) as f32,
            ),
            height: Dimension::length(lines.len() as f32),
        }
    } else {
        taffy::geometry::Size::auto()
    };
    Style {
        display: Display::Flex,
        min_size,
        size: taffy::geometry::Size {
            width: Dimension::auto(),
            height: Dimension::auto(),
        },
        align_self: if text.fill.is_some() {
            Some(AlignSelf::STRETCH)
        } else {
            // Inherit the container's align_items just like View
            // children; forcing FlexStart disables center/end/stretch.
            None
        },
        flex_grow: if text.fill.is_some() { 1.0 } else { 0.0 },
        flex_shrink: 0.0,
        ..Default::default()
    }
}

/// What a node asks of the engine: its kind, its style, and its children.
enum Shape<'a> {
    View(Box<Style>, &'a [VNode]),
    Fragment(&'a [VNode]),
    Text(&'a TextElement),
    Hidden,
}

impl LayoutEngine {
    pub fn new() -> Self {
        Self {
            tree: TaffyTree::new(),
            root: None,
            texts: Vec::new(),
            free_texts: Vec::new(),
        }
    }

    /// Computes full layout for a VNode tree within given available dimensions
    pub fn compute_layout(
        &mut self,
        root: &VNode,
        available_w: u16,
        available_h: u16,
    ) -> ComputedLayoutNode {
        #[cfg(test)]
        {
            LAYOUT_PASSES.with(|passes| passes.set(passes.get() + 1));
            LAST_CHANGES.with(|changes| changes.set(0));
        }
        let previous = self.root.take();
        let kept = self.sync(root, previous);
        let taffy_root = kept.node;
        self.root = Some(kept);

        let available = taffy::geometry::Size {
            width: AvailableSpace::Definite(available_w as f32),
            height: AvailableSpace::Definite(available_h as f32),
        };
        self.compute_taffy_layout(taffy_root, available);
        self.extract_node_layout(taffy_root, root)
    }

    fn compute_taffy_layout(
        &mut self,
        root: NodeId,
        available: taffy::geometry::Size<AvailableSpace>,
    ) {
        let texts = &mut self.texts;
        let _ = self.tree.compute_layout_with_measure(
            root,
            available,
            |known_dimensions, available_space, _node_id, context, _style| {
                let (width, height) = match context {
                    Some(&mut index) => {
                        let max_width = match available_space.width {
                            AvailableSpace::Definite(w) => Some(w.max(0.0) as u16),
                            _ => None,
                        };
                        match texts.get_mut(index).and_then(Option::as_mut) {
                            Some(slot) => slot.sizes.get(&slot.text, max_width),
                            None => (0.0, 0.0),
                        }
                    }
                    None => (0.0, 0.0),
                };
                taffy::geometry::Size {
                    width: known_dimensions.width.unwrap_or(width),
                    height: known_dimensions.height.unwrap_or(height),
                }
            },
        );
    }

    fn changed(&self) {
        #[cfg(test)]
        LAST_CHANGES.with(|changes| changes.set(changes.get() + 1));
    }

    fn shape<'a>(node: &'a VNode) -> Shape<'a> {
        match node {
            VNode::View(view) => Shape::View(Box::new(view_to_taffy_style(view)), &view.children),
            VNode::Text(text) => Shape::Text(text),
            // An overlay lays out as its trigger; the layer itself is
            // placed when painting.
            VNode::Overlay(overlay) => match &overlay.trigger {
                Some(trigger) => Self::shape(trigger),
                None => Shape::Hidden,
            },
            VNode::Fragment(children) if !children.is_empty() => Shape::Fragment(children),
            // Slots are resolved before layout starts, so one here is empty.
            VNode::Fragment(_) | VNode::Empty | VNode::Dynamic(_) => Shape::Hidden,
        }
    }

    /// The overlay's trigger when `node` is an overlay with one, else
    /// `node`: the node whose children and content we lay out.
    fn through_overlay(node: &VNode) -> &VNode {
        match node {
            VNode::Overlay(overlay) => match &overlay.trigger {
                Some(trigger) => Self::through_overlay(trigger),
                None => node,
            },
            other => other,
        }
    }

    /// Brings the kept tree in line with `node`, telling taffy about what
    /// changed and nothing else, and returns the new kept tree.
    fn sync(&mut self, node: &VNode, old: Option<Kept>) -> Kept {
        let node = Self::through_overlay(node);
        let shape = Self::shape(node);
        let kind = match shape {
            Shape::View(..) => Kind::View,
            Shape::Fragment(_) => Kind::Fragment,
            Shape::Text(_) => Kind::Text,
            Shape::Hidden => Kind::Hidden,
        };
        let old = match old {
            Some(old) if old.kind == kind => Some(old),
            Some(old) => {
                self.discard(old);
                None
            }
            None => None,
        };

        match shape {
            Shape::Hidden => old.unwrap_or_else(|| {
                self.changed();
                Kept {
                    node: self.tree.new_leaf(hidden_style()).unwrap(),
                    kind,
                    style: hidden_style(),
                    text: None,
                    children: Vec::new(),
                }
            }),
            Shape::Text(text) => match old {
                Some(kept) => {
                    let slot = kept.text.and_then(|i| self.texts[i].as_mut());
                    if let Some(slot) = slot
                        && slot.text != *text
                    {
                        slot.text = text.clone();
                        slot.sizes = TextSizes::default();
                        let style = text_style(text);
                        let _ = self.tree.set_style(kept.node, style.clone());
                        let _ = self.tree.mark_dirty(kept.node);
                        self.changed();
                        return Kept { style, ..kept };
                    }
                    kept
                }
                None => {
                    let index = self.alloc_text(text);
                    let style = text_style(text);
                    self.changed();
                    Kept {
                        node: self
                            .tree
                            .new_leaf_with_context(style.clone(), index)
                            .unwrap_or_else(|_| self.tree.new_leaf(Style::default()).unwrap()),
                        kind,
                        style,
                        text: Some(index),
                        children: Vec::new(),
                    }
                }
            },
            Shape::View(style, children) => self.sync_parent(kind, *style, children, old),
            Shape::Fragment(children) => self.sync_parent(kind, fragment_style(), children, old),
        }
    }

    fn sync_parent(
        &mut self,
        kind: Kind,
        style: Style,
        children: &[VNode],
        old: Option<Kept>,
    ) -> Kept {
        let (old_node, old_style, old_children) = match old {
            Some(old) => (Some(old.node), Some(old.style), old.children),
            None => (None, None, Vec::new()),
        };
        let old_ids: Vec<NodeId> = old_children.iter().map(|c| c.node).collect();
        let mut previous = old_children.into_iter();
        let mut kept_children = Vec::with_capacity(children.len());
        for child in children {
            let counterpart = previous.next();
            kept_children.push(self.sync(child, counterpart));
        }
        for extra in previous {
            self.discard(extra);
        }
        let ids: Vec<NodeId> = kept_children.iter().map(|c| c.node).collect();

        let node = match old_node {
            Some(node) => {
                if old_style.as_ref() != Some(&style) {
                    let _ = self.tree.set_style(node, style.clone());
                    self.changed();
                }
                if old_ids != ids {
                    let _ = self.tree.set_children(node, &ids);
                    self.changed();
                }
                node
            }
            None => {
                self.changed();
                self.tree
                    .new_with_children(style.clone(), &ids)
                    .unwrap_or_else(|_| self.tree.new_leaf(Style::default()).unwrap())
            }
        };
        Kept {
            node,
            kind,
            style,
            text: None,
            children: kept_children,
        }
    }

    fn alloc_text(&mut self, text: &TextElement) -> usize {
        let slot = Some(TextSlot {
            text: text.clone(),
            sizes: TextSizes::default(),
        });
        match self.free_texts.pop() {
            Some(index) => {
                self.texts[index] = slot;
                index
            }
            None => {
                self.texts.push(slot);
                self.texts.len() - 1
            }
        }
    }

    /// Forgets a kept subtree, in taffy and in the text list.
    fn discard(&mut self, kept: Kept) {
        for child in kept.children {
            self.discard(child);
        }
        if let Some(index) = kept.text {
            self.texts[index] = None;
            self.free_texts.push(index);
        }
        let _ = self.tree.remove(kept.node);
        self.changed();
    }

    fn extract_node_layout(&self, taffy_node: NodeId, vnode: &VNode) -> ComputedLayoutNode {
        let l = self.tree.layout(taffy_node).cloned().unwrap_or_default();

        let x = l.location.x.max(0.0).round() as u16;
        let y = l.location.y.max(0.0).round() as u16;
        let width = l.size.width.max(0.0).round() as u16;
        let height = l.size.height.max(0.0).round() as u16;
        let content_w = l.content_size.width.max(0.0).round() as u16;
        let content_h = l.content_size.height.max(0.0).round() as u16;

        // Border plus padding, per side, as taffy resolved them.
        let cells = |value: f32| value.max(0.0).round() as u16;
        let content_inset = if matches!(vnode, VNode::View(_)) {
            Padding::only(
                cells(l.border.top + l.padding.top),
                cells(l.border.right + l.padding.right),
                cells(l.border.bottom + l.padding.bottom),
                cells(l.border.left + l.padding.left),
            )
        } else {
            Padding::zero()
        };
        let border_inset = if matches!(vnode, VNode::View(_)) {
            Padding::only(
                cells(l.border.top),
                cells(l.border.right),
                cells(l.border.bottom),
                cells(l.border.left),
            )
        } else {
            Padding::zero()
        };

        let (content_w, content_h, viewport_w, viewport_h) = if matches!(vnode, VNode::View(_)) {
            // Taffy measures content from inside the border, and its extent
            // includes both padding edges. Scroll offsets and clipping use
            // content coordinates, so we take the padding back off.
            let leading_w = cells(l.padding.left + l.padding.right);
            let leading_h = cells(l.padding.top + l.padding.bottom);
            (
                content_w.saturating_sub(leading_w),
                content_h.saturating_sub(leading_h),
                width
                    .saturating_sub(content_inset.left + content_inset.right)
                    .saturating_sub(l.scrollbar_size.width as u16),
                height
                    .saturating_sub(content_inset.top + content_inset.bottom)
                    .saturating_sub(l.scrollbar_size.height as u16),
            )
        } else {
            (content_w, content_h, width, height)
        };

        let rules = match vnode {
            VNode::View(v)
                if v.visual
                    .as_ref()
                    .is_some_and(|vis| vis.grid_rules != BorderStyle::None) =>
            {
                match self.tree.detailed_layout_info(taffy_node) {
                    taffy::DetailedLayoutInfo::Grid(info) => Some(Box::new(RuleLines {
                        columns: rule_positions(&info.columns, l.border.left + l.padding.left),
                        rows: rule_positions(&info.rows, l.border.top + l.padding.top),
                    })),
                    _ => None,
                }
            }
            _ => None,
        };

        let scroll_w = content_w.saturating_sub(viewport_w);
        let scroll_h = content_h.saturating_sub(viewport_h);

        let children_nodes = self.tree.children(taffy_node).unwrap_or_default();
        let child_vnodes = match vnode {
            VNode::View(view) => view.children.as_slice(),
            VNode::Fragment(children) => children.as_slice(),
            _ => &[],
        };

        let mut computed_children = Vec::new();
        for (idx, &child_taffy_id) in children_nodes.iter().enumerate() {
            let child_vnode = child_vnodes.get(idx).unwrap_or(&VNode::Empty);
            computed_children.push(self.extract_node_layout(child_taffy_id, child_vnode));
        }

        ComputedLayoutNode {
            rect: Rect::new(x, y, width, height),
            content_inset,
            border_inset,
            rules,
            viewport_size: (viewport_w, viewport_h),
            scrollbar_reserved: (l.scrollbar_size.width > 0.0, l.scrollbar_size.height > 0.0),
            content_size: (content_w, content_h),
            scroll_size: (scroll_w, scroll_h),
            children: computed_children,
        }
    }
}
