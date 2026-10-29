use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::{cell::Cell, rc::Rc};
use svtui::{
    Buffer,
    element::{render_node_interactive, reset_focus_counter},
    layout::Rect,
    prelude::*,
};
use svtui_components::*;

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}
fn render(manager: &mut FocusManager, build: impl Fn() -> VNode) -> (Buffer, HitMap) {
    for _ in 0..5 {
        reset_focus_counter();
        let tree = manager.with_context(&build);
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
fn existing_controls_declare_focus_semantics_and_disabled_activation() {
    let mut manager = FocusManager::new();
    let count = Rc::new(Cell::new(0));
    let action = count.clone();
    let (checked, set_checked) = create_signal(false);
    let build = || {
        view! {
            <FlexBox>
                <Button id=FocusId(100001) on_click={{let action=action.clone();move || action.set(action.get()+1)}}>"Action"</Button>
                <Button id=FocusId(100002) disabled=true>"Unavailable"</Button>
                <Checkbox id=FocusId(100003) checked=checked set_checked=set_checked label="Choice" />
            </FlexBox>
        }
    };
    render(&mut manager, build);
    assert_eq!(manager.tab_order(), vec![FocusId(100001), FocusId(100003)]);
    manager.dispatch(key(KeyCode::Tab));
    manager.dispatch(key(KeyCode::Enter));
    assert_eq!(count.get(), 1);
    manager.dispatch(key(KeyCode::Tab));
    manager.dispatch(key(KeyCode::Char(' ')));
    assert!(checked.get());
    render(&mut manager, build);
    let node = manager.tree().node(FocusId(100003)).unwrap();
    assert_eq!(node.interaction.semantics.checked, Some(true));
}

#[test]
fn dropdown_nested_navigation_escape_and_tab_are_scoped_inside_dialog() {
    let (open, set_open) = create_signal(true);
    let build = || {
        view! {
            <Dialog id=FocusId(100100) open=true title="Dialog">
                <Button id=FocusId(100001)>"Menu opener"</Button>
                <DropdownMenu id=FocusId(100101) open={open.get()} on_close={move || set_open.set(false)}>
                    <DropdownItem id=FocusId(100002) label="First" />
                    <DropdownSubmenu id=FocusId(100003) layer_id=FocusId(100102) label="More">
                        <DropdownItem id=FocusId(100004) label="Nested first" />
                        <DropdownItem id=FocusId(100005) label="Nested second" />
                    </DropdownSubmenu>
                    <DropdownItem id=FocusId(100006) label="Last" />
                </DropdownMenu>
                <Button id=FocusId(100007)>"After menu"</Button>
            </Dialog>
        }
    };
    let mut manager = FocusManager::new();
    render(&mut manager, build);
    assert_eq!(manager.focused(), Some(FocusId(100002)));
    manager.dispatch(key(KeyCode::Down));
    assert_eq!(manager.focused(), Some(FocusId(100003)));
    manager.dispatch(key(KeyCode::Right));
    render(&mut manager, build);
    assert_eq!(manager.focused(), Some(FocusId(100004)));
    manager.dispatch(key(KeyCode::Down));
    assert_eq!(manager.focused(), Some(FocusId(100005)));
    manager.dispatch(key(KeyCode::Esc));
    render(&mut manager, build);
    assert_eq!(manager.focused(), Some(FocusId(100003)));
    assert!(open.get());
    manager.dispatch(key(KeyCode::Tab));
    render(&mut manager, build);
    assert!(!open.get());
    assert_eq!(manager.focused(), Some(FocusId(100007)));
    manager.dispatch(key(KeyCode::Tab));
    assert_eq!(manager.focused(), Some(FocusId(100001)));
}

#[test]
fn popover_traps_tab_in_both_directions_and_restores_on_escape() {
    let (open, set_open) = create_signal(true);
    let build = || {
        view! {
            <FlexBox>
                <Popover id=FocusId(100100) open={open.get()} on_close={move || set_open.set(false)}
                    trigger={view! { <Button id=FocusId(100001)>"Open"</Button> }}>
                    <FlexBox><Button id=FocusId(100002)>"One"</Button><Button id=FocusId(100003)>"Two"</Button></FlexBox>
                </Popover>
                <Button id=FocusId(100004)>"After"</Button>
            </FlexBox>
        }
    };
    let mut manager = FocusManager::new();
    render(&mut manager, build);
    assert_eq!(manager.focused(), Some(FocusId(100002)));
    manager.dispatch(key(KeyCode::Tab));
    assert_eq!(manager.focused(), Some(FocusId(100003)));
    manager.dispatch(key(KeyCode::Tab));
    render(&mut manager, build);
    assert!(open.get());
    assert_eq!(manager.focused(), Some(FocusId(100002)));
    manager.dispatch(key(KeyCode::BackTab));
    assert_eq!(manager.focused(), Some(FocusId(100003)));
    assert!(!manager.focus(FocusId(100004)));
    manager.dispatch(key(KeyCode::Esc));
    render(&mut manager, build);
    assert_eq!(manager.focused(), Some(FocusId(100001)));
}

#[test]
fn editor_owns_noop_edits_but_releases_shortcuts_and_tab() {
    let (value, set_value) = create_signal(String::new());
    let build = || {
        view! {
            <FlexBox><TextInput id=FocusId(100001) value=value set_value=set_value />
                <Button id=FocusId(100002)>"Next"</Button></FlexBox>
        }
    };
    let mut manager = FocusManager::new();
    render(&mut manager, build);
    manager.dispatch(key(KeyCode::Tab));
    assert!(manager.dispatch(key(KeyCode::Backspace)).handled);
    assert!(manager.dispatch(key(KeyCode::Left)).handled);
    let ctrl_c = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
    assert!(!manager.dispatch(ctrl_c).handled);
    manager.dispatch(key(KeyCode::Char('q')));
    assert_eq!(value.get(), "q");
    let mut app = App::new(build).on_key(move |key, context| {
        if key == ctrl_c {
            context.exit();
            EventResult::Handled
        } else {
            EventResult::Ignored
        }
    });
    assert!(app.dispatch_key(&mut manager, ctrl_c).exit);
    assert!(!App::new(build).dispatch_key(&mut manager, ctrl_c).exit);
    manager.dispatch(key(KeyCode::Tab));
    assert_eq!(manager.focused(), Some(FocusId(100002)));
}

#[test]
fn textarea_yields_tab_without_inserting_indentation() {
    let (value, set_value) = create_signal(String::from("text"));
    let build = || {
        view! { <FlexBox><TextArea id=FocusId(100001) value=value set_value=set_value />
        <Button id=FocusId(100002)>"Next"</Button></FlexBox> }
    };
    let mut manager = FocusManager::new();
    render(&mut manager, build);
    manager.dispatch(key(KeyCode::Tab));
    manager.dispatch(key(KeyCode::Tab));
    assert_eq!(manager.focused(), Some(FocusId(100002)));
    assert_eq!(value.get(), "text");
}

#[test]
fn offscreen_focus_is_registered_and_revealed_by_its_scroll_owner() {
    let scroll = FocusId(100010);
    let last = FocusId(100003);
    let build = || {
        view! {
            <FlexBox width=20 height=4 overflow_y=Overflow::Auto focus_id=scroll>
                <Button id=FocusId(100001)>"First"</Button>
                <Button id=FocusId(100002)>"Second"</Button>
                <Button id=last>"Last"</Button>
            </FlexBox>
        }
    };
    let mut manager = FocusManager::new();
    render(&mut manager, build);
    assert!(manager.tab_order().contains(&last));
    assert!(manager.focus(last));
    let (buffer, hits) = render(&mut manager, build);
    assert!(svtui::element::get_scroll(scroll).1 > 0);
    let target = hits
        .targets
        .iter()
        .find(|t| t.focus_id == Some(last))
        .unwrap();
    assert!(target.rect.height > 0 && target.rect.y < 4);
    assert!((0..4).all(|y| buffer.get(19, y).unwrap().ch == '█'));
}

#[test]
fn pointer_activation_and_wheel_do_not_reach_modal_background() {
    let count = Rc::new(Cell::new(0));
    let click = count.clone();
    let build = || {
        view! {
            <FlexBox>
                <Button id=FocusId(100001) on_click={{let click=click.clone();move || click.set(click.get()+1)}}>"Background"</Button>
                <Dialog id=FocusId(100100) open=true title="Modal"><Button id=FocusId(100002)>"Inside"</Button></Dialog>
            </FlexBox>
        }
    };
    let mut manager = FocusManager::new();
    let (_, hits) = render(&mut manager, build);
    let target = hits
        .targets
        .iter()
        .find(|t| t.focus_id == Some(FocusId(100001)))
        .unwrap();
    assert!(
        !manager
            .pointer_down(target, target.rect.x, target.rect.y)
            .handled
    );
    assert_eq!(count.get(), 0);
    assert_eq!(manager.focused(), Some(FocusId(100002)));
    let scrolled = Rc::new(Cell::new(0));
    let seen = scrolled.clone();
    let mut wheel_target = target.clone();
    wheel_target.on_scroll = Some(Rc::new(move |_, _| seen.set(seen.get() + 1)));
    assert!(!manager.scroll(&wheel_target, 0, 1).handled);
    assert_eq!(scrolled.get(), 0);
}

#[test]
fn selecting_a_menu_item_closes_the_menu_without_dismissing_its_dialog() {
    let (dialog, set_dialog) = create_signal(true);
    let (menu, set_menu) = create_signal(true);
    let build = || {
        view! {
            <Dialog id=FocusId(100100) open={dialog.get()} on_close={move || set_dialog.set(false)}>
                <DropdownMenu id=FocusId(100101) open={menu.get()} on_close={move || set_menu.set(false)}>
                    <DropdownItem id=FocusId(100001) label="Action" />
                </DropdownMenu>
            </Dialog>
        }
    };
    let mut manager = FocusManager::new();
    render(&mut manager, build);
    manager.dispatch(key(KeyCode::Enter));
    render(&mut manager, build);
    assert!(!menu.get());
    assert!(dialog.get());
}

#[test]
fn rejected_controlled_tab_dismissal_keeps_focus_in_the_popup() {
    let build = || {
        view! {
            <FlexBox><DropdownMenu id=FocusId(100100) open=true
                trigger={view! { <Button id=FocusId(100001)>"Open"</Button> }}>
                <Button id=FocusId(100002)>"Stay"</Button>
            </DropdownMenu><Button id=FocusId(100003)>"After"</Button></FlexBox>
        }
    };
    let mut manager = FocusManager::new();
    render(&mut manager, build);
    manager.dispatch(key(KeyCode::Tab));
    render(&mut manager, build);
    assert_eq!(manager.focused(), Some(FocusId(100002)));
}

#[test]
fn passive_popover_focuses_container_and_escape_restores_trigger() {
    let (open, set_open) = create_signal(false);
    let build = || {
        view! {
            <Popover id=FocusId(100100) open={open.get()} on_close={move || set_open.set(false)}
                trigger={view! { <Button id=FocusId(100001)>"Info"</Button> }}>
                <TextSpan>"Explanation"</TextSpan>
            </Popover>
        }
    };
    let mut manager = FocusManager::new();
    render(&mut manager, build);
    manager.focus(FocusId(100001));
    set_open.set(true);
    render(&mut manager, build);
    assert_eq!(manager.focused(), Some(FocusId(100100)));
    manager.dispatch(key(KeyCode::Tab));
    assert_eq!(manager.focused(), Some(FocusId(100100)));
    manager.dispatch(key(KeyCode::Esc));
    render(&mut manager, build);
    assert!(!open.get());
    assert_eq!(manager.focused(), Some(FocusId(100001)));
}

#[test]
fn dialog_within_menu_has_its_own_tab_sequence() {
    let build = || {
        view! {
            <DropdownMenu id=FocusId(100100) open=true>
                <DropdownItem id=FocusId(100001) label="Menu item" />
                <Dialog id=FocusId(100101) open=true>
                    <Button id=FocusId(100002)>"A"</Button><Button id=FocusId(100003)>"B"</Button>
                </Dialog>
            </DropdownMenu>
        }
    };
    let mut manager = FocusManager::new();
    render(&mut manager, build);
    assert_eq!(manager.tab_order(), vec![FocusId(100002), FocusId(100003)]);
    manager.dispatch(key(KeyCode::Tab));
    assert_eq!(manager.focused(), Some(FocusId(100003)));
    manager.dispatch(key(KeyCode::Tab));
    assert_eq!(manager.focused(), Some(FocusId(100002)));
}

#[test]
fn core_only_custom_control_has_the_same_focus_and_activation_capabilities() {
    let count = Rc::new(Cell::new(0));
    let action = count.clone();
    let build = || {
        view! {
            <FlexBox width=10 height=1 focus_id=FocusId(100001) tab_stop=true activation=Activation::Press
                semantics={Semantics::new(Role::Button,"Core action")}
                on_click={{let action=action.clone();move || action.set(action.get()+1)}}>
                <TextSpan>"Core action"</TextSpan>
            </FlexBox>
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
fn uncontrolled_menu_opens_with_keyboard_and_up_selects_the_last_item() {
    let build = || {
        view! {
            <DropdownMenu id=FocusId(100100)
                trigger={view! { <Button id=FocusId(100001)>"Open"</Button> }}>
                <DropdownItem id=FocusId(100002) label="First" />
                <DropdownItem id=FocusId(100003) label="Last" />
            </DropdownMenu>
        }
    };
    let mut manager = FocusManager::new();
    render(&mut manager, build);
    manager.focus(FocusId(100001));
    manager.dispatch(key(KeyCode::Up));
    render(&mut manager, build);
    assert!(manager.is_open(FocusId(100100)));
    assert_eq!(manager.focused(), Some(FocusId(100003)));
    manager.dispatch(key(KeyCode::Esc));
    render(&mut manager, build);
    assert!(!manager.is_open(FocusId(100100)));
    assert_eq!(manager.focused(), Some(FocusId(100001)));
    manager.dispatch(key(KeyCode::Enter));
    render(&mut manager, build);
    assert_eq!(manager.focused(), Some(FocusId(100002)));
}

#[test]
fn controlled_submenu_uses_core_open_and_close_requests() {
    let (open, set_open) = create_signal(false);
    let build = || {
        view! {
            <DropdownMenu id=FocusId(100100) open=true>
                <DropdownSubmenu id=FocusId(100001) layer_id=FocusId(100101) open={open.get()} label="More"
                    on_open={move || set_open.set(true)} on_close={move || set_open.set(false)}>
                    <DropdownItem id=FocusId(100002) label="Nested" />
                </DropdownSubmenu>
            </DropdownMenu>
        }
    };
    let mut manager = FocusManager::new();
    render(&mut manager, build);
    manager.dispatch(key(KeyCode::Right));
    render(&mut manager, build);
    assert!(open.get());
    assert_eq!(manager.focused(), Some(FocusId(100002)));
    manager.dispatch(key(KeyCode::Left));
    render(&mut manager, build);
    assert!(!open.get());
    assert_eq!(manager.focused(), Some(FocusId(100001)));
}

#[test]
fn modal_scope_can_explicitly_override_ctrl_c() {
    let build = || {
        view! {
            <Dialog id=FocusId(100100) open=true on_key={|key: KeyEvent, context: &mut EventContext| {
                if key.code==KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                    context.exit();EventResult::Handled
                } else { EventResult::Ignored }
            }} />
        }
    };
    let mut manager = FocusManager::new();
    render(&mut manager, build);
    assert!(
        manager
            .dispatch(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL))
            .exit
    );
}

#[test]
fn missing_opener_restores_to_the_nearest_surviving_scope() {
    let (open, set_open) = create_signal(true);
    let (trigger, set_trigger) = create_signal(true);
    let build = || {
        view! {
            <FlexBox>
                <Button id=FocusId(100000)>"Outside"</Button>
                <FlexBox focus_id=FocusId(100010) focus_scope=FocusScope::default()>
                    <Popover id=FocusId(100100) open={open.get()}
                        trigger={if trigger.get() { Some(view! { <Button id=FocusId(100001)>"Open"</Button> }) } else { None }}>
                        <Button id=FocusId(100002)>"Inside"</Button>
                    </Popover>
                </FlexBox>
            </FlexBox>
        }
    };
    let mut manager = FocusManager::new();
    render(&mut manager, build);
    set_trigger.set(false);
    render(&mut manager, build);
    set_open.set(false);
    render(&mut manager, build);
    assert_eq!(manager.focused(), Some(FocusId(100010)));
}

#[test]
fn queued_focus_can_target_a_control_mounted_by_the_same_event() {
    let (show, set_show) = create_signal(false);
    let build = || {
        view! {
            <FlexBox>
                <FlexBox focus_id=FocusId(100001) tab_stop=true on_key={move |key: KeyEvent, context: &mut EventContext| {
                    if key.code==KeyCode::Enter {
                        set_show.set(true);context.focus(FocusId(100002));EventResult::Handled
                    } else { EventResult::Ignored }
                }}><TextSpan>"Reveal"</TextSpan></FlexBox>
                {show.get().then(|| view! { <Button id=FocusId(100002)>"New"</Button> })}
            </FlexBox>
        }
    };
    let mut manager = FocusManager::new();
    render(&mut manager, build);
    manager.focus(FocusId(100001));
    manager.dispatch(key(KeyCode::Enter));
    render(&mut manager, build);
    assert_eq!(manager.focused(), Some(FocusId(100002)));
}

#[test]
fn menu_arrows_cycle_only_the_current_list_and_traverse_submenus_horizontally() {
    let build = || {
        view! {
            <DropdownMenu id=FocusId(100100) open=true>
                <DropdownItem id=FocusId(100001) label="First" />
                <DropdownSubmenu id=FocusId(100002) layer_id=FocusId(100101) label="More">
                    <DropdownItem id=FocusId(100004) label="Nested first" />
                    <DropdownSubmenu id=FocusId(100005) layer_id=FocusId(100102) label="Deeper">
                        <DropdownItem id=FocusId(100007) label="Deep first" />
                        <DropdownItem id=FocusId(100008) label="Deep last" />
                    </DropdownSubmenu>
                    <DropdownItem id=FocusId(100006) label="Nested last" />
                </DropdownSubmenu>
                <DropdownItem id=FocusId(100003) label="Last" />
            </DropdownMenu>
        }
    };
    let mut manager = FocusManager::new();
    render(&mut manager, build);
    for (code, expected) in [
        (KeyCode::Up, 100003),
        (KeyCode::Down, 100001),
        (KeyCode::Down, 100002),
        (KeyCode::Down, 100003),
        (KeyCode::Up, 100002),
        (KeyCode::Up, 100001),
        (KeyCode::Down, 100002),
        (KeyCode::Right, 100004),
        (KeyCode::Up, 100006),
        (KeyCode::Down, 100004),
        (KeyCode::Down, 100005),
        (KeyCode::Down, 100006),
        (KeyCode::Up, 100005),
        (KeyCode::Right, 100007),
        (KeyCode::Up, 100008),
        (KeyCode::Down, 100007),
        (KeyCode::Right, 100007),
        (KeyCode::Left, 100005),
        (KeyCode::Down, 100006),
        (KeyCode::Left, 100002),
        (KeyCode::Down, 100003),
        (KeyCode::Left, 100003),
    ] {
        manager.dispatch(key(code));
        render(&mut manager, build);
        assert_eq!(manager.focused(), Some(FocusId(expected)), "{code:?}");
    }
    assert!(!manager.is_open(FocusId(100101)));
    assert!(!manager.is_open(FocusId(100102)));
}
