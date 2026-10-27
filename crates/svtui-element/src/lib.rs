pub mod events;
pub mod focus;
pub mod measure;
pub mod node;
pub mod render;
pub mod scroll;

pub use events::{HitMap, HitTarget};
pub use focus::{
    FocusHandle, FocusId, FocusState, get_focused, is_focused, next_render_focus_id,
    reset_focus_counter, set_focused, update_focus_context, use_focus,
};
pub use measure::measure_node;
pub use node::{BoxElement, IntoNode, TextElement, VNode};
pub use render::{render_node, render_node_interactive, render_node_offset};
pub use scroll::{get_scroll, set_scroll};

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;
    use svtui_layout::{BorderStyle, Overflow, Rect};
    use svtui_terminal::Buffer;

    #[test]
    fn test_measure_text() {
        let text = TextElement::new("Hello, world!").into_node();
        let (w, h) = measure_node(&text, 80, 24);
        assert_eq!(w, 13);
        assert_eq!(h, 1);
    }

    #[test]
    fn test_hit_map() {
        let mut hit_map = HitMap::new();
        hit_map.register(HitTarget {
            rect: Rect::new(5, 5, 10, 3),
            on_click: Some(Rc::new(|| {})),
            on_click_at: None,
            focus_id: Some(FocusId(1)),
            on_key: None,
            on_scroll: None,
        });

        assert!(hit_map.hit_test(6, 6).is_some());
        assert!(hit_map.hit_test(0, 0).is_none());
        assert_eq!(hit_map.focusable_ids(), vec![FocusId(1)]);
    }

    #[test]
    fn test_stable_focus_ids_across_renders() {
        reset_focus_counter();
        let id1 = next_render_focus_id();
        let id2 = next_render_focus_id();
        assert_eq!(id1, FocusId(0));
        assert_eq!(id2, FocusId(1));

        set_focused(Some(id2));

        reset_focus_counter();
        let frame2_id1 = next_render_focus_id();
        let frame2_id2 = next_render_focus_id();
        assert_eq!(frame2_id1, FocusId(0));
        assert_eq!(frame2_id2, FocusId(1));

        assert!(!is_focused(frame2_id1));
        assert!(is_focused(frame2_id2));

        // User-created IDs are globally unique and never collide with render-counter IDs
        let u1 = FocusId::new();
        let u2 = FocusId::new();
        assert!(u1.0 >= 100_000);
        assert_eq!(u2.0, u1.0 + 1);
    }

    #[test]
    fn test_use_focus_and_focus_within() {
        let mut buffer = Buffer::new(40, 10);
        let mut hit_map = HitMap::new();

        let parent_id = FocusId(100);
        let child_id = FocusId(101);

        let tree = BoxElement::new()
            .focus_id(parent_id)
            .child(
                BoxElement::new()
                    .focus_id(child_id)
                    .child(TextElement::new("Child button")),
            )
            .into_node();

        // Render to establish DOM ancestry
        render_node_interactive(&tree, Rect::new(0, 0, 40, 10), &mut buffer, &mut hit_map);

        // When child is focused:
        update_focus_context(Some(child_id), &hit_map);

        let child_state = use_focus(child_id);
        assert!(child_state.is_focused());
        assert!(!child_state.has_focus_within());
        assert!(child_state.is_active());

        // Parent should contain focus (:focus-within)
        let parent_state = use_focus(parent_id);
        assert!(!parent_state.is_focused());
        assert!(parent_state.has_focus_within());
        assert!(parent_state.is_active());

        // When parent is focused directly:
        update_focus_context(Some(parent_id), &hit_map);
        let parent_state_direct = use_focus(parent_id);
        assert!(parent_state_direct.is_focused());
        assert!(!parent_state_direct.has_focus_within());

        // When nothing is focused:
        update_focus_context(None, &hit_map);
        assert!(!use_focus(parent_id).is_active());
        assert!(!use_focus(child_id).is_active());
    }

    #[test]
    fn test_box_min_max_dimensions() {
        let b = BoxElement::new()
            .width(10)
            .min_width(20)
            .max_width(50)
            .into_node();
        let (w, _) = measure_node(&b, 100, 100);
        assert_eq!(w, 20); // Clamped to min_width 20

        let b2 = BoxElement::new()
            .width(60)
            .min_width(20)
            .max_width(50)
            .into_node();
        let (w2, _) = measure_node(&b2, 100, 100);
        assert_eq!(w2, 50); // Clamped to max_width 50
    }

    #[test]
    fn test_box_overflow_scroll() {
        let mut buffer = Buffer::new(20, 5);
        let mut hit_map = HitMap::new();

        let node = BoxElement::new()
            .width(20)
            .height(5)
            .overflow(Overflow::Scroll)
            .child(TextElement::new("Line 1"))
            .child(TextElement::new("Line 2"))
            .child(TextElement::new("Line 3"))
            .child(TextElement::new("Line 4"))
            .child(TextElement::new("Line 5"))
            .child(TextElement::new("Line 6"))
            .child(TextElement::new("Line 7"))
            .child(TextElement::new("Line 8"))
            .into_node();

        render_node_interactive(&node, Rect::new(0, 0, 20, 5), &mut buffer, &mut hit_map);

        // Content should render within bounds, and scrollbar track/thumb should be on column 19
        assert_eq!(buffer.get(0, 0).unwrap().ch, 'L');
        assert_eq!(buffer.get(1, 0).unwrap().ch, 'i');
        // Scrollbar thumb or track is drawn at x=19 with full block █
        let cell = buffer.get(19, 0).unwrap();
        assert_eq!(cell.ch, '█');
    }

    #[test]
    fn test_horizontal_scrollbar_ui_parity() {
        let mut buffer = Buffer::new(10, 5);
        let mut hit_map = HitMap::new();

        let node = BoxElement::new()
            .width(10)
            .height(5)
            .overflow_x(Overflow::Scroll)
            .child(TextElement::new(
                "A very long horizontal sentence exceeding 10 cols",
            ))
            .into_node();

        render_node_interactive(&node, Rect::new(0, 0, 10, 5), &mut buffer, &mut hit_map);

        // Horizontal scrollbar thumb or track is drawn at y=4 with matching lower half block ▄
        let cell = buffer.get(0, 4).unwrap();
        assert_eq!(cell.ch, '▄');
    }

    #[test]
    fn test_horizontal_text_scrolling_offset() {
        let mut buffer = Buffer::new(10, 3);
        let mut hit_map = HitMap::new();

        let node = BoxElement::new()
            .width(10)
            .height(3)
            .border(BorderStyle::None)
            .overflow_x(Overflow::Scroll)
            .scroll_x(5)
            .child(TextElement::new("0123456789ABCDEF"))
            .into_node();

        render_node_interactive(&node, Rect::new(0, 0, 10, 3), &mut buffer, &mut hit_map);

        // At x=0, we expect character '5' because scroll_x = 5
        assert_eq!(buffer.get(0, 0).unwrap().ch, '5');
        assert_eq!(buffer.get(1, 0).unwrap().ch, '6');
    }

    #[test]
    fn test_mouse_scroll_handler_horizontal_and_fallback() {
        let mut buffer = Buffer::new(20, 5);
        let mut hit_map = HitMap::new();

        let node = BoxElement::new()
            .width(20)
            .height(5)
            .overflow_x(Overflow::Scroll)
            .child(TextElement::new("0123456789012345678901234567890123456789"))
            .into_node();

        reset_focus_counter();
        render_node_interactive(&node, Rect::new(0, 0, 20, 5), &mut buffer, &mut hit_map);

        // Even without explicit focus_id, on_scroll handler should be registered
        let target = hit_map.hit_test(5, 2).expect("target should be hit");
        let on_scroll = target.on_scroll.as_ref().expect("on_scroll should exist");

        // Scroll horizontally by 4
        on_scroll(4, 0);

        // Re-render and verify text shifted (App calls reset_focus_counter each frame)
        reset_focus_counter();
        let mut buffer2 = Buffer::new(20, 5);
        let mut hit_map2 = HitMap::new();
        render_node_interactive(&node, Rect::new(0, 0, 20, 5), &mut buffer2, &mut hit_map2);
        assert_eq!(buffer2.get(0, 0).unwrap().ch, '4');

        // Test vertical wheel translation on horizontal-only container (dx=0, dy=1 -> horizontal scroll)
        let target2 = hit_map2.hit_test(5, 2).expect("target2 should be hit");
        let on_scroll2 = target2.on_scroll.as_ref().expect("on_scroll2 should exist");
        on_scroll2(0, 1);

        reset_focus_counter();
        let mut buffer3 = Buffer::new(20, 5);
        let mut hit_map3 = HitMap::new();
        render_node_interactive(&node, Rect::new(0, 0, 20, 5), &mut buffer3, &mut hit_map3);
        // 4 + 2 = 6
        assert_eq!(buffer3.get(0, 0).unwrap().ch, '6');
    }

    #[test]
    fn test_nested_scroll_box_scroll() {
        let scroll_id = FocusId::new(); // User calls FocusId::new() in main()
        let tree = BoxElement::new() // Root Box
            .child(
                BoxElement::new() // Dialog Box
                    .child(
                        BoxElement::new() // Scrollable Box
                            .focus_id(scroll_id)
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
    fn test_render_node_cursor_offset() {
        let input_id = FocusId::new();
        let tree = BoxElement::new()
            .focus_id(input_id)
            .border(BorderStyle::Single)
            .padding(1)
            .width(20)
            .height(3)
            .cursor_offset(3, 0)
            .into_node();

        // When not focused: cursor should not be set
        let mut buffer = Buffer::new(30, 10);
        let mut hit_map = HitMap::new();
        render_node_interactive(&tree, Rect::new(5, 2, 20, 3), &mut buffer, &mut hit_map);
        assert_eq!(buffer.cursor(), None);

        // When focused: cursor should be positioned at inner_x + 3, inner_y + 0
        // Rect(5, 2, 20, 3) with border(1) and padding(1) -> inner_x = 5 + 1 + 1 = 7, inner_y = 2 + 1 + 1 = 4
        set_focused(Some(input_id));
        let mut buffer2 = Buffer::new(30, 10);
        let mut hit_map2 = HitMap::new();
        render_node_interactive(&tree, Rect::new(5, 2, 20, 3), &mut buffer2, &mut hit_map2);
        assert_eq!(buffer2.cursor(), Some((7 + 3, 4)));

        // Clean up focus state
        set_focused(None);
    }
}
