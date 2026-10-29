use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use sveltui::prelude::*;
use sveltui::{
    Buffer,
    element::{get_scroll, render_node_interactive, reset_focus_counter},
    layout::Rect,
    reactive::{begin_frame, end_frame},
};

#[component]
fn ScrollableContent() -> impl IntoNode {
    let id = use_hook(FocusId::new);
    view! {
        <FlexBox width=12 height=4 border=BorderStyle::Single
            focus_id=id tab_stop=true overflow=Overflow::Scroll>
            {#each (0..12) as line}
                <TextSpan>{format!("{line:02} abcdefghijklmnopqrstuvwxyz")}</TextSpan>
            {/each}
        </FlexBox>
    }
}

fn render(manager: &mut FocusManager) -> (Buffer, HitMap) {
    for _ in 0..5 {
        reset_focus_counter();
        begin_frame();
        let node = manager.with_context(|| view! { <ScrollableContent /> });
        end_frame();
        let mut buffer = Buffer::new(80, 40);
        let mut hits = HitMap::new();
        manager.with_context(|| {
            render_node_interactive(&node, Rect::new(0, 0, 80, 40), &mut buffer, &mut hits)
        });
        let changed = manager.reconcile(hits.interactions.clone());
        let revealed = manager.focused().is_some_and(|id| hits.reveal(id));
        if !changed && !revealed {
            return (buffer, hits);
        }
    }
    panic!("scroll component did not stabilize before painting");
}

#[test]
fn scrolling_survives_repainting_the_component() {
    let mut manager = FocusManager::new();
    let (before, hits) = render(&mut manager);
    let target = hits.targets.iter().find(|t| t.on_scroll.is_some()).unwrap();
    let id = target.focus_id.unwrap();
    let x = target.rect.x + 2;
    let y = target.rect.y + 1;
    let wheel_target = hits.scroll_target(x, y).unwrap();
    assert!(manager.scroll(wheel_target, 0, 2).handled);

    let (after, hits) = render(&mut manager);
    let target = hits.scroll_target(x, y).unwrap();
    assert_eq!(target.focus_id, Some(id));
    assert_eq!(get_scroll(id), (0, 2));
    assert_ne!(before.cells, after.cells);

    assert!(manager.scroll(target, 4, 0).handled);
    render(&mut manager);
    assert_eq!(get_scroll(id), (4, 2));

    manager.focus(id);
    render(&mut manager);
    assert_eq!(manager.focused(), Some(id));
    assert!(
        manager
            .dispatch(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE))
            .handled
    );
    render(&mut manager);
    assert_eq!(get_scroll(id), (4, 3));
    assert!(
        manager
            .dispatch(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE))
            .handled
    );
    render(&mut manager);
    assert_eq!(get_scroll(id), (6, 3));
}
