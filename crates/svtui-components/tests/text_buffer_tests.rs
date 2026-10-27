use svtui_components::TextBuffer;

#[test]
fn test_text_buffer_insert_and_cursor() {
    let mut buf = TextBuffer::new("");
    assert_eq!(buf.cursor(), 0);
    assert_eq!(buf.len(), 0);

    buf.insert_char('H');
    buf.insert_char('i');
    assert_eq!(buf.content(), "Hi");
    assert_eq!(buf.cursor(), 2);

    buf.move_left();
    assert_eq!(buf.cursor(), 1);
    buf.insert_char('e');
    buf.insert_char('y');
    assert_eq!(buf.content(), "Heyi");
    assert_eq!(buf.cursor(), 3);
}

#[test]
fn test_text_buffer_backspace_and_delete() {
    let mut buf = TextBuffer::new("Rust");
    assert_eq!(buf.cursor(), 4);

    assert!(buf.backspace());
    assert_eq!(buf.content(), "Rus");
    assert_eq!(buf.cursor(), 3);

    buf.set_cursor(1);
    assert!(buf.delete());
    assert_eq!(buf.content(), "Rs");
    assert_eq!(buf.cursor(), 1);
}

#[test]
fn test_text_buffer_word_navigation_and_deletion() {
    let mut buf = TextBuffer::new("one two three");
    assert_eq!(buf.cursor(), 13);

    buf.move_word_left();
    assert_eq!(buf.cursor(), 8); // at start of "three"

    buf.move_word_left();
    assert_eq!(buf.cursor(), 4); // at start of "two"

    buf.move_word_right();
    assert_eq!(buf.cursor(), 8); // after "two "

    buf.move_end();
    assert!(buf.delete_word_left());
    assert_eq!(buf.content(), "one two ");
    assert_eq!(buf.cursor(), 8);
}

#[test]
fn test_text_buffer_line_operations() {
    let mut buf = TextBuffer::new("first second third");
    buf.set_cursor(6); // at start of "second"

    assert!(buf.delete_to_end());
    assert_eq!(buf.content(), "first ");

    buf.insert_str("word");
    assert_eq!(buf.content(), "first word");
    buf.set_cursor(6);
    assert!(buf.delete_to_start());
    assert_eq!(buf.content(), "word");
    assert_eq!(buf.cursor(), 0);
}

#[test]
fn test_text_buffer_viewport_scrolling() {
    let mut buf = TextBuffer::new("");
    let visible_width = 5;

    for c in "0123456789".chars() {
        buf.insert_char(c);
        buf.update_viewport(visible_width, 1);
    }

    assert_eq!(buf.content(), "0123456789");
    assert_eq!(buf.cursor(), 10);
    assert_eq!(buf.scroll_offset(), 6);
    assert_eq!(buf.visible_text(visible_width), "6789");

    buf.move_home();
    buf.update_viewport(visible_width, 1);
    assert_eq!(buf.cursor(), 0);
    assert_eq!(buf.scroll_offset(), 0);
    assert_eq!(buf.visible_text(visible_width), "01234");
}

#[test]
fn test_text_buffer_unicode() {
    let mut buf = TextBuffer::new("🦀🚀");
    assert_eq!(buf.len(), 2);
    assert_eq!(buf.cursor(), 2);

    buf.move_left();
    assert_eq!(buf.cursor(), 1);
    buf.insert_char('✨');
    assert_eq!(buf.content(), "🦀✨🚀");
    assert_eq!(buf.cursor(), 2);

    assert!(buf.backspace());
    assert_eq!(buf.content(), "🦀🚀");
}

#[test]
fn test_text_buffer_multiline_split_and_merge() {
    let mut buf = TextBuffer::new("Line 1");
    buf.insert_char('\n');
    buf.insert_str("Line 2");

    assert_eq!(buf.line_count(), 2);
    assert_eq!(buf.content(), "Line 1\nLine 2");
    assert_eq!(buf.cursor_pos(), (1, 6));

    // Backspace Line 2 back to start of line 2
    for _ in 0..6 {
        buf.backspace();
    }
    assert_eq!(buf.cursor_pos(), (1, 0));

    // Backspace at (1, 0) merges with Line 1
    assert!(buf.backspace());
    assert_eq!(buf.line_count(), 1);
    assert_eq!(buf.content(), "Line 1");
    assert_eq!(buf.cursor_pos(), (0, 6));
}

#[test]
fn test_text_buffer_multiline_navigation_preferred_col() {
    let mut buf = TextBuffer::new("Very Long First Line\nShort\nAnother Long Line");
    buf.set_cursor_pos(0, 15);
    assert_eq!(buf.cursor_pos(), (0, 15));

    // Move down to "Short" (len 5) -> clamped to 5
    buf.move_down();
    assert_eq!(buf.cursor_pos(), (1, 5));

    // Move down to "Another Long Line" -> restores preferred column 15
    buf.move_down();
    assert_eq!(buf.cursor_pos(), (2, 15));
}

#[test]
fn test_text_buffer_multiline_viewport_2d() {
    let mut buf = TextBuffer::new("");
    for i in 0..10 {
        if i > 0 {
            buf.insert_char('\n');
        }
        buf.insert_str(&format!("Line {}", i));
    }

    // Viewport: 20 cols wide, 3 rows high
    buf.set_cursor_pos(8, 6);
    buf.update_viewport(20, 3);

    assert_eq!(buf.scroll_row, 6);
    let visible = buf.visible_lines(20, 3);
    assert_eq!(visible.len(), 3);
    assert_eq!(visible[0], "Line 6");
    assert_eq!(visible[1], "Line 7");
    assert_eq!(visible[2], "Line 8");
}
