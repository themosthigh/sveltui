use super::taffy_bridge::{LAST_CHANGES, LayoutEngine};
use crate::element::VNode;
use crate::prelude::*;
use crate::view;

fn list(rows: &[&str], height: u16) -> VNode {
    let rows: Vec<String> = rows.iter().map(|r| r.to_string()).collect();
    view! {
        <View width=20 height={height} overflow_y=Overflow::Scroll>
            {#each rows as row}
                <Paragraph>{row}</Paragraph>
            {/each}
        </View>
    }
}

fn strs(rows: &[String]) -> Vec<&str> {
    rows.iter().map(String::as_str).collect()
}

fn changes() -> usize {
    LAST_CHANGES.with(|changes| changes.get())
}

/// Lays `node` out with the engine we keep between frames and with a fresh
/// one, and checks they agree on every size and position.
fn assert_same(kept: &mut LayoutEngine, node: &VNode, width: u16, height: u16) {
    let retained = kept.compute_layout(node, width, height);
    let fresh = LayoutEngine::new().compute_layout(node, width, height);
    assert_eq!(format!("{retained:?}"), format!("{fresh:?}"));
}

#[test]
fn a_kept_engine_lays_out_like_a_fresh_one_through_edits() {
    let mut engine = LayoutEngine::new();
    let short = ["one", "two", "three", "four"];
    assert_same(&mut engine, &list(&short, 3), 30, 10);
    // Content changes: one row now wraps onto several lines.
    let wrapped = [
        "one",
        "two two two two two two two two two two",
        "three",
        "four",
    ];
    assert_same(&mut engine, &list(&wrapped, 3), 30, 10);
    // Rows come and go.
    assert_same(&mut engine, &list(&wrapped[1..], 3), 30, 10);
    let more = ["a", "b", "c", "d", "e", "f", "g", "h"];
    assert_same(&mut engine, &list(&more, 3), 30, 10);
    // The viewport grows and shrinks, and so does the screen.
    assert_same(&mut engine, &list(&more, 6), 30, 10);
    assert_same(&mut engine, &list(&more, 2), 12, 4);
    assert_same(&mut engine, &list(&wrapped, 3), 8, 4);
    // A different kind of node takes a row's place.
    let swapped = view! {
        <View width=20 height=3 overflow_y=Overflow::Scroll>
            <Paragraph>"first"</Paragraph>
            <View><TextSpan>"nested"</TextSpan></View>
            <Paragraph>"last"</Paragraph>
        </View>
    };
    assert_same(&mut engine, &swapped, 30, 10);
    assert_same(&mut engine, &list(&short, 3), 30, 10);
    assert_same(&mut engine, &VNode::Empty, 30, 10);
}

#[test]
fn a_row_that_scrolls_into_view_has_its_own_size_and_place() {
    let rows = [
        "one",
        "two",
        "three wraps over lines because it is long",
        "four",
    ];
    let mut engine = LayoutEngine::new();
    let first = engine.compute_layout(&list(&rows, 2), 30, 10);
    // The third row sits below the two-line viewport but still has a place.
    let third = &first.children[2];
    assert_eq!(third.rect.y, 2);
    assert!(third.rect.height > 1);
    // The content is taller than the viewport, so it can scroll to it.
    assert!(first.scroll_size.1 > 0);
    // After an edit elsewhere the row is where it was.
    let edited = ["ONE", "two", rows[2], "four"];
    let second = engine.compute_layout(&list(&edited, 2), 30, 10);
    assert_eq!(second.children[2].rect, third.rect);
    assert_eq!(second.scroll_size, first.scroll_size);
}

#[test]
fn a_frame_that_changed_nothing_tells_taffy_nothing() {
    let rows: Vec<String> = (0..50).map(|n| format!("row {n}")).collect();
    let rows: Vec<&str> = rows.iter().map(String::as_str).collect();
    let mut engine = LayoutEngine::new();
    engine.compute_layout(&list(&rows, 5), 30, 10);
    assert!(changes() > 50, "the first frame builds every node");
    engine.compute_layout(&list(&rows, 5), 30, 10);
    assert_eq!(changes(), 0);
}

#[test]
fn editing_one_row_touches_one_row() {
    let rows: Vec<String> = (0..50).map(|n| format!("row {n}")).collect();
    let mut edited = rows.clone();
    edited[20] = "changed".to_string();
    let mut engine = LayoutEngine::new();
    engine.compute_layout(&list(&strs(&rows), 5), 30, 10);
    engine.compute_layout(&list(&strs(&edited), 5), 30, 10);
    assert_eq!(changes(), 1);
}

#[test]
fn nodes_are_not_kept_beyond_the_tree() {
    let mut engine = LayoutEngine::new();
    let big = ["a", "b", "c", "d", "e"];
    engine.compute_layout(&list(&big, 3), 30, 10);
    engine.compute_layout(&list(&big[..1], 3), 30, 10);
    // Growing again reuses the freed text slots and lays out correctly.
    assert_same(&mut engine, &list(&big, 3), 30, 10);
}
