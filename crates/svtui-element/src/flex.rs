use crate::measure::{Bounds, child_box, children, in_flow, limits, measure, resolve_main_sizes};
use crate::node::{FlexBoxElement, VNode};
use svtui_layout::{Align, Rect, ScrollbarVisibility, Size, place_flex_children};

pub(crate) struct ChildLayout<'a> {
    pub node: &'a VNode,
    pub rect: Rect,
}

pub(crate) struct BoxLayout<'a> {
    pub children: Vec<ChildLayout<'a>>,
    pub viewport: (u16, u16),
    pub content: (u16, u16),
    pub vertical_scrollbar: bool,
    pub horizontal_scrollbar: bool,
}

fn arrange(b: &FlexBoxElement, viewport: (u16, u16)) -> Vec<ChildLayout<'_>> {
    let nodes = children(&b.children);
    let flow: Vec<_> = nodes.iter().copied().filter(|n| in_flow(n)).collect();
    let direction = b.flex_direction;
    let (main, cross) = direction.axes(viewport.0, viewport.1);
    let mut sizes = Vec::new();
    for &node in &flow {
        let style = child_box(node);
        let bounds = Bounds::new(
            (!b.overflow_x.is_scrollable() || style.is_some_and(|s| s.width != Size::Auto))
                .then_some(viewport.0),
            (!b.overflow_y.is_scrollable() || style.is_some_and(|s| s.height != Size::Auto))
                .then_some(viewport.1),
        );
        let (mut width, mut height) = measure(node, bounds);
        if let Some(style) = style {
            let (min_w, max_w) = limits(style.min_width, style.max_width, Some(viewport.0));
            let (min_h, max_h) = limits(style.min_height, style.max_height, Some(viewport.1));
            width = width.clamp(min_w, max_w);
            height = height.clamp(min_h, max_h);
        }
        let (child_main, mut child_cross) = direction.axes(width, height);
        if let Some(style) = style {
            let (_, cross_size) = direction.axes(style.width, style.height);
            let (_, min_cross) = direction.axes(style.min_width, style.min_height);
            let (_, max_cross) = direction.axes(style.max_width, style.max_height);
            if cross_size == Size::Fill
                || (cross_size == Size::Auto && b.align_items == Align::Stretch)
            {
                let (min, max) = limits(min_cross, max_cross, Some(cross));
                child_cross = cross.clamp(min, max);
            }
        } else if b.align_items == Align::Stretch {
            child_cross = cross;
        }
        sizes.push(direction.axes(child_main, child_cross));
    }
    resolve_main_sizes(&flow, &mut sizes, direction, main, b.gap);
    let mut rectangles = place_flex_children(
        direction,
        viewport,
        &sizes,
        b.gap,
        b.align_items,
        b.justify_content,
    )
    .into_iter();
    nodes
        .into_iter()
        .map(|node| ChildLayout {
            node,
            rect: if in_flow(node) {
                rectangles.next().unwrap()
            } else {
                Rect::new(0, 0, 0, 0)
            },
        })
        .collect()
}

pub(crate) fn layout_box(b: &FlexBoxElement, width: u16, height: u16) -> BoxLayout<'_> {
    let mut vertical =
        b.overflow_y.is_scrollable() && b.scrollbar_visibility == ScrollbarVisibility::Always;
    let mut horizontal =
        b.overflow_x.is_scrollable() && b.scrollbar_visibility == ScrollbarVisibility::Always;
    loop {
        let viewport = (
            width.saturating_sub(u16::from(vertical)),
            height.saturating_sub(u16::from(horizontal)),
        );
        let children = arrange(b, viewport);
        let content = children
            .iter()
            .filter(|c| in_flow(c.node))
            .fold((0, 0), |(w, h), c| {
                (
                    w.max(c.rect.x.saturating_add(c.rect.width)),
                    h.max(c.rect.y.saturating_add(c.rect.height)),
                )
            });
        let show_vertical = vertical
            || (b.scrollbar_visibility == ScrollbarVisibility::Auto
                && b.overflow_y.is_scrollable()
                && content.1 > viewport.1);
        let show_horizontal = horizontal
            || (b.scrollbar_visibility == ScrollbarVisibility::Auto
                && b.overflow_x.is_scrollable()
                && content.0 > viewport.0);
        if show_vertical == vertical && show_horizontal == horizontal {
            return BoxLayout {
                children,
                viewport,
                content,
                vertical_scrollbar: vertical,
                horizontal_scrollbar: horizontal,
            };
        }
        vertical = show_vertical;
        horizontal = show_horizontal;
    }
}
