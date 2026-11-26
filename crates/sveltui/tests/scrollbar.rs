use sveltui::{
    Buffer,
    element::{get_scroll, render_node_interactive, set_scroll},
    layout::Rect,
    prelude::*,
};

fn render(node: &VNode, width: u16, height: u16) -> (Buffer, HitMap) {
    let mut buffer = Buffer::new(width, height);
    let mut hits = HitMap::new();
    render_node_interactive(node, Rect::new(0, 0, width, height), &mut buffer, &mut hits);
    (buffer, hits)
}

#[test]
fn vertical_thumb_is_proportional_and_reaches_both_track_ends() {
    let id = FocusId::new();
    let node = view! {
        <View width=12 height=10 overflow_y=Overflow::Scroll scroll_id=id>
            {#each (0..20) as i}<TextSpan>{format!("line {i:02}")}</TextSpan>{/each}
        </View>
    };
    for (offset, start) in [(0, 0), (5, 2), (10, 5)] {
        set_scroll(id, (0, offset));
        let (buffer, _) = render(&node, 12, 10);
        let thumb: Vec<_> = (0..10)
            .filter(|&y| buffer.get(11, y).unwrap().bold)
            .collect();
        assert_eq!(thumb, (start..start + 5).collect::<Vec<_>>());
    }
}

#[test]
fn horizontal_thumb_is_proportional_and_reaches_both_track_ends() {
    let id = FocusId::new();
    let node = view! {
        <View width=10 height=3 overflow_x=Overflow::Scroll scroll_id=id>
            <TextSpan>"0123456789abcdefghij"</TextSpan>
        </View>
    };
    for (offset, start) in [(0, 0), (5, 2), (10, 5)] {
        set_scroll(id, (offset, 0));
        let (buffer, _) = render(&node, 10, 3);
        let thumb: Vec<_> = (0..10)
            .filter(|&x| buffer.get(x, 2).unwrap().bold)
            .collect();
        assert_eq!(thumb, (start..start + 5).collect::<Vec<_>>());
    }
}

#[test]
fn both_tracks_leave_the_last_content_row_and_column_visible_at_the_end() {
    let id = FocusId::new();
    let node = view! {
        <View width=12 height=7 border=BorderStyle::Single padding=1
            overflow=Overflow::Scroll scroll_id=id>
            {#each (0..6) as i}<TextSpan>{format!("{i}abcdefghijkZ")}</TextSpan>{/each}
        </View>
    };
    let (_, hits) = render(&node, 12, 7);
    let target = hits.targets.iter().find(|t| t.on_scroll.is_some()).unwrap();
    target.on_scroll.as_ref().unwrap()(i16::MAX, i16::MAX);
    let (buffer, _) = render(&node, 12, 7);
    // Seven content columns and two content rows remain after padding and
    // tracks. The tracks sit just inside the border, past the padding.
    assert_eq!(get_scroll(id), (6, 4));
    assert_eq!(buffer.get(8, 3).unwrap().ch, 'Z');
    assert_eq!(
        buffer.get(10, 5).unwrap().ch,
        ScrollbarChars::default().corner
    );
    assert_eq!(buffer.get(10, 4).unwrap().fg, Some(Color::White));
    assert_eq!(buffer.get(9, 5).unwrap().fg, Some(Color::White));
}

#[test]
fn content_that_fits_a_padded_view_does_not_scroll_into_blank_space() {
    let id = FocusId::new();
    let node = view! {
        <View width=12 height=7 border=BorderStyle::Single padding=1
            overflow=Overflow::Scroll scroll_id=id>
            <TextSpan>"first"</TextSpan><TextSpan>"last"</TextSpan>
        </View>
    };
    let (_, hits) = render(&node, 12, 7);
    let target = hits.targets.iter().find(|t| t.on_scroll.is_some()).unwrap();
    target.on_scroll.as_ref().unwrap()(i16::MAX, i16::MAX);
    assert_eq!(get_scroll(id), (0, 0));
    let (buffer, _) = render(&node, 12, 7);
    assert_eq!(buffer.get(2, 2).unwrap().ch, 'f');
    assert_eq!(buffer.get(2, 3).unwrap().ch, 'l');
    assert!(!buffer.cells.iter().any(|c| matches!(c.ch, '█' | '▄')));
}

#[test]
fn long_scroll_ranges_do_not_overflow_thumb_arithmetic() {
    let id = FocusId::new();
    let node = view! {
        <View width=4 height=100 overflow_y=Overflow::Scroll scroll_id=id>
            <View width=1 height=60000 flex_shrink=0.0 />
        </View>
    };
    set_scroll(id, (0, 59900));
    let (buffer, _) = render(&node, 4, 100);
    assert_eq!(buffer.get(3, 99).unwrap().fg, Some(Color::White));
}

#[test]
fn hidden_scrollbars_keep_the_full_content_viewport() {
    let id = FocusId::new();
    let child = FocusId::new();
    let node = view! {
        <View width=10 height=3 overflow=Overflow::Scroll scroll_id=id
            scrollbar_visibility=ScrollbarVisibility::Never>
            <Pressable focus_id=child><View height=6 flex_shrink=0.0 /></Pressable>
        </View>
    };
    let (buffer, hits) = render(&node, 10, 3);
    assert_eq!(hits.layouts[&child].width, 10);
    assert!(!buffer.cells.iter().any(|c| matches!(c.ch, '█' | '▄')));
}

/// A bordered, padded view of 40 lines: 10 cells wide and 12 tall, so the
/// box inside the border is rows 1 to 10, and the vertical bar is column 8.
fn padded_list(id: FocusId) -> VNode {
    view! {
        <View width=10 height=12 border=BorderStyle::Single padding=1
            overflow_y=Overflow::Scroll scroll_id=id>
            {#each (0..40) as i}<TextSpan>{format!("{i:02}")}</TextSpan>{/each}
        </View>
    }
    .into_node()
}

fn track(buffer: &Buffer, x: u16) -> Vec<u16> {
    (0..12)
        .filter(|&y| matches!(buffer.get(x, y).unwrap().ch, '█'))
        .collect()
}

#[test]
fn padding_insets_the_content_but_not_the_scrollbar() {
    let id = FocusId::new();
    let node = padded_list(id);
    set_scroll(id, (0, 0));
    let (buffer, _) = render(&node, 10, 12);
    // The track runs the whole height inside the border, in the column just
    // inside it, with the padding column between it and the content.
    assert_eq!(track(&buffer, 8), (1..=10).collect::<Vec<_>>());
    assert!(track(&buffer, 7).is_empty());
    assert!(
        buffer.get(8, 1).unwrap().bold,
        "the thumb starts at the top"
    );
    assert_eq!(buffer.get(2, 2).unwrap().ch, '0');

    set_scroll(id, (0, u16::MAX));
    let (buffer, _) = render(&node, 10, 12);
    assert!(buffer.get(8, 10).unwrap().bold, "and ends at the bottom");
}

#[test]
fn pressing_the_track_pages_towards_the_press() {
    let id = FocusId::new();
    let node = padded_list(id);
    set_scroll(id, (0, 0));
    let (_, hits) = render(&node, 10, 12);
    let mut manager = FocusManager::new();

    // Eight rows show at a time; the thumb is the top two cells.
    manager.pointer_down_at(&hits, 8, 9);
    manager.pointer_up(&hits, 8, 9);
    assert_eq!(get_scroll(id), (0, 8));

    let (_, hits) = render(&node, 10, 12);
    manager.pointer_down_at(&hits, 8, 1);
    manager.pointer_up(&hits, 8, 1);
    assert_eq!(get_scroll(id), (0, 0));
}

#[test]
fn dragging_the_thumb_keeps_it_under_the_pointer() {
    let id = FocusId::new();
    let node = padded_list(id);
    set_scroll(id, (0, 0));
    let (_, hits) = render(&node, 10, 12);
    let mut manager = FocusManager::new();

    // Grab the thumb by its second cell; a press on the thumb doesn't scroll.
    manager.pointer_down_at(&hits, 8, 2);
    assert!(manager.pointer_captured());
    assert_eq!(get_scroll(id), (0, 0));

    // Four cells down: the thumb's top moves to the fifth track cell.
    manager.pointer_drag(&hits, 8, 6);
    assert_eq!(get_scroll(id), (0, 16));
    let (buffer, hits) = render(&node, 10, 12);
    let thumb: Vec<_> = (1..=10)
        .filter(|&y| buffer.get(8, y).unwrap().bold)
        .collect();
    assert_eq!(thumb, [5, 6]);

    // Past the end, and even off the bar, it stops at the bottom.
    manager.pointer_drag(&hits, 30, 50);
    assert_eq!(get_scroll(id), (0, 32));
    manager.pointer_drag(&hits, 0, 0);
    assert_eq!(get_scroll(id), (0, 0));
    manager.pointer_up(&hits, 0, 0);
    assert!(!manager.pointer_captured());

    // Once released, dragging does nothing.
    assert!(manager.pointer_drag(&hits, 8, 9).is_none());
    assert_eq!(get_scroll(id), (0, 0));
}

#[test]
fn the_horizontal_thumb_drags_along_its_row() {
    let id = FocusId::new();
    let node = view! {
        <View width=12 height=4 border=BorderStyle::Single padding={Padding::symmetric(1, 0)}
            overflow_x=Overflow::Scroll scroll_id=id>
            <TextSpan>"0123456789abcdefghijklmnopqrst"</TextSpan>
        </View>
    }
    .into_node();
    set_scroll(id, (0, 0));
    let (buffer, hits) = render(&node, 12, 4);
    // The bar spans the row inside the border, padding and all.
    let bar: Vec<_> = (0..12)
        .filter(|&x| buffer.get(x, 2).unwrap().ch == ScrollbarChars::default().horizontal_track)
        .collect();
    assert_eq!(bar, (1..=10).collect::<Vec<_>>());

    let mut manager = FocusManager::new();
    manager.pointer_down_at(&hits, 1, 2);
    manager.pointer_drag(&hits, 40, 2);
    manager.pointer_up(&hits, 40, 2);
    assert_eq!(get_scroll(id), (22, 0));
}

#[test]
fn the_wheel_over_the_bar_still_scrolls() {
    let id = FocusId::new();
    let node = padded_list(id);
    set_scroll(id, (0, 0));
    let (_, hits) = render(&node, 10, 12);
    let target = hits.scroll_target(8, 5).unwrap();
    target.on_scroll.as_ref().unwrap()(0, 3);
    assert_eq!(get_scroll(id), (0, 3));
}

/// A scroll view sized by flex has no size of its own, but its scrollbars
/// still take cells, so the last row and column must stay reachable.
#[test]
fn flex_sized_scroll_view_reaches_the_end_of_its_content() {
    let id = FocusId::new();
    let lines = (0..20).map(|i| format!("{i:02}{}", "x".repeat(21)) + "z");
    let node = view! {
        <View width=12 height=8 flex_direction=FlexDirection::Column>
            <View
                flex_grow=1.0
                border=BorderStyle::Single
                overflow=Overflow::Scroll
                scroll_id=id
            >
                {#each lines as line}<TextSpan>{line}</TextSpan>{/each}
            </View>
        </View>
    };
    let (_, hits) = render(&node, 12, 8);
    let target = hits.targets.iter().find(|t| t.on_scroll.is_some()).unwrap();
    target.on_scroll.as_ref().unwrap()(i16::MAX, i16::MAX);
    // 24 columns and 20 rows in a view with a border and two bars: the
    // viewport is 9x6, so the furthest offset is (24 - 9, 20 - 6) plus the
    // cell the bars take from each axis.
    assert_eq!(get_scroll(id), (15, 15));
    let (buffer, _) = render(&node, 12, 8);
    let shown: String = (0..8)
        .flat_map(|y| (0..12).map(move |x| (x, y)))
        .map(|(x, y)| buffer.get(x, y).unwrap().ch)
        .collect();
    assert!(
        shown.contains('z'),
        "the last column is hidden under the scrollbar: {shown:?}"
    );
}

/// Padding insets the content only. The bars of a flex-sized view sit just
/// inside the border like those of a fixed one, and the content still ends
/// at the same place.
#[test]
fn padding_does_not_move_the_bars_of_a_flex_sized_view() {
    let id = FocusId::new();
    let node = view! {
        <View width=12 height=7 flex_direction=FlexDirection::Column>
            <View
                flex_grow=1.0
                border=BorderStyle::Single
                padding=1
                overflow=Overflow::Scroll
                scroll_id=id
            >
                {#each (0..6) as i}<TextSpan>{format!("{i}abcdefghijkZ")}</TextSpan>{/each}
            </View>
        </View>
    };
    let (_, hits) = render(&node, 12, 7);
    let target = hits.targets.iter().find(|t| t.on_scroll.is_some()).unwrap();
    target.on_scroll.as_ref().unwrap()(i16::MAX, i16::MAX);
    let (buffer, _) = render(&node, 12, 7);
    // The same numbers as the fixed-size view with this border and padding.
    assert_eq!(get_scroll(id), (6, 4));
    assert_eq!(buffer.get(8, 3).unwrap().ch, 'Z');
    let chars = ScrollbarChars::default();
    // The vertical bar fills the column inside the right border, and the
    // horizontal one the row inside the bottom border, padding or not.
    assert_eq!(buffer.get(10, 5).unwrap().ch, chars.corner);
    for y in 1..5 {
        let ch = buffer.get(10, y).unwrap().ch;
        assert!(
            ch == chars.vertical_thumb || ch == chars.vertical_track,
            "no vertical bar at row {y}: {ch:?}"
        );
    }
    for x in 1..10 {
        let ch = buffer.get(x, 5).unwrap().ch;
        assert!(
            ch == chars.horizontal_thumb || ch == chars.horizontal_track,
            "no horizontal bar at column {x}: {ch:?}"
        );
    }
}
