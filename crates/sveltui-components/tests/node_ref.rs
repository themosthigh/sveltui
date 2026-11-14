use std::cell::Cell;
use sveltui::element::*;
use sveltui::layout::Rect;
use sveltui::prelude::*;
use sveltui::terminal::Buffer;
use sveltui_components::*;

thread_local! {
    static SEEN: Cell<Option<FocusId>> = const { Cell::new(None) };
}

#[component]
fn Form() -> impl IntoNode {
    let first = node_ref();
    let second = node_ref();
    SEEN.with(|seen| seen.set(Some(second.id())));
    view! {
        <View>
            <TextInput bind:this={first} width=10 />
            <TextInput bind:this={second} width=10 />
        </View>
    }
}

fn render(manager: &mut FocusManager) {
    for _ in 0..6 {
        reset_focus_counter();
        begin_frame();
        let node = manager.with_context(|| Form(FormProps::builder().build()).into_node());
        end_frame();
        let mut hits = HitMap::new();
        let mut buffer = Buffer::new(20, 4);
        manager.with_context(|| {
            render_node_interactive(&node, Rect::new(0, 0, 20, 4), &mut buffer, &mut hits)
        });
        manager.reconcile(hits.interactions);
    }
}

#[test]
fn a_ref_keeps_its_id_and_reaches_the_bound_view() {
    let mut manager = FocusManager::new();
    render(&mut manager);
    let id = SEEN.with(|seen| seen.get()).expect("ref id");
    render(&mut manager);
    assert_eq!(SEEN.with(|seen| seen.get()), Some(id));
    assert!(manager.focus(id), "the bound input takes focus");
    assert_eq!(manager.focused(), Some(id));
}
