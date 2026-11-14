use sveltui::prelude::*;
use sveltui::{
    Buffer,
    element::{get_scroll, measure_node, render_node_interactive},
};

fn render(node: &VNode, width: u16, height: u16) -> (Buffer, HitMap) {
    let mut buffer = Buffer::new(width, height);
    let mut hits = HitMap::new();
    render_node_interactive(
        node,
        sveltui::layout::Rect::new(0, 0, width, height),
        &mut buffer,
        &mut hits,
    );
    (buffer, hits)
}

fn rect(hits: &HitMap, id: FocusId) -> sveltui::layout::Rect {
    hits.targets
        .iter()
        .find(|t| t.focus_id == Some(id))
        .unwrap()
        .rect
}

#[test]
fn intrinsic_dimensions_follow_direction_with_gap_border_and_padding() {
    for (direction, expected) in [
        (FlexDirection::Row, (9, 6)),
        (FlexDirection::Column, (6, 9)),
    ] {
        let node = view! {
            <View flex_direction=direction gap=2 border=BorderStyle::Single padding={Padding::all(1)}>
                <TextSpan>"ab"</TextSpan>
                <TextSpan>"c\nd"</TextSpan>
            </View>
        };
        assert_eq!(measure_node(&node, 100, 100), expected);
    }
}

#[test]
fn justification_and_cross_alignment_work_on_both_axes() {
    for direction in [FlexDirection::Row, FlexDirection::Column] {
        for (justify, positions) in [
            (JustifyContent::FlexStart, [0, 3]),
            (JustifyContent::Center, [3, 6]),
            (JustifyContent::FlexEnd, [5, 8]),
            (JustifyContent::SpaceBetween, [0, 8]),
        ] {
            for (align, cross) in [
                (AlignItems::FlexStart, 0),
                (AlignItems::Center, 2),
                (AlignItems::FlexEnd, 4),
            ] {
                let ids = [FocusId::new(), FocusId::new()];
                let (width, height) = direction.axes(10, 6);
                let (cw, ch) = direction.axes(2, 2);
                let node = view! {
                    <View flex_direction=direction width=width height=height gap=1
                        justify_content=justify align_items=align>
                        <Pressable focus_id={ids[0]} tab_stop=true><View width=cw height=ch /></Pressable>
                        <Pressable focus_id={ids[1]} tab_stop=true><View width=cw height=ch /></Pressable>
                    </View>
                };
                let (_, hits) = render(&node, 20, 20);
                for (id, main) in ids.into_iter().zip(positions) {
                    let (x, y) = direction.axes(main, cross);
                    assert_eq!(rect(&hits, id), sveltui::layout::Rect::new(x, y, cw, ch));
                }
                assert_eq!(hits.focusable_ids(), ids);
            }
        }
    }
}

#[test]
fn flex_growth_shares_remaining_space_and_redistributes_capped_allocations() {
    for direction in [FlexDirection::Row, FlexDirection::Column] {
        let ids = [FocusId::new(), FocusId::new(), FocusId::new()];
        let (width, height) = direction.axes(100, 4);
        let (fixed_w, fixed_h) = direction.axes(Dimension::Length(20.0), Dimension::Auto);
        let (fill_w, fill_h) = direction.axes(Dimension::Auto, Dimension::Auto);
        let (max_w_a, max_h_a) = direction.axes(30, u16::MAX);
        let (max_w_b, max_h_b) = direction.axes(60, u16::MAX);
        let node = view! {
            <View flex_direction=direction width=width height=height>
                <Pressable focus_id={ids[0]} tab_stop=true><View width=fixed_w height=fixed_h flex_shrink=0.0 /></Pressable>
                <Pressable focus_id={ids[1]} tab_stop=true><View width=fill_w height=fill_h flex_grow=1.0 flex_basis=0 max_width=max_w_a max_height=max_h_a /></Pressable>
                <Pressable focus_id={ids[2]} tab_stop=true><View width=fill_w height=fill_h flex_grow=1.0 flex_basis=0 max_width=max_w_b max_height=max_h_b /></Pressable>
            </View>
        };
        let (_, hits) = render(&node, width, height);
        for (id, main) in ids.into_iter().zip([20, 30, 50]) {
            let r = rect(&hits, id);
            assert_eq!(direction.axes(r.width, r.height), (main, 4));
        }
    }
}

#[test]
fn flex_minimums_are_resolved_before_other_child_maximums() {
    let a = FocusId::new();
    let b = FocusId::new();
    let node = view! {
        <View flex_direction=FlexDirection::Row width=100 height=1>
            <Pressable focus_id=a tab_stop=true><View flex_grow=1.0 flex_basis=0 min_width=90 height=1 /></Pressable>
            <Pressable focus_id=b tab_stop=true><View flex_grow=1.0 flex_basis=0 max_width=20 height=1 /></Pressable>
        </View>
    };
    let (_, hits) = render(&node, 100, 2);
    assert_eq!(rect(&hits, a).width, 90);
    assert_eq!(rect(&hits, b).width, 10);
}

#[test]
fn flex_remainders_go_to_earlier_siblings_after_gaps() {
    let ids = [FocusId::new(), FocusId::new(), FocusId::new()];
    let node = view! {
        <View flex_direction=FlexDirection::Row width=10 height=1 gap=1>
            <Pressable focus_id={ids[0]} tab_stop=true><View flex_grow=1.0 flex_basis=0 height=1 /></Pressable>
            <Pressable focus_id={ids[1]} tab_stop=true><View flex_grow=1.0 flex_basis=0 height=1 /></Pressable>
            <Pressable focus_id={ids[2]} tab_stop=true><View flex_grow=1.0 flex_basis=0 height=1 /></Pressable>
        </View>
    };
    let (_, hits) = render(&node, 10, 2);
    for (id, (x, w)) in ids.into_iter().zip([(0, 3), (4, 2), (7, 3)]) {
        assert_eq!(rect(&hits, id), sveltui::layout::Rect::new(x, 0, w, 1));
    }
}

#[test]
fn stretch_respects_explicit_sizes_and_maximums() {
    let a = FocusId::new();
    let b = FocusId::new();
    let c = FocusId::new();
    let node = view! {
        <View width=10 height=6 align_items=AlignItems::Stretch>
            <Pressable focus_id=a tab_stop=true><View height=1 max_width=6><TextSpan>"a"</TextSpan></View></Pressable>
            <Pressable focus_id=b tab_stop=true><View height=1 width=3 /></Pressable>
            <Pressable focus_id=c tab_stop=true><View height=1 width={Dimension::Percent(1.0)} max_width=5 /></Pressable>
        </View>
    };
    let (_, hits) = render(&node, 20, 10);
    assert_eq!(rect(&hits, a).width, 6);
    assert_eq!(rect(&hits, b).width, 3);
    assert_eq!(rect(&hits, c).width, 5);
}

#[test]
fn percentages_and_stretch_use_the_constrained_parent_content_box() {
    let parent = FocusId::new();
    let child = FocusId::new();
    let node = view! {
        <View width=40 border=BorderStyle::Single padding={Padding::all(1)}>
            <Pressable focus_id=parent tab_stop=true>
                <View width={Dimension::Percent(0.5)} border=BorderStyle::Single>
                    <Pressable focus_id=child tab_stop=true><View width={Dimension::Percent(1.0)} height=1 /></Pressable>
                </View>
            </Pressable>
        </View>
    };
    let (_, hits) = render(&node, 100, 20);
    assert_eq!(rect(&hits, parent).width, 18);
    assert_eq!(rect(&hits, child).width, 16);
    assert_eq!(rect(&hits, child).x, 3);
}

#[test]
fn minimum_wins_conflicting_maximum_and_survives_parent_pressure() {
    let id = FocusId::new();
    let node = view! {
        <View width=5 height=2 overflow_x=Overflow::Scroll>
            <Pressable focus_id=id tab_stop=true>
                <View width=4 min_width=12 max_width=8 height=1><TextSpan>"abcdefghijkl"</TextSpan></View>
            </Pressable>
        </View>
    };
    let (_, hits) = render(&node, 20, 5);
    assert_eq!(
        measure_node(
            &view! { <View width=4 min_width=12 max_width=8 height=1 /> },
            5,
            2
        )
        .0,
        12
    );
    assert_eq!(rect(&hits, id).width, 5);
    let scroll = hits.targets.iter().find(|t| t.on_scroll.is_some()).unwrap();
    scroll.on_scroll.as_ref().unwrap()(100, 0);
    assert_eq!(get_scroll(scroll.focus_id.unwrap()).0, 7);
}

#[test]
fn horizontal_flex_scrolls_without_shrinking_fixed_children() {
    let id = FocusId::new();
    let node = view! {
        <View width=10 height=3 flex_direction=FlexDirection::Row
            overflow_x=Overflow::Scroll scroll_x=6 scroll_id=id>
            <View width=8 height=1 flex_shrink=0.0><TextSpan>"ABCDEFGH"</TextSpan></View>
            <View width=8 height=1 flex_shrink=0.0><TextSpan>"ijklmnop"</TextSpan></View>
        </View>
    };
    let (buffer, _) = render(&node, 20, 5);
    let line: String = (0..10).map(|x| buffer.get(x, 0).unwrap().ch).collect();
    assert_eq!(line, "GHijklmnop");
    assert_eq!(get_scroll(id), (6, 0));
    assert_eq!(buffer.get(0, 2).unwrap().ch, '▄');
}

#[test]
fn vertical_scrollbar_reduces_fill_width_without_causing_horizontal_scroll() {
    let child = FocusId::new();
    let node = view! {
        <View width=10 height=3 overflow=Overflow::Scroll>
            <Pressable focus_id=child tab_stop=true><View height=5 flex_shrink=0.0 /></Pressable>
        </View>
    };
    let (buffer, hits) = render(&node, 20, 10);
    assert_eq!(rect(&hits, child).width, 9);
    assert_eq!(buffer.get(9, 0).unwrap().ch, '█');
    assert_ne!(buffer.get(0, 2).unwrap().ch, '▄');
}

#[test]
fn coupled_scrollbars_and_tiny_viewports_do_not_panic() {
    for (width, height) in [(10, 3), (1, 1), (0, 0)] {
        let node = view! {
            <View width=width height=height overflow=Overflow::Scroll>
                <View width=10 height=4 flex_shrink=0.0 />
            </View>
        };
        let (buffer, _) = render(&node, width, height);
        if width == 10 {
            assert_eq!(buffer.get(9, 0).unwrap().ch, '█');
            assert_eq!(buffer.get(0, 2).unwrap().ch, '▄');
        }
    }
}

#[test]
fn overlay_trigger_keeps_its_allocated_percentage_size_and_anchor() {
    let trigger = FocusId::new();
    let popup = FocusId::new();
    let node = view! {
        <View width=40 height=4 flex_direction=FlexDirection::Row>
            <View width=10 height=2 />
            <Overlay open=true placement=Placement::BottomStart
                trigger={view! { <Pressable focus_id=trigger tab_stop=true><View width={Dimension::Percent(0.5)} height=2 /></Pressable> }}>
                <Pressable focus_id=popup tab_stop=true><View width=5 height=1><TextSpan>"popup"</TextSpan></View></Pressable>
            </Overlay>
        </View>
    };
    let (_, hits) = render(&node, 80, 24);
    assert_eq!(
        rect(&hits, trigger),
        sveltui::layout::Rect::new(10, 0, 20, 2)
    );
    assert_eq!(rect(&hits, popup), sveltui::layout::Rect::new(10, 2, 5, 1));
}

#[test]
fn empty_and_fragment_children_do_not_add_phantom_gaps() {
    let id = FocusId::new();
    let node = view! {
        <View flex_direction=FlexDirection::Row gap=2>
            {None::<VNode>}
            {vec![vec![view! { <TextSpan>"ab"</TextSpan> }]]}
            <Overlay open=false />
            <Pressable focus_id=id tab_stop=true><View width=1 height=1><TextSpan>"c"</TextSpan></View></Pressable>
        </View>
    };
    assert_eq!(measure_node(&node, 80, 24), (5, 1));
    let (buffer, hits) = render(&node, 80, 24);
    assert_eq!(rect(&hits, id).x, 4);
    assert_eq!(buffer.get(4, 0).unwrap().ch, 'c');
}

#[test]
fn scrolled_hit_targets_match_visible_cells_and_keep_local_click_coordinates() {
    use std::cell::Cell;
    use std::rc::Rc;
    let first = FocusId::new();
    let second = FocusId::new();
    let clicked = Rc::new(Cell::new(None));
    let received = clicked.clone();
    let received_at = received.clone();
    let child = view! {
        <Pressable focus_id=first tab_stop=true on_click_at={move |x, y| received_at.set(Some((x, y)))}>
            <View width=8 height=1 flex_shrink=0.0><TextSpan>"ABCDEFGH"</TextSpan></View>
        </Pressable>
    };
    let node = view! {
        <View width=10 height=3 flex_direction=FlexDirection::Row overflow_x=Overflow::Scroll scroll_x=6>
            {child}
            <Pressable focus_id=second tab_stop=true><View width=8 height=1 flex_shrink=0.0><TextSpan>"ijklmnop"</TextSpan></View></Pressable>
        </View>
    };
    let (_, hits) = render(&node, 20, 5);
    assert_eq!(rect(&hits, first), sveltui::layout::Rect::new(0, 0, 2, 1));
    assert_eq!(rect(&hits, second), sveltui::layout::Rect::new(2, 0, 8, 1));
    hits.hit_test(0, 0).unwrap().on_click_at.as_ref().unwrap()(0, 0);
    assert_eq!(clicked.get(), Some((6, 0)));
    assert_ne!(hits.hit_test(10, 0).and_then(|t| t.focus_id), Some(second));
}

#[test]
fn space_between_handles_empty_singleton_and_excess_content() {
    for count in 0..=2 {
        let ids: Vec<_> = (0..count).map(|_| FocusId::new()).collect();
        let children = ids
            .iter()
            .copied()
            .map(|id| view! { <Pressable focus_id=id tab_stop=true><View width=6 height=1 /></Pressable> })
            .collect::<Vec<_>>();
        let node = view! {
            <View width=10 height=1 flex_direction=FlexDirection::Row justify_content=JustifyContent::SpaceBetween>
                {children}
            </View>
        };
        let (_, hits) = render(&node, 30, 4);
        if count > 0 {
            assert_eq!(rect(&hits, ids[0]).x, 0);
        }
    }
}

#[test]
fn percentage_full_width_is_distinct_from_sharing_remaining_space() {
    let percentage = FocusId::new();
    let fill = FocusId::new();
    let node = view! {
        <View flex_direction=FlexDirection::Row width=20 height=1>
            <Pressable focus_id=percentage tab_stop=true><View width={Dimension::Percent(1.0)} height=1 flex_shrink=0.0 /></Pressable>
            <Pressable focus_id=fill tab_stop=true><View flex_grow=1.0 flex_basis=0 min_width=3 height=1 /></Pressable>
        </View>
    };
    let (_, hits) = render(&node, 30, 4);
    assert_eq!(rect(&hits, percentage).width, 20);
    assert_eq!(rect(&hits, fill), sveltui::layout::Rect::new(20, 0, 3, 1));
}

#[test]
fn one_overflow_axis_does_not_clip_visible_overflow_on_the_other() {
    let node = view! {
        <View width=2 height=1 overflow_y=Overflow::Hidden>
            <TextSpan>"ABCDE\nsecond"</TextSpan>
        </View>
    };
    let (buffer, _) = render(&node, 10, 5);
    assert_eq!(buffer.get(4, 0).unwrap().ch, 'E');
    assert_eq!(buffer.get(0, 1).unwrap().ch, ' ');
}

#[test]
fn auto_cross_size_does_not_reserve_scrollbars_for_flex_siblings() {
    for direction in [FlexDirection::Row, FlexDirection::Column] {
        let (width, height) = direction.axes(Dimension::Length(20.0), Dimension::Auto);
        let (child_w, child_h) = direction.axes(Dimension::Auto, Dimension::Length(6.0));
        let node = view! {
            <View flex_direction=direction width=width height=height overflow=Overflow::Scroll>
                <View width=child_w height=child_h flex_grow=1.0 flex_basis=0 />
                <View width=child_w height=child_h flex_grow=1.0 flex_basis=0 />
            </View>
        };
        assert_eq!(measure_node(&node, 80, 80), direction.axes(20, 6));
    }
}
