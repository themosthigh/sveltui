use super::*;
use sveltui::{
    Buffer,
    element::{render_node_interactive, reset_focus_counter},
    layout::Rect,
};

#[test]
fn actual_layout_demo_centers_its_capped_frame_after_resizing() {
    use sveltui::element::taffy_bridge::LayoutEngine;

    for (width, height) in [(160, 80), (80, 24), (200, 60), (160, 80)] {
        reset_focus_counter();
        begin_frame();
        let node = view! { <LayoutDemo /> };
        end_frame();
        let layout = LayoutEngine::new().compute_layout(&node, width, height);
        let frame = layout.children[0].rect;
        assert_eq!(frame.width, width.min(100));
        assert_eq!(frame.height, height.min(50));
        assert_eq!(frame.x, (width - frame.width) / 2);
        assert_eq!(frame.y, (height - frame.height) / 2);
    }
}

#[test]
fn actual_layout_footer_renders_white_foreground_on_white_background_after_resizing() {
    let mut manager = FocusManager::new();
    let footer = "Tab: Switch controls | Space/Enter: Click | Q/Esc: Quit";
    for (width, height) in [(100, 32), (80, 24), (120, 50), (100, 32)] {
        let buffer = (0..5)
            .find_map(|_| {
                reset_focus_counter();
                begin_frame();
                let node = manager.with_context(|| {
                    view! {
                        <View width={Dimension::Percent(1.0)} height={Dimension::Percent(1.0)}
                            foreground_color=Color::White background_color=Color::White>
                            <LayoutDemo />
                        </View>
                    }
                });
                end_frame();
                let mut buffer = Buffer::new(width, height);
                let mut hits = HitMap::new();
                manager.with_context(|| {
                    render_node_interactive(
                        &node,
                        Rect::new(0, 0, width, height),
                        &mut buffer,
                        &mut hits,
                    )
                });
                if !manager.reconcile(hits.interactions) {
                    return Some(buffer);
                }
                None
            })
            .expect("layout focus did not stabilize");
        let (x, y) = (0..height)
            .find_map(|y| {
                let row: String = (0..width).map(|x| buffer.get(x, y).unwrap().ch).collect();
                row.find(footer)
                    .map(|byte| (row[..byte].chars().count() as u16, y))
            })
            .unwrap_or_else(|| panic!("footer missing at {width}x{height}"));
        for offset in 0..footer.len() as u16 {
            let cell = buffer.get(x + offset, y).unwrap();
            assert_eq!(
                cell.fg,
                Some(Color::White),
                "at {width}x{height}, cell {offset}"
            );
            assert_eq!(
                cell.bg,
                Some(Color::White),
                "at {width}x{height}, cell {offset}"
            );
            assert_eq!(cell.fg, cell.bg, "foreground and background must match");
        }
    }
}
