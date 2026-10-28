use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::{cell::Cell, rc::Rc};
use svtui_element::*;

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}
fn button(id: usize) -> VNode {
    BoxElement::new()
        .focus_id(id)
        .tab_stop(true)
        .activation(Activation::Press)
        .into_node()
}
fn bind(manager: &mut FocusManager, node: VNode) {
    manager.reconcile(InteractionTree::from_node(&node));
}

#[test]
fn focusability_is_opt_in_and_listeners_do_not_implicitly_consume() {
    let hit = Rc::new(Cell::new(0));
    let seen = hit.clone();
    let node = BoxElement::new()
        .child(BoxElement::new().focus_id(1).on_click(|| {}))
        .child(BoxElement::new().focus_id(2).on_key(move |_, _| {
            seen.set(seen.get() + 1);
            EventResult::Ignored
        }))
        .child(button(3))
        .into_node();
    let mut manager = FocusManager::new();
    bind(&mut manager, node);
    assert_eq!(manager.tree().nodes().len(), 2);
    assert_eq!(manager.tab_order(), vec![FocusId(3)]);
    assert!(!manager.dispatch(key(KeyCode::Char('c'))).handled);
    manager.dispatch(key(KeyCode::Tab));
    assert_eq!(manager.focused(), Some(FocusId(3)));
    assert!(!manager.focus(FocusId(2)));
    assert_eq!(hit.get(), 0);
}

#[test]
fn events_bubble_through_nonfocusable_listeners_and_stop_when_handled() {
    let calls = Rc::new(Cell::new(0));
    let seen = calls.clone();
    let parent = BoxElement::new()
        .focus_id(1)
        .on_key(move |key, ctx| {
            seen.set(seen.get() + 1);
            if key.code == KeyCode::Esc {
                ctx.exit();
                EventResult::Handled
            } else {
                EventResult::Ignored
            }
        })
        .child(button(2))
        .into_node();
    let mut manager = FocusManager::new();
    bind(&mut manager, parent);
    manager.focus(FocusId(2));
    assert!(manager.dispatch(key(KeyCode::Esc)).exit);
    assert_eq!(calls.get(), 1);
    assert!(!manager.dispatch(key(KeyCode::Char('s'))).handled);
    assert_eq!(calls.get(), 2);
}

#[test]
fn group_has_one_tab_stop_and_owns_arrows_and_disabled_activation() {
    let actions = Rc::new(Cell::new(0));
    let clicked = actions.clone();
    let group = BoxElement::new()
        .focus_id(10)
        .focus_group(FocusGroup::menu())
        .child(button(1))
        .child(
            BoxElement::new()
                .focus_id(2)
                .tab_stop(true)
                .disabled(true)
                .activation(Activation::Press)
                .on_click(move || clicked.set(clicked.get() + 1)),
        )
        .child(button(3));
    let node = BoxElement::new().child(group).child(button(4)).into_node();
    let mut manager = FocusManager::new();
    bind(&mut manager, node);
    assert_eq!(manager.tab_order(), vec![FocusId(1), FocusId(4)]);
    manager.dispatch(key(KeyCode::Tab));
    manager.dispatch(key(KeyCode::Down));
    assert_eq!(manager.focused(), Some(FocusId(2)));
    assert!(manager.dispatch(key(KeyCode::Enter)).handled);
    assert_eq!(actions.get(), 0);
    manager.dispatch(key(KeyCode::Tab));
    assert_eq!(manager.focused(), Some(FocusId(4)));
    manager.dispatch(key(KeyCode::BackTab));
    assert_eq!(manager.focused(), Some(FocusId(2)));
}

#[test]
fn modal_scope_contains_navigation_and_blocks_background_keys() {
    let root = BoxElement::new()
        .child(button(1))
        .child(
            OverlayElement::new()
                .id(FocusId(100))
                .open(true)
                .layer(DismissLayer::dialog())
                .content(BoxElement::new().child(button(2)).child(button(3))),
        )
        .into_node();
    let mut manager = FocusManager::new();
    bind(&mut manager, root);
    assert_eq!(manager.focused(), Some(FocusId(2)));
    assert_eq!(manager.tab_order(), vec![FocusId(2), FocusId(3)]);
    assert!(!manager.focus(FocusId(1)));
    manager.dispatch(key(KeyCode::BackTab));
    assert_eq!(manager.focused(), Some(FocusId(3)));
    assert!(manager.dispatch(key(KeyCode::Char('q'))).handled);
    assert!(
        !manager
            .dispatch(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL))
            .exit
    );
}

#[test]
fn controlled_dismissal_is_one_action_and_restores_the_opener() {
    let closed = Rc::new(Cell::new(0));
    let count = closed.clone();
    let make = |open, count: Rc<Cell<usize>>| {
        BoxElement::new()
            .child(button(1))
            .child(
                OverlayElement::new()
                    .id(FocusId(100))
                    .opener(FocusId(1))
                    .open(open)
                    .layer(DismissLayer::dialog())
                    .on_close(move || count.set(count.get() + 1))
                    .content(button(2)),
            )
            .into_node()
    };
    let mut manager = FocusManager::new();
    bind(&mut manager, make(true, count));
    manager.dispatch(key(KeyCode::Esc));
    assert_eq!(closed.get(), 1);
    assert_eq!(manager.focused(), Some(FocusId(2))); // controlled close not yet accepted
    bind(&mut manager, make(false, closed.clone()));
    assert_eq!(manager.focused(), Some(FocusId(1)));
}

#[test]
fn menu_tab_exits_its_chain_without_losing_the_next_destination_on_reconcile() {
    let make = |open| {
        BoxElement::new()
            .child(button(1))
            .child(
                OverlayElement::new()
                    .id(FocusId(100))
                    .opener(FocusId(1))
                    .open(open)
                    .managed(true)
                    .layer(DismissLayer::menu())
                    .content(
                        BoxElement::new()
                            .focus_id(10)
                            .focus_group(FocusGroup::menu())
                            .child(button(2)),
                    ),
            )
            .child(button(3))
            .into_node()
    };
    let mut manager = FocusManager::new();
    bind(&mut manager, make(true));
    manager.dispatch(key(KeyCode::Tab));
    assert!(!manager.is_open(FocusId(100)));
    assert_eq!(manager.focused(), Some(FocusId(3)));
    bind(&mut manager, make(false));
    assert_eq!(manager.focused(), Some(FocusId(3)));
}

#[test]
fn focus_context_is_app_local_and_restored_after_nested_bindings() {
    let mut a = FocusManager::new();
    bind(&mut a, button(1));
    a.focus(FocusId(1));
    let mut b = FocusManager::new();
    bind(&mut b, button(2));
    b.focus(FocusId(2));
    assert!(!is_focused(FocusId(1)));
    a.with_context(|| {
        assert!(is_focused(FocusId(1)));
        b.with_context(|| {
            assert!(is_focused(FocusId(2)));
            assert!(!is_focused(FocusId(1)));
        });
        assert!(is_focused(FocusId(1)));
    });
    assert!(!is_focused(FocusId(1)));
}

#[test]
fn removal_uses_logical_order_and_hidden_controls_are_excluded() {
    let mut manager = FocusManager::new();
    bind(
        &mut manager,
        BoxElement::new()
            .child(button(1))
            .child(button(2))
            .child(button(3))
            .into_node(),
    );
    manager.focus(FocusId(2));
    bind(
        &mut manager,
        BoxElement::new()
            .child(button(1))
            .child(BoxElement::new().focus_id(2).tab_stop(true).hidden(true))
            .child(button(3))
            .into_node(),
    );
    assert_eq!(manager.focused(), Some(FocusId(3)));
}

#[test]
fn empty_dialog_focuses_its_container_and_plain_layout_allocates_no_records() {
    let mut manager = FocusManager::new();
    bind(
        &mut manager,
        BoxElement::new()
            .focus_id(1)
            .child(TextElement::new("plain"))
            .into_node(),
    );
    assert!(manager.tree().nodes().is_empty());
    bind(
        &mut manager,
        OverlayElement::new()
            .id(FocusId(100))
            .open(true)
            .layer(DismissLayer::dialog())
            .into_node(),
    );
    assert_eq!(manager.focused(), Some(FocusId(100)));
    manager.dispatch(key(KeyCode::Tab));
    assert_eq!(manager.focused(), Some(FocusId(100)));
}
