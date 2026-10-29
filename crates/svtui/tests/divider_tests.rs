use svtui::prelude::*;
use svtui::{Buffer, element::render_node, layout::Rect};
use svtui_components::*;

fn render(node: &VNode, width: u16, height: u16) -> Buffer {
    let mut buffer = Buffer::new(width, height);
    render_node(node, Rect::new(0, 0, width, height), &mut buffer);
    buffer
}

fn line(buffer: &Buffer, y: u16, start: u16, end: u16) -> String {
    (start..end).map(|x| buffer.get(x, y).unwrap().ch).collect()
}

#[test]
fn divider_fills_wide_container_content_width() {
    let node = view! {
        <FlexBox width=80 border=BorderStyle::Single padding=1>
            <Divider />
        </FlexBox>
    };
    let buffer = render(&node, 100, 10);
    assert_eq!(line(&buffer, 2, 2, 78), "─".repeat(76));
    assert_eq!(buffer.get(78, 2).unwrap().ch, ' ');
    assert_eq!(buffer.get(79, 2).unwrap().ch, '│');
}

#[test]
fn divider_never_paints_past_a_narrow_allocation() {
    let node = view! { <FlexBox width=10><Divider /></FlexBox> };
    let buffer = render(&node, 60, 5);
    assert_eq!(line(&buffer, 0, 0, 10), "─".repeat(10));
    assert_eq!(buffer.get(10, 0).unwrap().ch, ' ');
}

#[test]
fn divider_does_not_force_an_auto_container_to_an_arbitrary_width() {
    let node = view! { <FlexBox><TextSpan>"Content"</TextSpan><Divider /></FlexBox> };
    let buffer = render(&node, 100, 5);
    assert_eq!(line(&buffer, 1, 0, 7), "─".repeat(7));
    assert_eq!(buffer.get(7, 1).unwrap().ch, ' ');
}

#[test]
fn divider_honors_fixed_percentage_fill_and_width_limits() {
    for (width, min, max, expected) in [
        (Size::Fixed(8), 0, 40, 8),
        (Size::Percent(50.0), 0, 40, 20),
        (Size::Fill, 0, 40, 40),
        (Size::Fill, 0, 12, 12),
        (Size::Fixed(8), 12, 14, 12),
        (Size::Fixed(8), 12, 6, 12),
        (Size::Fixed(0), 0, 40, 0),
    ] {
        let node = view! {
            <FlexBox width=40>
                <Divider width=width min_width=min max_width=max />
            </FlexBox>
        };
        let buffer = render(&node, 60, 5);
        assert_eq!(
            line(&buffer, 0, 0, expected),
            "─".repeat(usize::from(expected))
        );
        assert_eq!(buffer.get(expected, 0).unwrap().ch, ' ');
    }
}

#[test]
fn divider_fills_its_share_of_a_flex_row() {
    let node = view! {
        <FlexBox width=50 height=1 flex_direction=FlexDirection::Row gap=1>
            <FlexBox width=10 height=1><TextSpan>"Fixed"</TextSpan></FlexBox>
            <Divider />
            <Divider max_width=12 />
        </FlexBox>
    };
    let buffer = render(&node, 60, 3);
    assert_eq!(line(&buffer, 0, 11, 37), "─".repeat(26));
    assert_eq!(buffer.get(37, 0).unwrap().ch, ' ');
    assert_eq!(line(&buffer, 0, 38, 50), "─".repeat(12));
    assert_eq!(buffer.get(50, 0).unwrap().ch, ' ');
}

#[test]
fn titled_dividers_clip_and_fill_after_resizing_the_same_tree() {
    let node = view! {
        <FlexBox width=Size::Fill height=2>
            <Divider title="Settings" />
        </FlexBox>
    };
    for width in [0, 1, 5, 12, 30, 100] {
        let buffer = render(&node, width, 3);
        let expected: String = "─── Settings "
            .chars()
            .chain(std::iter::repeat('─'))
            .take(usize::from(width))
            .collect();
        assert_eq!(line(&buffer, 0, 0, width), expected);
        assert_eq!(line(&buffer, 1, 0, width), " ".repeat(usize::from(width)));
    }
}

#[test]
fn divider_excludes_the_scrollbar_and_preserves_colors() {
    let node = view! {
        <FlexBox width=10 height=3 overflow=Overflow::Auto bg=Color::White>
            <Divider color=Color::Black />
            <FlexBox height=4 />
        </FlexBox>
    };
    let buffer = render(&node, 20, 5);
    assert_eq!(line(&buffer, 0, 0, 9), "─".repeat(9));
    assert_eq!(buffer.get(9, 0).unwrap().ch, '█');
    for x in 0..9 {
        let cell = buffer.get(x, 0).unwrap();
        assert_eq!(cell.fg, Some(Color::Black));
        assert_eq!(cell.bg, Some(Color::White));
    }
    assert_ne!(buffer.get(0, 2).unwrap().ch, '▄');
}

#[test]
fn divider_fill_does_not_create_horizontal_scroll_content() {
    let node = view! {
        <FlexBox width=12 height=2 overflow_x=Overflow::Auto>
            <Divider />
        </FlexBox>
    };
    let buffer = render(&node, 30, 4);
    assert_eq!(line(&buffer, 0, 0, 12), "─".repeat(12));
    assert_eq!(line(&buffer, 1, 0, 12), " ".repeat(12));
}

#[test]
fn text_fill_uses_allocated_width_without_inflating_intrinsic_measurement() {
    let node = view! {
        <FlexBox width=6 height=3 align_items=Align::Stretch>
            <Paragraph fill='.'>"ab\nx"</Paragraph>
        </FlexBox>
    };
    let buffer = render(&node, 10, 5);
    assert_eq!(line(&buffer, 0, 0, 6), "ab....");
    assert_eq!(line(&buffer, 1, 0, 6), "x.....");
    assert_eq!(line(&buffer, 2, 0, 6), "      ");
    let filled_text = view! { <Paragraph fill='.'>"ab"</Paragraph> };
    assert_eq!(svtui::element::measure_node(&filled_text, 80, 24), (2, 1));
}
