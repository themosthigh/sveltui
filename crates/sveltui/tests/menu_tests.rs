use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use sveltui::{
    Buffer,
    element::{render_node_interactive, reset_focus_counter},
    layout::Rect,
    prelude::*,
};
use sveltui_components::{Button, ButtonProps, ContextMenu, DropdownMenu, Menu, MenuBar};

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}
fn render(manager: &mut FocusManager, build: &impl Fn() -> VNode) -> HitMap {
    render_size(manager, build, 80, 24)
}
fn render_size(
    manager: &mut FocusManager,
    build: &impl Fn() -> VNode,
    width: u16,
    height: u16,
) -> HitMap {
    for _ in 0..8 {
        reset_focus_counter();
        begin_frame();
        let node = manager.with_context(build);
        end_frame();
        let mut hits = HitMap::new();
        let mut buffer = Buffer::new(width, height);
        manager.with_context(|| {
            render_node_interactive(
                &node,
                Rect::new(0, 0, width, height),
                &mut buffer,
                &mut hits,
            )
        });
        let changed = manager.reconcile(hits.interactions.clone());
        let revealed = manager.focused().is_some_and(|id| hits.reveal(id));
        if !changed && !revealed {
            return hits;
        }
    }
    panic!("menu focus did not stabilize");
}
fn position(hits: &HitMap, id: usize) -> (u16, u16) {
    let rect = hits
        .targets
        .iter()
        .rev()
        .find(|t| t.focus_id == Some(FocusId(id)))
        .expect("visible target")
        .rect;
    (rect.x, rect.y)
}
fn hover(manager: &mut FocusManager, hits: &HitMap, id: usize) {
    let (x, y) = position(hits, id);
    manager.pointer_move(hits, x, y);
}

#[test]
fn children_supply_semantics_typeahead_and_disabled_navigation() {
    let (count, set_count) = create_signal(0);
    let build = || {
        view! {
            <Menu::Root id=FocusId(100) default_open=true>
                <Menu::Trigger id=FocusId(1)>"Actions"</Menu::Trigger>
                <Menu::Content>
                    <Menu::Group>
                        <Menu::GroupHeading>"Files"</Menu::GroupHeading>
                        <Menu::Item id=FocusId(2)>"Alpha"</Menu::Item>
                        <Menu::Item id=FocusId(3) disabled=true on_select={move || set_count.set(99)}>"Beta"</Menu::Item>
                        <Menu::Separator />
                        <Menu::Item id=FocusId(4) on_select={move || set_count.update(|n| *n += 1)}><TextSpan>"Gamma"</TextSpan></Menu::Item>
                    </Menu::Group>
                </Menu::Content>
            </Menu::Root>
        }
    };
    let mut manager = FocusManager::new();
    render(&mut manager, &build);
    assert_eq!(manager.focused(), Some(FocusId(2)));
    assert_eq!(
        manager
            .tree()
            .node(FocusId(100))
            .unwrap()
            .interaction
            .semantics
            .label,
        "Actions"
    );
    manager.dispatch(key(KeyCode::Down));
    assert_eq!(manager.focused(), Some(FocusId(4)));
    manager.dispatch(key(KeyCode::Home));
    assert_eq!(manager.focused(), Some(FocusId(2)));
    manager.dispatch(key(KeyCode::Char('g')));
    assert_eq!(manager.focused(), Some(FocusId(4)));
    manager.dispatch(key(KeyCode::Enter));
    render(&mut manager, &build);
    assert_eq!(count.get(), 1);
    assert!(!manager.is_open(FocusId(100)));
    assert_eq!(manager.focused(), Some(FocusId(1)));
}

#[test]
fn hover_preserves_parent_until_child_entry_and_closes_competing_branches() {
    let build = || {
        view! {
            <DropdownMenu::Root id=FocusId(100) default_open=true>
                <DropdownMenu::Trigger id=FocusId(1)>"Actions"</DropdownMenu::Trigger>
                <DropdownMenu::Content width=16>
                    <DropdownMenu::Sub id=FocusId(101)>
                        <DropdownMenu::SubTrigger id=FocusId(2)>"Export"</DropdownMenu::SubTrigger>
                        <DropdownMenu::SubContent width=16>
                            <DropdownMenu::Item id=FocusId(3)>"JSON"</DropdownMenu::Item>
                        </DropdownMenu::SubContent>
                    </DropdownMenu::Sub>
                    <DropdownMenu::Sub id=FocusId(102)>
                        <DropdownMenu::SubTrigger id=FocusId(4)>"Share"</DropdownMenu::SubTrigger>
                        <DropdownMenu::SubContent><DropdownMenu::Item id=FocusId(5)>"Link"</DropdownMenu::Item></DropdownMenu::SubContent>
                    </DropdownMenu::Sub>
                    <DropdownMenu::Item id=FocusId(6)>"Close"</DropdownMenu::Item>
                </DropdownMenu::Content>
            </DropdownMenu::Root>
        }
    };
    let mut manager = FocusManager::new();
    let hits = render(&mut manager, &build);
    hover(&mut manager, &hits, 2);
    let hits = render(&mut manager, &build);
    assert!(manager.is_open(FocusId(101)));
    assert_eq!(manager.focused(), Some(FocusId(2)));
    hover(&mut manager, &hits, 3);
    assert_eq!(manager.focused(), Some(FocusId(3)));
    manager.pointer_move(&hits, 79, 23);
    render(&mut manager, &build);
    assert!(manager.is_open(FocusId(101)));
    hover(&mut manager, &hits, 4);
    let hits = render(&mut manager, &build);
    assert!(!manager.is_open(FocusId(101)));
    assert!(manager.is_open(FocusId(102)));
    assert_eq!(manager.focused(), Some(FocusId(4)));
    hover(&mut manager, &hits, 6);
    render(&mut manager, &build);
    assert!(!manager.is_open(FocusId(102)));
    assert_eq!(manager.focused(), Some(FocusId(6)));
}

#[test]
fn context_menu_uses_pointer_coordinates_and_keyboard_reanchors_to_trigger() {
    let build = || {
        view! {
            <ContextMenu::Root id=FocusId(100)>
                <ContextMenu::Trigger id=FocusId(1)>"Right click here"</ContextMenu::Trigger>
                <ContextMenu::Content width=12><ContextMenu::Item id=FocusId(2)>"Copy"</ContextMenu::Item></ContextMenu::Content>
            </ContextMenu::Root>
        }
    };
    let mut manager = FocusManager::new();
    let hits = render(&mut manager, &build);
    let (x, y) = position(&hits, 1);
    manager.pointer_down_at(&hits, x, y);
    render(&mut manager, &build);
    assert!(!manager.is_open(FocusId(100)));
    manager.context_menu(&hits, x + 4, y);
    let hits = render(&mut manager, &build);
    assert_eq!(position(&hits, 2), (x + 5, y + 1));
    assert_eq!(manager.focused(), Some(FocusId(2)));
    manager.dispatch(key(KeyCode::Esc));
    render(&mut manager, &build);
    assert_eq!(manager.focused(), Some(FocusId(1)));
    manager.dispatch(KeyEvent::new(KeyCode::F(10), KeyModifiers::SHIFT));
    let hits = render(&mut manager, &build);
    assert_eq!(position(&hits, 2), (x + 1, y + 2));
}

#[test]
fn menu_bar_has_one_tab_stop_and_coordinates_keyboard_and_hover_switching() {
    let build = || {
        view! {
            <FlexBox>
                <MenuBar::Root id=FocusId(90)>
                    <MenuBar::Menu id=FocusId(100) value="file">
                        <MenuBar::Trigger id=FocusId(1)>"File"</MenuBar::Trigger>
                        <MenuBar::Content width=14>
                            <MenuBar::Item id=FocusId(2)>"New"</MenuBar::Item>
                            <MenuBar::Sub id=FocusId(102)>
                                <MenuBar::SubTrigger id=FocusId(6)>"Export"</MenuBar::SubTrigger>
                                <MenuBar::SubContent><MenuBar::Item id=FocusId(7)>"JSON"</MenuBar::Item></MenuBar::SubContent>
                            </MenuBar::Sub>
                        </MenuBar::Content>
                    </MenuBar::Menu>
                    <MenuBar::Menu id=FocusId(101) value="edit">
                        <MenuBar::Trigger id=FocusId(3)>"Edit"</MenuBar::Trigger>
                        <MenuBar::Content width=14><MenuBar::Item id=FocusId(4)>"Copy"</MenuBar::Item></MenuBar::Content>
                    </MenuBar::Menu>
                </MenuBar::Root>
                <Button id=FocusId(5)>"After"</Button>
            </FlexBox>
        }
    };
    let mut manager = FocusManager::new();
    let hits = render(&mut manager, &build);
    assert_eq!(manager.tab_order(), vec![FocusId(1), FocusId(5)]);
    hover(&mut manager, &hits, 3);
    render(&mut manager, &build);
    assert!(!manager.is_open(FocusId(101)));
    manager.focus(FocusId(1));
    manager.dispatch(key(KeyCode::Down));
    render(&mut manager, &build);
    assert_eq!(manager.focused(), Some(FocusId(2)));
    manager.dispatch(key(KeyCode::Right));
    let hits = render(&mut manager, &build);
    assert!(!manager.is_open(FocusId(100)));
    assert!(manager.is_open(FocusId(101)));
    assert_eq!(manager.focused(), Some(FocusId(4)));
    hover(&mut manager, &hits, 1);
    let hits = render(&mut manager, &build);
    assert!(manager.is_open(FocusId(100)));
    assert!(!manager.is_open(FocusId(101)));
    hover(&mut manager, &hits, 6);
    render(&mut manager, &build);
    manager.dispatch(key(KeyCode::Right));
    render(&mut manager, &build);
    assert_eq!(manager.focused(), Some(FocusId(7)));
    manager.dispatch(key(KeyCode::Left));
    render(&mut manager, &build);
    assert_eq!(manager.focused(), Some(FocusId(6)));
    manager.dispatch(key(KeyCode::Tab));
    render(&mut manager, &build);
    assert!(!manager.is_open(FocusId(100)));
    assert_eq!(manager.focused(), Some(FocusId(5)));
}

#[test]
fn controlled_requests_are_once_per_event_and_rejected_close_retains_focus() {
    let (opened, set_opened) = create_signal(false);
    let (requests, set_requests) = create_signal(0);
    let build = || {
        view! {
            <DropdownMenu::Root id=FocusId(100) open={opened.get()} on_open_change={move |next| {
                set_requests.update(|n| *n += 1);
                if next { set_opened.set(true); }
            }}>
                <DropdownMenu::Trigger id=FocusId(1)>"Open"</DropdownMenu::Trigger>
                <DropdownMenu::Content><DropdownMenu::Item id=FocusId(2)>"Stay"</DropdownMenu::Item></DropdownMenu::Content>
            </DropdownMenu::Root>
        }
    };
    let mut manager = FocusManager::new();
    render(&mut manager, &build);
    manager.focus(FocusId(1));
    manager.dispatch(key(KeyCode::Enter));
    render(&mut manager, &build);
    assert_eq!(requests.get(), 1);
    manager.dispatch(key(KeyCode::Esc));
    render(&mut manager, &build);
    assert_eq!(requests.get(), 2);
    assert!(manager.is_open(FocusId(100)));
    assert_eq!(manager.focused(), Some(FocusId(2)));
}

#[test]
fn explicit_anchor_tracks_layout_and_clamps_after_resize() {
    let (offset, set_offset) = create_signal(3u16);
    let build = || {
        view! {
            <FlexBox>
                <FlexBox height={offset.get()} />
                <FlexBox focus_id=FocusId(9) width=8 height=1>"Anchor"</FlexBox>
                <DropdownMenu::Root id=FocusId(100) open=true>
                    <DropdownMenu::Trigger id=FocusId(1)>"Opener"</DropdownMenu::Trigger>
                    <DropdownMenu::Content anchor=FocusId(9) width=12><DropdownMenu::Item id=FocusId(2)>"Item"</DropdownMenu::Item></DropdownMenu::Content>
                </DropdownMenu::Root>
            </FlexBox>
        }
    };
    let mut manager = FocusManager::new();
    let hits = render(&mut manager, &build);
    assert_eq!(position(&hits, 2), (1, 5));
    set_offset.set(6);
    let hits = render(&mut manager, &build);
    assert_eq!(position(&hits, 2), (1, 8));
    let hits = render_size(&mut manager, &build, 10, 8);
    assert!(
        hits.targets
            .iter()
            .all(|t| t.rect.x + t.rect.width <= 10 && t.rect.y + t.rect.height <= 8)
    );
    assert_eq!(manager.tree().layers()[0].opener, Some(FocusId(1)));
}

#[test]
fn empty_menu_is_dismissible_and_outside_click_closes_the_full_chain() {
    let build = || {
        view! {
            <DropdownMenu::Root id=FocusId(100) default_open=true>
                <DropdownMenu::Trigger id=FocusId(1)>"Open"</DropdownMenu::Trigger>
                <DropdownMenu::Content>
                    <DropdownMenu::Sub id=FocusId(101)>
                        <DropdownMenu::SubTrigger id=FocusId(2)>"Empty"</DropdownMenu::SubTrigger>
                        <DropdownMenu::SubContent />
                    </DropdownMenu::Sub>
                </DropdownMenu::Content>
            </DropdownMenu::Root>
        }
    };
    let mut manager = FocusManager::new();
    render(&mut manager, &build);
    manager.dispatch(key(KeyCode::Right));
    let hits = render(&mut manager, &build);
    assert_eq!(manager.focused(), Some(FocusId(101)));
    manager.pointer_down_at(&hits, 79, 23);
    render(&mut manager, &build);
    assert!(!manager.is_open(FocusId(100)));
    assert!(!manager.is_open(FocusId(101)));
    assert_eq!(manager.focused(), Some(FocusId(1)));
}

#[test]
fn controlled_bar_switch_emits_one_new_value_without_an_intermediate_close() {
    let (active, set_active) = create_signal(None::<String>);
    let (requests, set_requests) = create_signal(Vec::<Option<String>>::new());
    let build = || {
        view! {
            <MenuBar::Root value={active.get()} on_value_change={move |next: Option<String>| {
                set_requests.update(|values| values.push(next.clone()));
                set_active.set(next);
            }}>
                <MenuBar::Menu id=FocusId(100) value="file">
                    <MenuBar::Trigger id=FocusId(1)>"File"</MenuBar::Trigger>
                    <MenuBar::Content><MenuBar::Item id=FocusId(2)>"New"</MenuBar::Item></MenuBar::Content>
                </MenuBar::Menu>
                <MenuBar::Menu id=FocusId(101) value="edit">
                    <MenuBar::Trigger id=FocusId(3)>"Edit"</MenuBar::Trigger>
                    <MenuBar::Content><MenuBar::Item id=FocusId(4)>"Copy"</MenuBar::Item></MenuBar::Content>
                </MenuBar::Menu>
            </MenuBar::Root>
        }
    };
    let mut manager = FocusManager::new();
    render(&mut manager, &build);
    manager.focus(FocusId(1));
    manager.dispatch(key(KeyCode::Down));
    render(&mut manager, &build);
    manager.dispatch(key(KeyCode::Right));
    render(&mut manager, &build);
    assert_eq!(
        requests.get(),
        vec![Some("file".into()), Some("edit".into())]
    );
    assert_eq!(manager.focused(), Some(FocusId(4)));
    // Hover leaves focus on the bar trigger; arrows must still switch menus.
    let hits = render(&mut manager, &build);
    hover(&mut manager, &hits, 1);
    render(&mut manager, &build);
    assert_eq!(manager.focused(), Some(FocusId(1)));
    manager.dispatch(key(KeyCode::Right));
    render(&mut manager, &build);
    assert_eq!(manager.focused(), Some(FocusId(4)));
    assert_eq!(active.get(), Some("edit".into()));
}

#[test]
fn generated_ids_and_sibling_menu_state_survive_opening_and_closing() {
    let build = || {
        view! {
            <FlexBox>
                <DropdownMenu::Root>
                    <DropdownMenu::Trigger>"First menu"</DropdownMenu::Trigger>
                    <DropdownMenu::Content>
                        <DropdownMenu::Item>"First action"</DropdownMenu::Item>
                    </DropdownMenu::Content>
                </DropdownMenu::Root>
                <DropdownMenu::Root>
                    <DropdownMenu::Trigger>"Second menu"</DropdownMenu::Trigger>
                    <DropdownMenu::Content>
                        <DropdownMenu::Item>"Second action"</DropdownMenu::Item>
                    </DropdownMenu::Content>
                </DropdownMenu::Root>
            </FlexBox>
        }
    };
    fn named(manager: &FocusManager, name: &str) -> FocusId {
        manager
            .tree()
            .nodes()
            .iter()
            .find(|node| node.interaction.semantics.label == name)
            .unwrap()
            .id
    }
    let mut manager = FocusManager::new();
    render(&mut manager, &build);
    let first = named(&manager, "First menu");
    let second = named(&manager, "Second menu");
    manager.focus(first);
    manager.dispatch(key(KeyCode::Down));
    render(&mut manager, &build);
    let action = named(&manager, "First action");
    assert_eq!(manager.focused(), Some(action));
    render(&mut manager, &build);
    assert_eq!(named(&manager, "First action"), action);
    assert_eq!(named(&manager, "Second menu"), second);
    manager.dispatch(key(KeyCode::Esc));
    render(&mut manager, &build);
    assert_eq!(manager.focused(), Some(first));
    manager.focus(second);
    manager.dispatch(key(KeyCode::Down));
    render(&mut manager, &build);
    assert_eq!(manager.focused(), Some(named(&manager, "Second action")));
    assert!(!manager.tree().layers()[0].active);
    assert!(manager.tree().layers()[1].active);
    assert_eq!(named(&manager, "First menu"), first);
}
