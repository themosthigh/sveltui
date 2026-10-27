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

#[test]
fn test_popover_component_into_node() {
    use svtui_components::{Button, Popover};
    use svtui_layout::Placement;

    let popover = Popover::new()
        .trigger(Button::new("Open Menu"))
        .content(Button::new("Inside Popover"))
        .open(true)
        .placement(Placement::BottomStart)
        .into_node();

    match popover {
        VNode::Overlay(overlay) => {
            assert!(overlay.open);
            assert_eq!(overlay.placement, Placement::BottomStart);
            assert!(overlay.trigger.is_some());
            assert!(overlay.content.is_some());
        }
        _ => panic!("Expected VNode::Overlay"),
    }
}

#[test]
fn test_dialog_component_into_node() {
    use svtui_components::Dialog;

    let dialog = Dialog::new()
        .title("Confirm Action")
        .open(true)
        .child(Button::new("OK"))
        .into_node();

    match dialog {
        VNode::Overlay(overlay) => {
            assert!(overlay.open);
            assert!(overlay.modal);
            assert!(overlay.backdrop);
            assert!(overlay.content.is_some());
        }
        _ => panic!("Expected VNode::Overlay"),
    }
}

#[test]
fn test_dropdown_component_into_node() {
    use svtui_components::{Button, DropdownItem, DropdownMenu};

    let dropdown = DropdownMenu::new()
        .trigger(Button::new("File"))
        .open(true)
        .item(DropdownItem::new("New File"))
        .item(DropdownItem::new("Save"))
        .into_node();

    match dropdown {
        VNode::Overlay(overlay) => {
            assert!(overlay.open);
            assert!(!overlay.modal);
            assert!(overlay.trigger.is_some());
            assert!(overlay.content.is_some());
        }
        _ => panic!("Expected VNode::Overlay"),
    }
}

#[test]
fn test_nested_dropdown_submenus_3_level() {
    use svtui_components::{Button, DropdownItem, DropdownMenu, DropdownSubmenu};

    // Level 3: Document formats
    let level_3 = DropdownSubmenu::new("Document")
        .item(DropdownItem::new("PDF Document (*.pdf)"))
        .item(DropdownItem::new("Markdown (*.md)"))
        .open(true);

    // Level 2: Export submenu containing Level 3
    let level_2 = DropdownSubmenu::new("Export As")
        .item(DropdownItem::new("Image (PNG)"))
        .item(level_3)
        .item(DropdownItem::new("Archive (*.zip)"))
        .open(true);

    // Level 1: Root dropdown menu containing Level 2
    let level_1 = DropdownMenu::new()
        .trigger(Button::new("File"))
        .open(true)
        .item(DropdownItem::new("New Project"))
        .item(level_2)
        .into_node();

    match level_1 {
        VNode::Overlay(overlay) => {
            assert!(overlay.open);
            assert!(overlay.content.is_some());
        }
        _ => panic!("Expected root VNode::Overlay"),
    }
}
