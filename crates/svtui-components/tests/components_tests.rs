use svtui::Snippet;
use svtui_components::{
    Badge, BadgeProps, Button, ButtonProps, Dialog, DialogProps, DropdownItem, DropdownItemProps,
    DropdownMenu, DropdownMenuProps, DropdownSubmenu, DropdownSubmenuProps, Popover, PopoverProps,
    TextArea, TextAreaProps, get_cursor, set_cursor,
};
use svtui_element::{FlexBox, FlexBoxProps, FocusId, IntoNode, VNode, measure_node};
use svtui_layout::Placement;
use svtui_macros::view;
use svtui_reactive::create_signal;

#[test]
fn test_button_measurement() {
    let btn = Button(
        ButtonProps::builder()
            .children(Snippet::from("Click Me"))
            .build(),
    );
    let node = btn.into_node();
    let (w, h) = measure_node(&node, 80, 24);
    assert!(w >= 10);
    assert_eq!(h, 3); // 1 row text + 2 rows border
}

#[test]
fn test_badge_measurement() {
    let badge = Badge(
        BadgeProps::builder()
            .children(Snippet::new(|_| "Ok"))
            .build(),
    );
    let node = badge.into_node();
    let (w, h) = measure_node(&node, 80, 24);
    assert!(w >= 4);
    assert_eq!(h, 3);
}

#[test]
fn test_text_area_basic_instantiation() {
    let (val, set_val) = create_signal(String::from("Hello\nWorld"));
    let area = TextArea(
        TextAreaProps::builder()
            .value(val)
            .set_value(set_val)
            .width(50)
            .height(10)
            .title("Notes")
            .show_line_numbers(true)
            .build(),
    );

    let node = area.into_node();
    match node {
        VNode::FlexBox(b) => {
            assert_eq!(b.title, Some("Notes".to_string()));
            assert_eq!(b.children.len(), 2);
        }
        _ => panic!("Expected FlexBox node"),
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
    let popover = Popover(
        PopoverProps::builder()
            .open(true)
            .placement(Placement::BottomStart)
            .slot_trigger(Snippet::from(
                Button(
                    ButtonProps::builder()
                        .children(Snippet::from("Open Menu"))
                        .build(),
                )
                .into_node(),
            ))
            .children(Snippet::from(
                Button(
                    ButtonProps::builder()
                        .children(Snippet::from("Inside Popover"))
                        .build(),
                )
                .into_node(),
            ))
            .build(),
    )
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
    let dialog = Dialog(
        DialogProps::builder()
            .title("Confirm Action")
            .open(true)
            .children(Snippet::from(
                Button(ButtonProps::builder().children(Snippet::from("OK")).build()).into_node(),
            ))
            .build(),
    )
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
    let dropdown = DropdownMenu(
        DropdownMenuProps::builder()
            .open(true)
            .slot_trigger(Snippet::from(
                Button(
                    ButtonProps::builder()
                        .children(Snippet::from("File"))
                        .build(),
                )
                .into_node(),
            ))
            .children(Snippet::from(view! {
                <FlexBox>
                    <DropdownItem label="New File" />
                    <DropdownItem label="Save" />
                </FlexBox>
            }))
            .build(),
    )
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
    // Level 3: Document formats
    let level_3 = DropdownSubmenu(
        DropdownSubmenuProps::builder()
            .label("Document")
            .open(true)
            .children(Snippet::from(view! {
                <FlexBox>
                    <DropdownItem label="PDF Document (*.pdf)" />
                    <DropdownItem label="Markdown (*.md)" />
                </FlexBox>
            }))
            .build(),
    );

    // Level 2: Export submenu containing Level 3
    let level_2 = DropdownSubmenu(
        DropdownSubmenuProps::builder()
            .label("Export As")
            .open(true)
            .children(Snippet::from(view! {
                <FlexBox>
                    <DropdownItem label="Image (PNG)" />
                    {level_3}
                    <DropdownItem label="Archive (*.zip)" />
                </FlexBox>
            }))
            .build(),
    );

    // Level 1: Root dropdown menu containing Level 2
    let level_1 = DropdownMenu(
        DropdownMenuProps::builder()
            .open(true)
            .slot_trigger(Snippet::from(
                Button(
                    ButtonProps::builder()
                        .children(Snippet::from("File"))
                        .build(),
                )
                .into_node(),
            ))
            .children(Snippet::from(view! {
                <FlexBox>
                    <DropdownItem label="New Project" />
                    {level_2}
                </FlexBox>
            }))
            .build(),
    )
    .into_node();

    match level_1 {
        VNode::Overlay(overlay) => {
            assert!(overlay.open);
            assert!(overlay.content.is_some());
        }
        _ => panic!("Expected root VNode::Overlay"),
    }
}
