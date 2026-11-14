use crate::*;
use crossterm::style::Color;
use sveltui::element::taffy_bridge::view_to_taffy_style;
use sveltui::element::*;
use sveltui::layout::*;
use sveltui::prelude::*;
use sveltui::terminal::Buffer;

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
        <View width=80 border=BorderStyle::Single padding=1>
            <Divider />
        </View>
    };
    let buffer = render(&node, 100, 10);
    assert_eq!(line(&buffer, 2, 2, 78), "─".repeat(76));
    assert_eq!(buffer.get(78, 2).unwrap().ch, ' ');
    assert_eq!(buffer.get(79, 2).unwrap().ch, '│');
}

#[test]
fn divider_never_paints_past_a_narrow_allocation() {
    let node = view! { <View width=10><Divider /></View> };
    let buffer = render(&node, 60, 5);
    assert_eq!(line(&buffer, 0, 0, 10), "─".repeat(10));
    assert_eq!(buffer.get(10, 0).unwrap().ch, ' ');
}

#[test]
fn divider_does_not_force_an_auto_container_to_an_arbitrary_width() {
    let node = view! { <View><TextSpan>"Content"</TextSpan><Divider /></View> };
    let buffer = render(&node, 100, 5);
    assert_eq!(line(&buffer, 1, 0, 7), "─".repeat(7));
    assert_eq!(buffer.get(7, 1).unwrap().ch, ' ');
}

#[test]
fn divider_honors_fixed_percentage_auto_and_width_limits() {
    for (width, min, max, expected) in [
        (Dimension::Length(8.0), 0, 40, 8),
        (Dimension::Percent(0.5), 0, 40, 20),
        (Dimension::Percent(1.0), 0, 40, 40),
        (Dimension::Percent(1.0), 0, 12, 12),
        (Dimension::Length(8.0), 12, 14, 12),
        (Dimension::Length(8.0), 12, 6, 12),
        (Dimension::Length(0.0), 0, 40, 0),
    ] {
        let node = view! {
            <View width=40>
                <Divider width=width min_width=min max_width=max />
            </View>
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
        <View width=50 height=1 flex_direction=FlexDirection::Row gap=1>
            <View width=10 height=1><TextSpan>"Fixed"</TextSpan></View>
            <Divider flex_grow=1.0_f32 flex_basis=0 />
            <Divider max_width=12 flex_grow=1.0_f32 flex_basis=0 />
        </View>
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
        <View width={Dimension::Percent(1.0)} height=2>
            <Divider title="Settings" />
        </View>
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
        <View width=10 height=3 overflow=Overflow::Scroll background_color=Color::White>
            <Divider color=Color::Black />
            <View height=4 flex_shrink=0.0 />
        </View>
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
        <View width=12 height=2 overflow_x=Overflow::Scroll>
            <Divider />
        </View>
    };
    let buffer = render(&node, 30, 4);
    assert_eq!(line(&buffer, 0, 0, 12), "─".repeat(12));
    assert_eq!(line(&buffer, 1, 0, 12), " ".repeat(12));
}

#[test]
fn native_dimensions_preserve_fractional_lengths_and_percentage_units() {
    use crate::DividerProps;

    let view = ViewElement::new()
        .width(40.5)
        .height(Dimension::Percent(0.5))
        .flex_basis(10.25)
        .flex_grow(2.0)
        .flex_shrink(0.0);
    let style = view_to_taffy_style(&view);
    assert_eq!(style.size.width, Dimension::Length(40.5).into());
    assert_eq!(style.size.height, Dimension::Percent(0.5).into());
    assert_eq!(style.flex_basis, Dimension::Length(10.25).into());
    assert_eq!(style.flex_grow, 2.0);
    assert_eq!(style.flex_shrink, 0.0);

    let props = DividerProps::builder()
        .width(40.5)
        .min_width(10.25)
        .max_width(Dimension::Percent(0.5))
        .build();
    assert_eq!(props.width, Dimension::Length(40.5));
    assert_eq!(props.min_width, Some(Dimension::Length(10.25)));
    assert_eq!(props.max_width, Some(Dimension::Percent(0.5)));
}
