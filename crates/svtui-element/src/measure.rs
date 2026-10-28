use crate::node::{BoxElement, VNode};
use svtui_layout::{
    BorderStyle, FlexDirection, FlexItem, ScrollbarVisibility, Size, resolve_flex_sizes,
};

/// Measure a node against a finite parent content area. Explicit sizes and
/// minimums may exceed that area; clipping belongs to the rendering phase.
pub fn measure_node(node: &VNode, max_w: u16, max_h: u16) -> (u16, u16) {
    measure(node, Bounds::new(Some(max_w), Some(max_h)))
}

#[derive(Clone, Copy)]
pub(crate) struct Bounds {
    pub width: Option<u16>,
    pub height: Option<u16>,
}

impl Bounds {
    pub fn new(width: Option<u16>, height: Option<u16>) -> Self {
        Self { width, height }
    }
}

pub(crate) fn limits(min: Option<Size>, max: Option<Size>, parent: Option<u16>) -> (u16, u16) {
    let minimum = min.and_then(|s| definite(s, parent)).unwrap_or(0);
    let maximum = max.and_then(|s| definite(s, parent)).unwrap_or(u16::MAX);
    (minimum, maximum.max(minimum))
}

fn definite(size: Size, parent: Option<u16>) -> Option<u16> {
    match size {
        Size::Fixed(value) => Some(value),
        Size::Percent(_) | Size::Fill => parent.map(|p| size.resolve(p, 0)),
        Size::Auto => None,
    }
}

fn resolve(
    size: Size,
    content: u16,
    parent: Option<u16>,
    min: Option<Size>,
    max: Option<Size>,
) -> u16 {
    let preferred =
        definite(size, parent).unwrap_or_else(|| content.min(parent.unwrap_or(u16::MAX)));
    let (min, max) = limits(min, max, parent);
    preferred.clamp(min, max)
}

pub(crate) fn decoration(b: &BoxElement) -> (u16, u16) {
    let border: u16 = if b.border == BorderStyle::None { 0 } else { 2 };
    (
        border
            .saturating_add(b.padding.left)
            .saturating_add(b.padding.right),
        border
            .saturating_add(b.padding.top)
            .saturating_add(b.padding.bottom),
    )
}

pub(crate) fn children(nodes: &[VNode]) -> Vec<&VNode> {
    fn collect<'a>(nodes: &'a [VNode], output: &mut Vec<&'a VNode>) {
        for node in nodes {
            match node {
                VNode::Empty => {}
                VNode::Box(b) if b.interaction.as_ref().is_some_and(|i| i.hidden) => {}
                VNode::Fragment(items) => collect(items, output),
                _ => output.push(node),
            }
        }
    }
    let mut output = Vec::new();
    collect(nodes, &mut output);
    output
}

pub(crate) fn in_flow(node: &VNode) -> bool {
    match node {
        VNode::Empty => false,
        VNode::Overlay(overlay) => overlay.trigger.as_deref().is_some_and(in_flow),
        _ => true,
    }
}

pub(crate) fn child_box(node: &VNode) -> Option<&BoxElement> {
    match node {
        VNode::Box(b) => Some(b),
        VNode::Overlay(overlay) => overlay.trigger.as_deref().and_then(child_box),
        _ => None,
    }
}

pub(crate) fn resolve_main_sizes(
    nodes: &[&VNode],
    sizes: &mut [(u16, u16)],
    direction: FlexDirection,
    main: u16,
    gap: u16,
) {
    let mut items: Vec<_> = nodes
        .iter()
        .zip(sizes.iter())
        .map(|(node, &(w, h))| {
            let mut item = FlexItem {
                size: direction.axes(w, h).0,
                min: 0,
                max: u16::MAX,
                fill: false,
            };
            if let Some(style) = child_box(node) {
                let (size, _) = direction.axes(style.width, style.height);
                let (min, _) = direction.axes(style.min_width, style.min_height);
                let (max, _) = direction.axes(style.max_width, style.max_height);
                (item.min, item.max) = limits(min, max, Some(main));
                item.fill = size == Size::Fill;
            }
            item
        })
        .collect();
    resolve_flex_sizes(&mut items, main, gap);
    for (size, item) in sizes.iter_mut().zip(items) {
        let (_, cross) = direction.axes(size.0, size.1);
        *size = direction.axes(item.size, cross);
    }
}

pub(crate) fn content_size(direction: FlexDirection, sizes: &[(u16, u16)], gap: u16) -> (u16, u16) {
    let mut main = 0_u64;
    let mut cross = 0;
    for &(width, height) in sizes {
        let (m, c) = direction.axes(width, height);
        main += u64::from(m);
        cross = cross.max(c);
    }
    main += u64::from(gap) * sizes.len().saturating_sub(1) as u64;
    direction.axes(main.min(u64::from(u16::MAX)) as u16, cross)
}

pub(crate) fn measure(node: &VNode, bounds: Bounds) -> (u16, u16) {
    match node {
        VNode::Empty => (0, 0),
        VNode::Overlay(overlay) => overlay
            .trigger
            .as_deref()
            .map_or((0, 0), |n| measure(n, bounds)),
        VNode::Text(text) => {
            let width = text
                .content
                .lines()
                .map(|line| line.chars().count())
                .max()
                .unwrap_or(0);
            (
                width.min(usize::from(u16::MAX)) as u16,
                text.content
                    .lines()
                    .count()
                    .max(1)
                    .min(usize::from(u16::MAX)) as u16,
            )
        }
        VNode::Fragment(items) => {
            let sizes: Vec<_> = children(items)
                .into_iter()
                .filter(|n| in_flow(n))
                .map(|n| measure(n, bounds))
                .collect();
            content_size(FlexDirection::Column, &sizes, 0)
        }
        VNode::Box(b) => {
            let (extra_w, extra_h) = decoration(b);
            let known_w = definite(b.width, bounds.width).map(|w| {
                let (min, max) = limits(b.min_width, b.max_width, bounds.width);
                w.clamp(min, max).saturating_sub(extra_w)
            });
            let known_h = definite(b.height, bounds.height).map(|h| {
                let (min, max) = limits(b.min_height, b.max_height, bounds.height);
                h.clamp(min, max).saturating_sub(extra_h)
            });
            let flow: Vec<_> = children(&b.children)
                .into_iter()
                .filter(|n| in_flow(n))
                .collect();
            let mut sizes: Vec<_> = flow
                .iter()
                .map(|n| measure(n, Bounds::new(known_w, known_h)))
                .collect();
            if let Some(main) = b.flex_direction.axes(known_w, known_h).0 {
                resolve_main_sizes(&flow, &mut sizes, b.flex_direction, main, b.gap);
            }
            let (content_w, content_h) = content_size(b.flex_direction, &sizes, b.gap);
            let natural_w =
                content_w
                    .saturating_add(extra_w)
                    .max(b.title.as_ref().map_or(0, |t| {
                        (t.chars().count().saturating_add(4)).min(usize::from(u16::MAX)) as u16
                    }));
            let natural_h = content_h.saturating_add(extra_h);
            let mut width = resolve(b.width, natural_w, bounds.width, b.min_width, b.max_width);
            let mut height = resolve(
                b.height,
                natural_h,
                bounds.height,
                b.min_height,
                b.max_height,
            );
            // Auto-sized containers reserve room for their scrollbars too.
            let mut vertical = false;
            let mut horizontal = false;
            for _ in 0..3 {
                if b.scrollbar_visibility != ScrollbarVisibility::Never {
                    vertical |= b.overflow_y.is_scrollable()
                        && (b.scrollbar_visibility == ScrollbarVisibility::Always
                            || content_h
                                > height
                                    .saturating_sub(extra_h)
                                    .saturating_sub(u16::from(horizontal)));
                    horizontal |= b.overflow_x.is_scrollable()
                        && (b.scrollbar_visibility == ScrollbarVisibility::Always
                            || content_w
                                > width
                                    .saturating_sub(extra_w)
                                    .saturating_sub(u16::from(vertical)));
                }
                if b.width == Size::Auto {
                    width = resolve(
                        b.width,
                        natural_w.saturating_add(u16::from(vertical)),
                        bounds.width,
                        b.min_width,
                        b.max_width,
                    );
                }
                if b.height == Size::Auto {
                    height = resolve(
                        b.height,
                        natural_h.saturating_add(u16::from(horizontal)),
                        bounds.height,
                        b.min_height,
                        b.max_height,
                    );
                }
            }
            (width, height)
        }
    }
}
