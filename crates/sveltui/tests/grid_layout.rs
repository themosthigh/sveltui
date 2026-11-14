//! Views laid out as grids: templates, fractions, spans, placement, gaps
//! and auto rows.

use sveltui::element::taffy_bridge::{ComputedLayoutNode, LayoutEngine};
use sveltui::prelude::*;
use sveltui::view;

fn layout(node: impl IntoNode, width: u16, height: u16) -> ComputedLayoutNode {
    LayoutEngine::new().compute_layout(&node.into_node(), width, height)
}

fn rects(layout: &ComputedLayoutNode) -> Vec<(u16, u16, u16, u16)> {
    layout
        .children
        .iter()
        .map(|c| (c.rect.x, c.rect.y, c.rect.width, c.rect.height))
        .collect()
}

#[test]
fn equal_fractions_split_the_grid_into_quadrants() {
    let root = layout(
        view! {
            <View display=Display::Grid width=40 height=10
                grid_template_columns={[fr(1.0), fr(1.0)]} grid_template_rows={[fr(1.0), fr(1.0)]}>
                <View /><View /><View /><View />
            </View>
        },
        40,
        10,
    );
    assert_eq!(
        rects(&root),
        [(0, 0, 20, 5), (20, 0, 20, 5), (0, 5, 20, 5), (20, 5, 20, 5)]
    );
}

#[test]
fn a_fixed_sidebar_and_a_flexible_main_area() {
    let root = layout(
        view! {
            <View display=Display::Grid width=60 height=10 column_gap=1
                grid_template_columns={[length(20), fr(1.0)]}>
                <View /><View />
            </View>
        },
        60,
        10,
    );
    assert_eq!(rects(&root), [(0, 0, 20, 10), (21, 0, 39, 10)]);
}

#[test]
fn spans_and_explicit_lines_place_children() {
    let root = layout(
        view! {
            <View display=Display::Grid width=30 height=9
                grid_template_columns={[evenly(3)]} grid_template_rows={[length(3), fr(1.0), length(1)]}>
                <View grid_column={span(3)} />
                <View grid_column={lines(1, 3)} />
                <View />
                <View grid_row={line(3)} grid_column={lines(1, -1)} />
            </View>
        },
        30,
        9,
    );
    assert_eq!(
        rects(&root),
        [(0, 0, 30, 3), (0, 3, 20, 5), (20, 3, 10, 5), (0, 8, 30, 1)]
    );
}

#[test]
fn extra_children_go_into_auto_rows() {
    let root = layout(
        view! {
            <View display=Display::Grid width=20 row_gap=1
                grid_template_columns={[repeat(2, [fr(1.0)])]} grid_auto_rows={length(2)}>
                <View /><View /><View /><View /><View />
            </View>
        },
        20,
        20,
    );
    let ys: Vec<_> = root
        .children
        .iter()
        .map(|c| (c.rect.y, c.rect.height))
        .collect();
    assert_eq!(ys, [(0, 2), (0, 2), (3, 2), (3, 2), (6, 2)]);
    assert_eq!(root.rect.height, 8, "three auto rows and two gaps");
}

#[test]
fn auto_fill_fits_as_many_columns_as_there_is_room_for() {
    let root = layout(
        view! {
            <View display=Display::Grid width=35
                grid_template_columns={[repeat(Repeat::AutoFill, [length(10)])]}>
                <View height=1 /><View height=1 /><View height=1 /><View height=1 />
            </View>
        },
        35,
        5,
    );
    let positions: Vec<_> = root.children.iter().map(|c| (c.rect.x, c.rect.y)).collect();
    assert_eq!(positions, [(0, 0), (10, 0), (20, 0), (0, 1)]);
}

#[test]
fn column_flow_fills_columns_first() {
    let root = layout(
        view! {
            <View display=Display::Grid width=20 height=4 grid_auto_flow=GridAutoFlow::Column
                grid_template_rows={[fr(1.0), fr(1.0)]} grid_auto_columns={length(5)}>
                <View /><View /><View />
            </View>
        },
        20,
        4,
    );
    let positions: Vec<_> = root.children.iter().map(|c| (c.rect.x, c.rect.y)).collect();
    assert_eq!(positions, [(0, 0), (0, 2), (5, 0)]);
}

#[test]
fn text_in_a_content_sized_column_is_measured() {
    let node = view! {
        <View display=Display::Grid width=20 grid_template_columns={[max_content(), fr(1.0)]} column_gap=2>
            <TextSpan>"Name"</TextSpan><TextSpan>"Ada"</TextSpan>
            <TextSpan>"Language"</TextSpan><TextSpan>"Rust"</TextSpan>
        </View>
    }
    .into_node();
    let root = layout(node, 20, 2);
    let xs: Vec<_> = root.children.iter().map(|c| (c.rect.x, c.rect.y)).collect();
    assert_eq!(
        xs,
        [(0, 0), (10, 0), (0, 1), (10, 1)],
        "the label column fits its widest label"
    );
}
