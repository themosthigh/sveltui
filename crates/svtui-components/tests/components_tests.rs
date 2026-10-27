use svtui_components::{Badge, Button, TextArea, get_cursor, set_cursor};
use svtui_element::{FocusId, IntoNode, VNode, measure_node};
use svtui_reactive::create_signal;

#[test]
fn test_button_measurement() {
    let btn = Button::new("Click Me");
    let node = btn.into_node();
    let (w, h) = measure_node(&node, 80, 24);
    assert!(w >= 10);
    assert_eq!(h, 3); // 1 row text + 2 rows border
}

#[test]
fn test_badge_measurement() {
    let badge = Badge::new("OK");
    let node = badge.into_node();
    let (w, h) = measure_node(&node, 80, 24);
    assert!(w >= 4);
    assert_eq!(h, 3);
}

#[test]
fn test_text_area_basic_instantiation() {
    let (val, set_val) = create_signal(String::from("Hello\nWorld"));
    let area = TextArea::new(val, set_val)
        .width(50)
        .height(10)
        .title("Notes")
        .show_line_numbers(true);

    let node = area.into_node();
    match node {
        VNode::Box(b) => {
            assert_eq!(b.title, Some("Notes".to_string()));
            assert_eq!(b.children.len(), 2);
        }
        _ => panic!("Expected Box node"),
    }
}

#[test]
fn test_text_input_cursor_forward_typing() {
    let (val, set_val) = create_signal(String::new());
    let id = FocusId(999);

    // Initially cursor should be at 0
    assert_eq!(get_cursor(id, 0), 0);

    // Type 'a'
    let mut text: String = val.get();
    let cur = get_cursor(id, text.len());
    text.insert(cur, 'a');
    set_cursor(id, cur + 1);
    set_val.set(text);

    assert_eq!(val.get(), "a");
    assert_eq!(get_cursor(id, 1), 1);

    // Type 'b'
    let mut text: String = val.get();
    let cur = get_cursor(id, text.len());
    text.insert(cur, 'b');
    set_cursor(id, cur + 1);
    set_val.set(text);

    // Value must be "ab", NOT "ba"!
    assert_eq!(val.get(), "ab");
    assert_eq!(get_cursor(id, 2), 2);

    // Type 'c'
    let mut text: String = val.get();
    let cur = get_cursor(id, text.len());
    text.insert(cur, 'c');
    set_cursor(id, cur + 1);
    set_val.set(text);

    // Value must be "abc"!
    assert_eq!(val.get(), "abc");
    assert_eq!(get_cursor(id, 3), 3);
}
