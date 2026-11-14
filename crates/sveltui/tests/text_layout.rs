use sveltui::{
    Buffer,
    element::{render_node, taffy_bridge::LayoutEngine},
    layout::Rect,
    prelude::*,
};

fn line(buffer: &Buffer, y: u16, start: u16, end: u16) -> String {
    (start..end).map(|x| buffer.get(x, y).unwrap().ch).collect()
}

fn render(node: &VNode, width: u16, height: u16) -> Buffer {
    let mut buffer = Buffer::new(width, height);
    render_node(node, Rect::new(0, 0, width, height), &mut buffer);
    buffer
}

#[test]
fn direct_text_nodes_follow_parent_alignment_and_justification_on_both_axes() {
    for paragraph in [false, true] {
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
                    let (width, height) = direction.axes(10, 6);
                    let node = view! {
                        <View width=width height=height flex_direction=direction gap=1
                            align_items=align justify_content=justify>
                            {#if paragraph}
                                <Paragraph>"ab\ncd"</Paragraph>
                                <Paragraph>"ef\ngh"</Paragraph>
                            {:else}
                                <TextSpan>"ab\ncd"</TextSpan>
                                <TextSpan>"ef\ngh"</TextSpan>
                            {/if}
                        </View>
                    };
                    let layout = LayoutEngine::new().compute_layout(&node, width, height);
                    let buffer = render(&node, width, height);
                    for ((child, main), content) in
                        layout.children.iter().zip(positions).zip(["abcd", "efgh"])
                    {
                        let (x, y) = direction.axes(main, cross);
                        assert_eq!(
                            child.rect,
                            Rect::new(x, y, 2, 2),
                            "paragraph={paragraph}, {direction:?}, {justify:?}, {align:?}"
                        );
                        let actual: String = [(x, y), (x + 1, y), (x, y + 1), (x + 1, y + 1)]
                            .into_iter()
                            .map(|(x, y)| buffer.get(x, y).unwrap().ch)
                            .collect();
                        assert_eq!(actual, content);
                    }
                }
            }
        }
    }
}

#[test]
fn direct_text_nodes_stretch_in_the_parent_cross_axis() {
    for paragraph in [false, true] {
        for direction in [FlexDirection::Row, FlexDirection::Column] {
            let (width, height) = direction.axes(10, 6);
            let node = view! {
                <View width=width height=height flex_direction=direction align_items=AlignItems::Stretch>
                    {#if paragraph}<Paragraph>"ab\ncd"</Paragraph>
                    {:else}<TextSpan>"ab\ncd"</TextSpan>{/if}
                </View>
            };
            let layout = LayoutEngine::new().compute_layout(&node, width, height);
            let (child_w, child_h) = direction.axes(2, 6);
            assert_eq!(layout.children[0].rect, Rect::new(0, 0, child_w, child_h));
        }
    }
}

#[test]
fn centered_text_remains_aligned_after_resizing() {
    let node = view! {
        <View width={Dimension::Percent(1.0)} height=3 align_items=AlignItems::Center justify_content=JustifyContent::Center>
            <TextSpan>"Span"</TextSpan>
            <Paragraph>"Text"</Paragraph>
        </View>
    };
    for width in [12, 20, 6, 12] {
        let layout = LayoutEngine::new().compute_layout(&node, width, 3);
        let x = (width - 4) / 2;
        let buffer = render(&node, width, 3);
        for (idx, text) in ["Span", "Text"].into_iter().enumerate() {
            let rect = layout.children[idx].rect;
            assert_eq!(rect.x, x);
            let actual: String = (x..x + 4)
                .map(|x| buffer.get(x, rect.y).unwrap().ch)
                .collect();
            assert_eq!(actual, text);
        }
    }
}

#[test]
fn stretched_paragraph_reflows_and_keeps_the_next_sibling_below_it() {
    let node = view! {
        <View width=12 align_items=AlignItems::Stretch>
            <Paragraph>"one two three four five six"</Paragraph>
            <TextSpan>"!"</TextSpan>
        </View>
    };
    let layout = LayoutEngine::new().compute_layout(&node, 12, 10);
    assert_eq!(layout.children[0].rect, Rect::new(0, 0, 12, 3));
    assert_eq!(layout.children[1].rect.y, 3);
    let buffer = render(&node, 12, 10);
    let rows: Vec<_> = (0..4)
        .map(|y| {
            (0..12)
                .map(|x| buffer.get(x, y).unwrap().ch)
                .collect::<String>()
                .trim_end()
                .to_owned()
        })
        .collect();
    assert_eq!(rows, ["one two", "three four", "five six", "!"]);
}

#[test]
fn text_fill_uses_allocated_width_without_inflating_intrinsic_measurement() {
    let node = view! {
        <View width=6 height=3 align_items=AlignItems::Stretch>
            <Paragraph fill='.'>"ab\nx"</Paragraph>
        </View>
    };
    let buffer = render(&node, 10, 5);
    assert_eq!(line(&buffer, 0, 0, 6), "ab....");
    assert_eq!(line(&buffer, 1, 0, 6), "x.....");
    assert_eq!(line(&buffer, 2, 0, 6), "      ");
    let filled_text = view! { <Paragraph fill='.'>"ab"</Paragraph> };
    assert_eq!(sveltui::element::measure_node(&filled_text, 80, 24), (2, 1));
}
