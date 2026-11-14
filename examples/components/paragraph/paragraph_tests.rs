use sveltui::prelude::*;
use sveltui::{
    Buffer,
    element::{measure_node, render_node},
    layout::Rect,
};

const TEXT: &str = "Long sentences should wrap at word boundaries and remain inside the paragraph.\nExplicit newlines start a fresh line.";

fn line(buffer: &Buffer, y: u16, start: u16, end: u16) -> String {
    (start..end).map(|x| buffer.get(x, y).unwrap().ch).collect()
}

fn assert_wrapping(paragraph: VNode) {
    let node = view! {
        <View width={Dimension::Percent(1.0)} border=BorderStyle::Single padding=1 gap=1>
            {paragraph}
            <TextSpan>"!"</TextSpan>
        </View>
    };
    for width in [24, 36, 16, 5, 24] {
        let (_, height) = measure_node(&node, width, 200);
        let mut buffer = Buffer::new(80, 200);
        render_node(&node, Rect::new(0, 0, width, 200), &mut buffer);
        let rows: Vec<_> = (2..height - 4)
            .map(|y| line(&buffer, y, 2, width - 2).trim_end().to_owned())
            .collect();
        let actual: String = rows
            .join("")
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect();
        let expected: String = TEXT.chars().filter(|c| !c.is_whitespace()).collect();
        assert_eq!(
            actual, expected,
            "all text must survive wrapping at width={width}"
        );
        if width == 24 {
            assert_eq!(
                rows,
                [
                    "Long sentences",
                    "should wrap at word",
                    "boundaries and",
                    "remain inside the",
                    "paragraph.",
                    "Explicit newlines",
                    "start a fresh line.",
                ]
            );
        }
        if width >= 16 {
            assert_eq!(
                rows.join(" ").split_whitespace().collect::<Vec<_>>(),
                TEXT.split_whitespace().collect::<Vec<_>>(),
                "words should stay together"
            );
        }
        assert_eq!(buffer.get(2, height - 3).unwrap().ch, '!');
        for y in 1..height - 1 {
            assert_eq!(buffer.get(0, y).unwrap().ch, '│');
            assert_eq!(buffer.get(1, y).unwrap().ch, ' ');
            assert_eq!(buffer.get(width - 2, y).unwrap().ch, ' ');
            assert_eq!(buffer.get(width - 1, y).unwrap().ch, '│');
        }
        for y in 0..200 {
            assert_eq!(
                line(&buffer, y, width, 80),
                " ".repeat(usize::from(80 - width))
            );
        }
    }
}

#[test]
fn paragraph_children_wrap_without_losing_text_after_resizing() {
    assert_wrapping(view! { <Paragraph foreground_color=Color::White>{TEXT}</Paragraph> });
}

#[test]
fn paragraph_content_property_wraps_without_losing_text_after_resizing() {
    assert_wrapping(view! { <Paragraph foreground_color=Color::White content=TEXT /> });
}

#[test]
fn wrapping_preserves_blank_lines_and_splits_long_words() {
    let node = view! { <Paragraph>"abcdefghij\n\nend"</Paragraph> };
    assert_eq!(measure_node(&node, 4, 10), (4, 5));
    let mut buffer = Buffer::new(4, 5);
    render_node(&node, Rect::new(0, 0, 4, 5), &mut buffer);
    assert_eq!(
        (0..5).map(|y| line(&buffer, y, 0, 4)).collect::<Vec<_>>(),
        ["abcd", "efgh", "ij  ", "    ", "end "]
    );
    assert_eq!(measure_node(&node, 0, 10), (0, 0));
}

#[test]
fn text_spans_keep_single_line_overflow_behavior() {
    let node = view! { <TextSpan>"abcdefghij"</TextSpan> };
    assert_eq!(measure_node(&node, 4, 10), (10, 1));
}

#[test]
fn nested_auto_containers_reserve_height_for_wrapped_paragraphs() {
    let node = view! {
        <View width=12>
            <View>
                <Paragraph>"one two three four five six"</Paragraph>
            </View>
            <TextSpan>"!"</TextSpan>
        </View>
    };
    assert_eq!(measure_node(&node, 12, 10), (12, 4));
    let mut buffer = Buffer::new(12, 10);
    render_node(&node, Rect::new(0, 0, 12, 10), &mut buffer);
    assert_eq!(
        (0..4)
            .map(|y| line(&buffer, y, 0, 12).trim_end().to_owned())
            .collect::<Vec<_>>(),
        ["one two", "three four", "five six", "!"]
    );
}

#[test]
fn max_lines_adds_ellipsis_only_when_text_is_hidden() {
    for (text, width, max_lines, expected) in [
        (
            "one two three four five six",
            12,
            2,
            vec!["one two", "three four…"],
        ),
        (
            "one two three four five six",
            10,
            2,
            vec!["one two", "three fou…"],
        ),
        ("one two three four five six", 6, 2, vec!["one", "two…"]),
        (
            "one two three four five six",
            30,
            2,
            vec!["one two three four five six"],
        ),
        ("one two\nthree", 7, 2, vec!["one two", "three"]),
        ("short\nhidden", 12, 1, vec!["short…"]),
        ("ééééé\nnext", 4, 1, vec!["ééé…"]),
        ("long text", 1, 1, vec!["…"]),
        ("long text", 8, 0, vec![]),
        ("", 8, 1, vec![""]),
    ] {
        let node = view! {
            <View width={Dimension::Percent(1.0)}>
                <Paragraph max_lines=max_lines content=text />
                <TextSpan>"!"</TextSpan>
            </View>
        };
        let height = expected.len() as u16;
        assert_eq!(measure_node(&node, width, 20), (width, height + 1));
        let mut buffer = Buffer::new(width + 2, 20);
        render_node(&node, Rect::new(0, 0, width, 20), &mut buffer);
        let actual: Vec<_> = (0..height)
            .map(|y| line(&buffer, y, 0, width).trim_end().to_owned())
            .collect();
        assert_eq!(actual, expected, "width={width}, max_lines={max_lines}");
        assert_eq!(line(&buffer, height, 0, width).trim_end(), "!");
        for y in 0..20 {
            assert_eq!(line(&buffer, y, width, width + 2), "  ");
        }
    }
}

#[test]
fn limited_paragraph_children_reflow_when_resized() {
    let node = view! {
        <View width={Dimension::Percent(1.0)}>
            <Paragraph max_lines=1>"alpha beta gamma"</Paragraph>
        </View>
    };
    for (width, expected) in [(6, "alpha…"), (20, "alpha beta gamma"), (6, "alpha…")] {
        assert_eq!(measure_node(&node, width, 10), (width, 1));
        let mut buffer = Buffer::new(24, 3);
        render_node(&node, Rect::new(0, 0, width, 3), &mut buffer);
        assert_eq!(line(&buffer, 0, 0, width).trim_end(), expected);
        assert_eq!(line(&buffer, 1, 0, 24), " ".repeat(24));
    }
    let empty = view! { <Paragraph max_lines=1>"hidden"</Paragraph> };
    assert_eq!(measure_node(&empty, 0, 10), (0, 0));
}

#[test]
fn poem_demo_keyboard_and_wheel_share_the_scroll_position() {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use sveltui::element::{get_scroll, render_node_interactive, reset_focus_counter};

    let mut manager = FocusManager::new();
    let render = |manager: &mut FocusManager| {
        reset_focus_counter();
        begin_frame();
        let node = manager.with_context(|| view! { <super::ParagraphDemo /> });
        end_frame();
        let mut buffer = Buffer::new(80, 24);
        let mut hits = HitMap::new();
        render_node_interactive(&node, Rect::new(0, 0, 80, 24), &mut buffer, &mut hits);
        manager.reconcile(hits.interactions.clone());
        (buffer, hits)
    };
    let (before, hits) = render(&mut manager);
    let id = manager.tab_order()[0];
    manager.focus(id);
    assert!(
        manager
            .dispatch(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE))
            .handled
    );
    assert_eq!(get_scroll(id), (0, 1));
    let target = hits.targets.iter().find(|t| t.on_scroll.is_some()).unwrap();
    assert_eq!(target.focus_id, Some(id));
    manager.scroll(target, 0, 3);
    let (after, _) = render(&mut manager);
    assert_eq!(get_scroll(id), (0, 4));
    assert_ne!(before.cells, after.cells);
}
