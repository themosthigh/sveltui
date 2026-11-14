use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::{cell::Cell, rc::Rc};
use sveltui::{
    Buffer,
    element::{render_node_interactive, reset_focus_counter},
    layout::Rect,
    prelude::*,
};

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn render(manager: &mut FocusManager, build: impl Fn() -> VNode) -> (Buffer, HitMap) {
    for _ in 0..5 {
        reset_focus_counter();
        begin_frame();
        let tree = manager.with_context(&build);
        end_frame();
        let mut buffer = Buffer::new(80, 24);
        let mut hits = HitMap::new();
        manager.with_context(|| {
            render_node_interactive(&tree, Rect::new(0, 0, 80, 24), &mut buffer, &mut hits)
        });
        let changed = manager.reconcile(hits.interactions.clone());
        let revealed = manager.focused().is_some_and(|id| hits.reveal(id));
        if !changed && !revealed {
            return (buffer, hits);
        }
    }
    panic!("focus did not stabilize before painting");
}

#[test]
fn core_only_custom_control_has_the_same_focus_and_activation_capabilities() {
    let count = Rc::new(Cell::new(0));
    let action = count.clone();
    let build = || {
        view! {
            <Pressable focus_id={FocusId(100001)} tab_stop=true activation=Activation::Press
                semantics={Semantics::new(Role::Button,"Core action")}
                on_click={{let action=action.clone();move || action.set(action.get()+1)}}>
                <View width=10 height=1>
                    <TextSpan>"Core action"</TextSpan>
                </View>
            </Pressable>
        }
    };
    let mut manager = FocusManager::new();
    let (_, hits) = render(&mut manager, build);
    manager.dispatch(key(KeyCode::Tab));
    manager.dispatch(key(KeyCode::Enter));
    assert_eq!(count.get(), 1);
    let target = hits
        .targets
        .iter()
        .find(|t| t.focus_id == Some(FocusId(100001)))
        .unwrap();
    manager.pointer_down(target, 0, 0);
    assert_eq!(count.get(), 2);
}

#[test]
fn on_focus_fires_once_when_input_moves_focus_and_never_from_render() {
    let log = Rc::new(std::cell::RefCell::new(Vec::new()));
    let item = |id: usize, log: &Rc<std::cell::RefCell<Vec<usize>>>| {
        let log = log.clone();
        view! {
            <Pressable focus_id={FocusId(id)} tab_stop=true
                on_focus={move || log.borrow_mut().push(id)}>
                <TextSpan>{format!("item {id}")}</TextSpan>
            </Pressable>
        }
    };
    let build = || {
        let group = FocusGroup {
            orientation: Orientation::Horizontal,
            wrap: true,
            typeahead: false,
            disabled_focusable: false,
        };
        view! {
            <Pressable focus_group=group>
                <View flex_direction=FlexDirection::Row gap=1>
                    {item(1, &log)}
                    {item(2, &log)}
                </View>
            </Pressable>
        }
    };
    let mut manager = FocusManager::new();
    render(&mut manager, build);
    assert!(log.borrow().is_empty(), "rendering never fires on_focus");

    manager.dispatch(key(KeyCode::Tab));
    render(&mut manager, build);
    manager.dispatch(key(KeyCode::Right));
    render(&mut manager, build);
    render(&mut manager, build);
    assert_eq!(*log.borrow(), vec![1, 2]);

    // Keys that do not move focus do not fire it again.
    manager.dispatch(key(KeyCode::Char('x')));
    // Programmatic focus is not input and does not fire it.
    manager.focus(FocusId(1));
    render(&mut manager, build);
    assert_eq!(*log.borrow(), vec![1, 2]);
}
