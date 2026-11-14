//! Layout props that come straight from taffy's style: auto margins,
//! absolute positioning, separate gaps, `display: none`, aspect ratio and
//! fragment flattening.

use sveltui::element::render_node;
use sveltui::element::taffy_bridge::{ComputedLayoutNode, LayoutEngine};
use sveltui::layout::Rect;
use sveltui::prelude::*;
use sveltui::terminal::Buffer;
use sveltui::view;

fn layout(node: impl IntoNode, width: u16, height: u16) -> ComputedLayoutNode {
    LayoutEngine::new().compute_layout(&node.into_node(), width, height)
}

fn rect(layout: &ComputedLayoutNode) -> (u16, u16, u16, u16) {
    let r = layout.rect;
    (r.x, r.y, r.width, r.height)
}

#[test]
fn auto_side_margins_center_horizontally() {
    let root = layout(
        view! {
            <View width=40 height=10>
                <View width=10 height=2 margin={Margin::auto_x()} />
            </View>
        },
        40,
        10,
    );
    assert_eq!(rect(&root.children[0]), (15, 0, 10, 2));
}

#[test]
fn auto_margins_center_both_ways() {
    let root = layout(
        view! {
            <View width=40 height=10>
                <View width=10 height=2 margin={Margin::auto()} />
            </View>
        },
        40,
        10,
    );
    assert_eq!(rect(&root.children[0]), (15, 4, 10, 2));
}

#[test]
fn absolute_views_are_placed_by_inset_and_leave_the_flow() {
    let root = layout(
        view! {
            <View width=40 height=10>
                <View position=Position::Absolute inset={Inset::new().top(0).right(1)} width=5 height=1 />
                <View height=1 />
            </View>
        },
        40,
        10,
    );
    assert_eq!(rect(&root.children[0]), (34, 0, 5, 1));
    assert_eq!(
        root.children[1].rect.y, 0,
        "the flow child doesn't make room for the absolute one"
    );
}

#[test]
fn row_and_column_gaps_are_separate() {
    let root = layout(
        view! {
            <View width=9 flex_direction=FlexDirection::Row flex_wrap=FlexWrap::Wrap
                column_gap=1 row_gap=2>
                <View width=4 height=1 />
                <View width=4 height=1 />
                <View width=4 height=1 />
                <View width=4 height=1 />
            </View>
        },
        9,
        10,
    );
    let positions: Vec<_> = root.children.iter().map(|c| (c.rect.x, c.rect.y)).collect();
    assert_eq!(positions, [(0, 0), (5, 0), (0, 3), (5, 3)]);
}

#[test]
fn display_none_takes_no_space_and_draws_nothing() {
    let node = view! {
        <View width=20 height=3>
            <View display=Display::None height=1><TextSpan>"hidden"</TextSpan></View>
            <TextSpan>"shown"</TextSpan>
        </View>
    }
    .into_node();
    let root = layout(node.clone(), 20, 3);
    assert_eq!(root.children[1].rect.y, 0);

    let mut buffer = Buffer::new(20, 3);
    render_node(&node, Rect::new(0, 0, 20, 3), &mut buffer);
    let first_row: String = (0..20).map(|x| buffer.get(x, 0).unwrap().ch).collect();
    assert_eq!(first_row.trim_end(), "shown");
}

#[test]
fn aspect_ratio_sets_the_missing_side() {
    let root = layout(
        view! {
            <View width=40 height=20 align_items=AlignItems::Start>
                <View width=10 aspect_ratio=2.0 />
            </View>
        },
        40,
        20,
    );
    assert_eq!(rect(&root.children[0]), (0, 0, 10, 5));
}

#[test]
fn nested_fragments_join_their_parent_row() {
    let items = VNode::Fragment(vec![
        VNode::Fragment(vec![
            view! { <TextSpan>"a"</TextSpan> }.into_node(),
            view! { <TextSpan>"b"</TextSpan> }.into_node(),
        ]),
        VNode::Fragment(vec![view! { <TextSpan>"c"</TextSpan> }.into_node()]),
    ]);
    let root = layout(
        view! { <View flex_direction=FlexDirection::Row gap=1>{items}</View> },
        20,
        1,
    );
    let positions: Vec<_> = root.children.iter().map(|c| (c.rect.x, c.rect.y)).collect();
    assert_eq!(positions, [(0, 0), (2, 0), (4, 0)]);
}

#[test]
fn a_row_keeps_its_gaps_next_to_an_auto_margin() {
    let root = layout(
        view! {
            <View width=30 flex_direction=FlexDirection::Row gap=2>
                <View width=4 height=1 />
                <View width=4 height=1 />
                <View width=5 height=1 margin={Margin { left: MarginEdge::Auto, ..Margin::zero() }} />
            </View>
        },
        30,
        1,
    );
    let xs: Vec<_> = root.children.iter().map(|c| c.rect.x).collect();
    assert_eq!(
        xs,
        [0, 6, 25],
        "gap between the first two, the last flush right"
    );
}
