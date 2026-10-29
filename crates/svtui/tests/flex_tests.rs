use svtui::prelude::*;
use svtui::{
    Buffer,
    element::{get_scroll, measure_node, render_node_interactive},
};
use svtui_components::*;

fn render(node: &VNode, width: u16, height: u16) -> (Buffer, HitMap) {
    let mut buffer = Buffer::new(width, height);
    let mut hits = HitMap::new();
    render_node_interactive(
        node,
        svtui::layout::Rect::new(0, 0, width, height),
        &mut buffer,
        &mut hits,
    );
    (buffer, hits)
}

fn rect(hits: &HitMap, id: FocusId) -> svtui::layout::Rect {
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
            <Box flex_direction=direction gap=2 border=BorderStyle::Single padding=1>
                <Text>"ab"</Text>
                <Text>"c\nd"</Text>
            </Box>
        };
        assert_eq!(measure_node(&node, 100, 100), expected);
    }
}

#[test]
fn justification_and_cross_alignment_work_on_both_axes() {
    for direction in [FlexDirection::Row, FlexDirection::Column] {
        for (justify, positions) in [
            (Justify::Start, [0, 3]),
            (Justify::Center, [2, 5]),
            (Justify::End, [5, 8]),
            (Justify::SpaceBetween, [0, 8]),
        ] {
            for (align, cross) in [(Align::Start, 0), (Align::Center, 2), (Align::End, 4)] {
                let ids = [FocusId::new(), FocusId::new()];
                let (width, height) = direction.axes(10, 6);
                let (cw, ch) = direction.axes(2, 2);
                let node = view! {
                    <Box flex_direction=direction width=width height=height gap=1
                        justify_content=justify align_items=align>
                        <Box focus_id={ids[0]} tab_stop=true width=cw height=ch />
                        <Box focus_id={ids[1]} tab_stop=true width=cw height=ch />
                    </Box>
                };
                let (_, hits) = render(&node, 20, 20);
                for (id, main) in ids.into_iter().zip(positions) {
                    let (x, y) = direction.axes(main, cross);
                    assert_eq!(rect(&hits, id), svtui::layout::Rect::new(x, y, cw, ch));
                }
                assert_eq!(hits.focusable_ids(), ids);
            }
        }
    }
}

#[test]
fn fill_shares_remaining_space_and_redistributes_capped_allocations() {
    for direction in [FlexDirection::Row, FlexDirection::Column] {
        let ids = [FocusId::new(), FocusId::new(), FocusId::new()];
        let (width, height) = direction.axes(100, 4);
        let (fixed_w, fixed_h) = direction.axes(Size::Fixed(20), Size::Fill);
        let (fill_w, fill_h) = direction.axes(Size::Fill, Size::Fill);
        let (max_w_a, max_h_a) = direction.axes(30, u16::MAX);
        let (max_w_b, max_h_b) = direction.axes(60, u16::MAX);
        let node = view! {
            <Box flex_direction=direction width=width height=height>
                <Box focus_id={ids[0]} tab_stop=true width=fixed_w height=fixed_h />
                <Box focus_id={ids[1]} tab_stop=true width=fill_w height=fill_h max_width=max_w_a max_height=max_h_a />
                <Box focus_id={ids[2]} tab_stop=true width=fill_w height=fill_h max_width=max_w_b max_height=max_h_b />
            </Box>
        };
        let (_, hits) = render(&node, width, height);
        for (id, main) in ids.into_iter().zip([20, 30, 50]) {
            let r = rect(&hits, id);
            assert_eq!(direction.axes(r.width, r.height), (main, 4));
        }
    }
}

#[test]
fn fill_minimums_are_resolved_before_other_child_maximums() {
    let a = FocusId::new();
    let b = FocusId::new();
    let node = view! {
        <Box flex_direction=FlexDirection::Row width=100 height=1>
            <Box width=Size::Fill min_width=90 height=1 focus_id=a tab_stop=true />
            <Box width=Size::Fill max_width=20 height=1 focus_id=b tab_stop=true />
        </Box>
    };
    let (_, hits) = render(&node, 100, 2);
    assert_eq!(rect(&hits, a).width, 90);
    assert_eq!(rect(&hits, b).width, 10);
}

#[test]
fn fill_remainders_go_to_earlier_siblings_after_gaps() {
    let ids = [FocusId::new(), FocusId::new(), FocusId::new()];
    let node = view! {
        <Box flex_direction=FlexDirection::Row width=10 height=1 gap=1>
            <Box width=Size::Fill height=1 focus_id={ids[0]} tab_stop=true />
            <Box width=Size::Fill height=1 focus_id={ids[1]} tab_stop=true />
            <Box width=Size::Fill height=1 focus_id={ids[2]} tab_stop=true />
        </Box>
    };
    let (_, hits) = render(&node, 10, 2);
    for (id, (x, w)) in ids.into_iter().zip([(0, 3), (4, 3), (8, 2)]) {
        assert_eq!(rect(&hits, id), svtui::layout::Rect::new(x, 0, w, 1));
    }
}

#[test]
fn stretch_respects_explicit_sizes_and_maximums() {
    let a = FocusId::new();
    let b = FocusId::new();
    let c = FocusId::new();
    let node = view! {
        <Box width=10 height=6 align_items=Align::Stretch>
            <Box height=1 max_width=6 focus_id=a tab_stop=true><Text>"a"</Text></Box>
            <Box height=1 width=3 focus_id=b tab_stop=true />
            <Box height=1 width=Size::Fill max_width=5 focus_id=c tab_stop=true />
        </Box>
    };
    let (_, hits) = render(&node, 20, 10);
    assert_eq!(rect(&hits, a).width, 6);
    assert_eq!(rect(&hits, b).width, 3);
    assert_eq!(rect(&hits, c).width, 5);
}

#[test]
fn percentage_and_fill_use_the_constrained_parent_content_box() {
    let parent = FocusId::new();
    let child = FocusId::new();
    let node = view! {
        <Box width=40 border=BorderStyle::Single padding=1>
            <Box width=Size::Percent(50.0) border=BorderStyle::Single focus_id=parent tab_stop=true>
                <Box width=Size::Fill height=1 focus_id=child tab_stop=true />
            </Box>
        </Box>
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
        <Box width=5 height=2 overflow_x=Overflow::Scroll>
            <Box width=4 min_width=12 max_width=8 height=1 focus_id=id tab_stop=true><Text>"abcdefghijkl"</Text></Box>
        </Box>
    };
    let (_, hits) = render(&node, 20, 5);
    assert_eq!(
        measure_node(
            &view! { <Box width=4 min_width=12 max_width=8 height=1 /> },
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
        <Box width=10 height=3 flex_direction=FlexDirection::Row
            overflow_x=Overflow::Scroll scroll_x=6 focus_id=id tab_stop=true>
            <Box width=8 height=1><Text>"ABCDEFGH"</Text></Box>
            <Box width=8 height=1><Text>"ijklmnop"</Text></Box>
        </Box>
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
        <Box width=10 height=3 overflow=Overflow::Auto>
            <Box width=Size::Fill height=5 focus_id=child tab_stop=true />
        </Box>
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
            <Box width=width height=height overflow=Overflow::Auto>
                <Box width=10 height=4 />
            </Box>
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
        <Box width=40 height=4 flex_direction=FlexDirection::Row>
            <Box width=10 height=2 />
            <Overlay open=true placement=Placement::BottomStart
                trigger={view! { <Box width=Size::Percent(50.0) height=2 focus_id=trigger tab_stop=true /> }}>
                <Box width=5 height=1 focus_id=popup tab_stop=true><Text>"popup"</Text></Box>
            </Overlay>
        </Box>
    };
    let (_, hits) = render(&node, 80, 24);
    assert_eq!(rect(&hits, trigger), svtui::layout::Rect::new(10, 0, 20, 2));
    assert_eq!(rect(&hits, popup), svtui::layout::Rect::new(10, 2, 5, 1));
}

#[test]
fn empty_and_fragment_children_do_not_add_phantom_gaps() {
    let id = FocusId::new();
    let node = view! {
        <Box flex_direction=FlexDirection::Row gap=2>
            {None::<VNode>}
            {vec![vec![view! { <Text>"ab"</Text> }]]}
            <Overlay open=false />
            <Box width=1 height=1 focus_id=id tab_stop=true><Text>"c"</Text></Box>
        </Box>
    };
    assert_eq!(measure_node(&node, 80, 24), (5, 1));
    let (buffer, hits) = render(&node, 80, 24);
    assert_eq!(rect(&hits, id).x, 4);
    assert_eq!(buffer.get(4, 0).unwrap().ch, 'c');
}

#[test]
fn cursor_and_focus_ancestry_follow_nested_row_layout() {
    let parent = FocusId::new();
    let input = FocusId::new();
    let (value, set_value) = create_signal(String::from("abc"));
    let build = || {
        view! {
            <Box flex_direction=FlexDirection::Row width=30 height=5 focus_id=parent tab_stop=true gap=1>
                <Box width=4 height=3 />
                <TextInput value=value set_value=set_value width=Size::Fill id=input />
            </Box>
        }
    };
    let initial = build();
    let mut manager = FocusManager::new();
    manager.reconcile(InteractionTree::from_node(&initial));
    manager.focus(input);
    let node = manager.with_context(build);

    let (buffer, hits) = manager.with_context(|| render(&node, 40, 10));
    let target = rect(&hits, input);
    assert_eq!(target.x, 5);
    assert_eq!(target.width, 25);
    assert!(
        buffer
            .cursor()
            .is_some_and(|(x, y)| (6..30).contains(&x) && y == 1)
    );
    assert!(hits.ancestors.get(&input).unwrap().contains(&parent));
    assert_eq!(hits.focusable_ids(), vec![parent, input]);
}

#[test]
fn scrolled_hit_targets_match_visible_cells_and_keep_local_click_coordinates() {
    use std::cell::Cell;
    use std::rc::Rc;
    let first = FocusId::new();
    let second = FocusId::new();
    let clicked = Rc::new(Cell::new(None));
    let received = clicked.clone();
    // on_click_at is a two-argument builder; use it on a JSX-created box.
    let mut child =
        view! { <Box width=8 height=1 focus_id=first tab_stop=true><Text>"ABCDEFGH"</Text></Box> };
    if let VNode::Box(b) = &mut child {
        b.on_click_at = Some(Rc::new(move |x, y| received.set(Some((x, y)))));
    }
    let node = view! {
        <Box width=10 height=3 flex_direction=FlexDirection::Row overflow_x=Overflow::Scroll scroll_x=6>
            {child}
            <Box width=8 height=1 focus_id=second tab_stop=true><Text>"ijklmnop"</Text></Box>
        </Box>
    };
    let (_, hits) = render(&node, 20, 5);
    assert_eq!(rect(&hits, first), svtui::layout::Rect::new(0, 0, 2, 1));
    assert_eq!(rect(&hits, second), svtui::layout::Rect::new(2, 0, 8, 1));
    hits.hit_test(0, 0).unwrap().on_click_at.as_ref().unwrap()(0, 0);
    assert_eq!(clicked.get(), Some((6, 0)));
    assert_ne!(hits.hit_test(10, 0).and_then(|t| t.focus_id), Some(second));
}

#[test]
fn dropdown_submenu_label_and_arrow_share_one_full_width_row() {
    let submenu = FocusId::new();
    let leaf = FocusId::new();
    let build = || {
        view! {
            <DropdownMenu width=24 open=true placement={Placement::Anchor { x: 0, y: 0 }}>
                <DropdownSubmenu label="Export a very long document label" id=submenu>
                    <DropdownItem label="PDF" />
                </DropdownSubmenu>
                <DropdownItem id=leaf label="Close" />
            </DropdownMenu>
        }
    };
    let initial = build();
    let mut manager = FocusManager::new();
    manager.reconcile(InteractionTree::from_node(&initial));
    manager.focus(submenu);
    let node = manager.with_context(build);

    let (buffer, hits) = manager.with_context(|| render(&node, 80, 24));
    assert_eq!(rect(&hits, submenu), svtui::layout::Rect::new(1, 1, 22, 1));
    assert_eq!(rect(&hits, leaf).y, 2);
    assert_eq!(buffer.get(21, 1).unwrap().ch, '▶');
    for x in 1..=22 {
        assert_eq!(buffer.get(x, 1).unwrap().bg, Some(Color::White));
        assert_eq!(hits.hit_test(x, 1).unwrap().focus_id, Some(submenu));
    }
    assert_eq!(buffer.get(23, 1).unwrap().ch, '│');
}

#[test]
fn space_between_handles_empty_singleton_and_excess_content() {
    for count in 0..=2 {
        let ids: Vec<_> = (0..count).map(|_| FocusId::new()).collect();
        let children = ids
            .iter()
            .copied()
            .map(|id| view! { <Box width=6 height=1 focus_id=id tab_stop=true /> })
            .collect::<Vec<_>>();
        let node = view! {
            <Box width=10 height=1 flex_direction=FlexDirection::Row justify_content=Justify::SpaceBetween>
                {children}
            </Box>
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
        <Box width=20 height=1 flex_direction=FlexDirection::Row>
            <Box width=Size::Percent(100.0) height=1 focus_id=percentage tab_stop=true />
            <Box width=Size::Fill min_width=3 height=1 focus_id=fill tab_stop=true />
        </Box>
    };
    let (_, hits) = render(&node, 30, 4);
    assert_eq!(rect(&hits, percentage).width, 20);
    assert_eq!(rect(&hits, fill), svtui::layout::Rect::new(20, 0, 3, 1));
}

#[test]
fn one_overflow_axis_does_not_clip_visible_overflow_on_the_other() {
    let node = view! {
        <Box width=2 height=1 overflow_y=Overflow::Hidden>
            <Text>"ABCDE\nsecond"</Text>
        </Box>
    };
    let (buffer, _) = render(&node, 10, 5);
    assert_eq!(buffer.get(4, 0).unwrap().ch, 'E');
    assert_eq!(buffer.get(0, 1).unwrap().ch, ' ');
}

#[test]
fn auto_cross_size_does_not_reserve_scrollbars_for_unsplit_fill_siblings() {
    for direction in [FlexDirection::Row, FlexDirection::Column] {
        let (width, height) = direction.axes(Size::Fixed(20), Size::Auto);
        let (child_w, child_h) = direction.axes(Size::Fill, Size::Fixed(6));
        let node = view! {
            <Box flex_direction=direction width=width height=height overflow=Overflow::Auto>
                <Box width=child_w height=child_h />
                <Box width=child_w height=child_h />
            </Box>
        };
        assert_eq!(measure_node(&node, 80, 80), direction.axes(20, 6));
    }
}
