use crate::element::node::VNode;
use crate::layout::{FlexDirection, FlexDirectionExt};

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

pub(crate) fn children(nodes: &[VNode]) -> Vec<&VNode> {
    fn collect<'a>(nodes: &'a [VNode], output: &mut Vec<&'a VNode>) {
        for node in nodes {
            match node {
                VNode::Empty => {}
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
        VNode::Dynamic(slot) => measure(&slot.refresh().resolve(), bounds),
        VNode::Overlay(overlay) => overlay
            .trigger
            .as_deref()
            .map_or((0, 0), |n| measure(n, bounds)),
        VNode::Text(text) => {
            let lines = crate::element::text::lines(text, bounds.width);
            let width = lines
                .iter()
                .map(|line| line.chars().count())
                .max()
                .unwrap_or(0);
            (
                width.min(usize::from(u16::MAX)) as u16,
                lines.len().min(usize::from(u16::MAX)) as u16,
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
        VNode::View(_) => {
            let layout = crate::element::taffy_bridge::layout(
                node,
                bounds.width.unwrap_or(u16::MAX),
                bounds.height.unwrap_or(u16::MAX),
            );
            (layout.rect.width, layout.rect.height)
        }
    }
}
