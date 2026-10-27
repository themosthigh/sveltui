use svtui_layout::{BorderStyle, Rect};
use svtui_terminal::Buffer;

#[test]
fn test_buffer_set_string() {
    let mut buf = Buffer::new(20, 5);
    buf.set_string(2, 2, "Hello", None, None, false, 10);
    assert_eq!(buf.get(2, 2).unwrap().ch, 'H');
    assert_eq!(buf.get(3, 2).unwrap().ch, 'e');
    assert_eq!(buf.get(4, 2).unwrap().ch, 'l');
    assert_eq!(buf.get(5, 2).unwrap().ch, 'l');
    assert_eq!(buf.get(6, 2).unwrap().ch, 'o');
    assert_eq!(buf.get(7, 2).unwrap().ch, ' ');
}

#[test]
fn test_buffer_clipping() {
    let mut buf = Buffer::new(20, 10);
    buf.push_clip(Rect::new(2, 2, 4, 3));

    // Drawing inside clip (3, 3) should work
    buf.set_char(3, 3, 'X', None, None, false);
    assert_eq!(buf.get(3, 3).unwrap().ch, 'X');

    // Drawing outside clip (1, 1) should be ignored
    buf.set_char(1, 1, 'Y', None, None, false);
    assert_eq!(buf.get(1, 1).unwrap().ch, ' ');

    // Drawing outside clip (10, 3) should be ignored
    buf.set_char(10, 3, 'Z', None, None, false);
    assert_eq!(buf.get(10, 3).unwrap().ch, ' ');

    buf.pop_clip();
    // After popping, clip is restored to whole buffer
    buf.set_char(1, 1, 'Y', None, None, false);
    assert_eq!(buf.get(1, 1).unwrap().ch, 'Y');
}

#[test]
fn test_buffer_draw_border() {
    let mut buf = Buffer::new(10, 5);
    let rect = Rect::new(0, 0, 6, 3);
    let chars = BorderStyle::Rounded.chars().unwrap();
    buf.draw_border(rect, chars, None);

    assert_eq!(buf.get(0, 0).unwrap().ch, '╭');
    assert_eq!(buf.get(5, 0).unwrap().ch, '╮');
    assert_eq!(buf.get(0, 2).unwrap().ch, '╰');
    assert_eq!(buf.get(5, 2).unwrap().ch, '╯');
    assert_eq!(buf.get(1, 0).unwrap().ch, '─');
    assert_eq!(buf.get(0, 1).unwrap().ch, '│');
}

#[test]
fn test_buffer_cursor() {
    let mut buf = Buffer::new(10, 5);
    assert_eq!(buf.cursor(), None);

    buf.set_cursor(4, 2);
    assert_eq!(buf.cursor(), Some((4, 2)));

    buf.clear_cursor();
    assert_eq!(buf.cursor(), None);

    buf.set_cursor(7, 3);
    buf.clear();
    assert_eq!(buf.cursor(), None);
}
