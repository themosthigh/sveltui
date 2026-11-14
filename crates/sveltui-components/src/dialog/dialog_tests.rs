use crate::*;
use sveltui::element::*;
use sveltui::layout::*;
use sveltui::terminal::Buffer;

#[test]
fn test_nested_scroll_box_scroll() {
    let scroll_id = FocusId::new(); // User calls FocusId::new() in main()
    let tree = ViewElement::new() // Root View
        .child(
            ViewElement::new() // Dialog View
                .child(
                    ViewElement::new() // Scrollable View
                        .scroll_id(scroll_id)
                        .height(3)
                        .overflow_y(Overflow::Scroll)
                        .child(TextElement::new("Line 1"))
                        .child(TextElement::new("Line 2"))
                        .child(TextElement::new("Line 3"))
                        .child(TextElement::new("Line 4"))
                        .child(TextElement::new("Line 5"))
                        .child(TextElement::new("Line 6")),
                ),
        )
        .into_node();

    reset_focus_counter();
    let mut buffer = Buffer::new(20, 10);
    let mut hit_map = HitMap::new();
    render_node_interactive(&tree, Rect::new(0, 0, 20, 10), &mut buffer, &mut hit_map);

    // Hit test scroll container
    let target = hit_map
        .targets
        .iter()
        .find(|t| t.focus_id == Some(scroll_id))
        .expect("target should be found");
    let on_scroll = target.on_scroll.as_ref().expect("on_scroll should exist");
    on_scroll(0, 2);

    // Next frame render
    reset_focus_counter();
    let mut buffer2 = Buffer::new(20, 10);
    let mut hit_map2 = HitMap::new();
    render_node_interactive(&tree, Rect::new(0, 0, 20, 10), &mut buffer2, &mut hit_map2);

    assert_eq!(get_scroll(scroll_id), (0, 2));
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
fn bind_open_is_cleared_when_the_dialog_is_dismissed() {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use sveltui::prelude::*;

    let show = state(true);
    let build = move || {
        view! {
            <Dialog id={FocusId(100_301)} bind:open={show} title="Confirm">
                <Button id={FocusId(100_302)}>"OK"</Button>
            </Dialog>
        }
    };
    let mut manager = FocusManager::new();
    let frame = |manager: &mut FocusManager| {
        for _ in 0..5 {
            reset_focus_counter();
            begin_frame();
            let node = manager.with_context(build);
            end_frame();
            let mut buffer = Buffer::new(60, 20);
            let mut hits = HitMap::new();
            manager.with_context(|| {
                render_node_interactive(&node, Rect::new(0, 0, 60, 20), &mut buffer, &mut hits)
            });
            if !manager.reconcile(hits.interactions) {
                return;
            }
        }
    };
    frame(&mut manager);
    assert_eq!(
        manager.focused(),
        Some(FocusId(100_302)),
        "the dialog took focus"
    );
    manager.dispatch(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
    frame(&mut manager);
    assert!(!show.get(), "Escape closes the dialog and writes false");
}
