use crate::*;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crossterm::style::Color;
use sveltui::element::*;
use sveltui::layout::*;
use sveltui::prelude::*;
use sveltui::terminal::Buffer;

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

fn render_layout(node: &VNode, width: u16, height: u16) -> (Buffer, HitMap) {
    let mut buffer = Buffer::new(width, height);
    let mut hits = HitMap::new();
    render_node_interactive(
        node,
        sveltui::layout::Rect::new(0, 0, width, height),
        &mut buffer,
        &mut hits,
    );
    (buffer, hits)
}

fn rect(hits: &HitMap, id: FocusId) -> sveltui::layout::Rect {
    hits.targets
        .iter()
        .find(|t| t.focus_id == Some(id))
        .unwrap()
        .rect
}

#[test]
fn children_supply_semantics_typeahead_and_disabled_navigation() {
    let (count, set_count) = create_signal(0);
    let build = || {
        view! {
            <Menu::Root id={FocusId(100)} default_open=true>
                <Menu::Trigger id={FocusId(1)}>"Actions"</Menu::Trigger>
                <Menu::Content>
                    <Menu::Group>
                        <Menu::GroupHeading>"Files"</Menu::GroupHeading>
                        <Menu::Item id={FocusId(2)}>"Alpha"</Menu::Item>
                        <Menu::Item id={FocusId(3)} disabled=true on_select={move || set_count.set(99)}>"Beta"</Menu::Item>
                        <Menu::Separator />
                        <Menu::Item id={FocusId(4)} on_select={move || set_count.update(|n| *n += 1)}><TextSpan>"Gamma"</TextSpan></Menu::Item>
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
            <DropdownMenu::Root id={FocusId(100)} default_open=true>
                <DropdownMenu::Trigger id={FocusId(1)}>"Actions"</DropdownMenu::Trigger>
                <DropdownMenu::Content width=16>
                    <DropdownMenu::Sub id={FocusId(101)}>
                        <DropdownMenu::SubTrigger id={FocusId(2)}>"Export"</DropdownMenu::SubTrigger>
                        <DropdownMenu::SubContent width=16>
                            <DropdownMenu::Item id={FocusId(3)}>"JSON"</DropdownMenu::Item>
                        </DropdownMenu::SubContent>
                    </DropdownMenu::Sub>
                    <DropdownMenu::Sub id={FocusId(102)}>
                        <DropdownMenu::SubTrigger id={FocusId(4)}>"Share"</DropdownMenu::SubTrigger>
                        <DropdownMenu::SubContent><DropdownMenu::Item id={FocusId(5)}>"Link"</DropdownMenu::Item></DropdownMenu::SubContent>
                    </DropdownMenu::Sub>
                    <DropdownMenu::Item id={FocusId(6)}>"Close"</DropdownMenu::Item>
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
            <ContextMenu::Root id={FocusId(100)}>
                <ContextMenu::Trigger id={FocusId(1)}>"Right click here"</ContextMenu::Trigger>
                <ContextMenu::Content width=12><ContextMenu::Item id={FocusId(2)}>"Copy"</ContextMenu::Item></ContextMenu::Content>
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
            <View>
                <MenuBar::Root id={FocusId(90)}>
                    <MenuBar::Menu id={FocusId(100)} value="file">
                        <MenuBar::Trigger id={FocusId(1)}>"File"</MenuBar::Trigger>
                        <MenuBar::Content width=14>
                            <MenuBar::Item id={FocusId(2)}>"New"</MenuBar::Item>
                            <MenuBar::Sub id={FocusId(102)}>
                                <MenuBar::SubTrigger id={FocusId(6)}>"Export"</MenuBar::SubTrigger>
                                <MenuBar::SubContent><MenuBar::Item id={FocusId(7)}>"JSON"</MenuBar::Item></MenuBar::SubContent>
                            </MenuBar::Sub>
                        </MenuBar::Content>
                    </MenuBar::Menu>
                    <MenuBar::Menu id={FocusId(101)} value="edit">
                        <MenuBar::Trigger id={FocusId(3)}>"Edit"</MenuBar::Trigger>
                        <MenuBar::Content width=14><MenuBar::Item id={FocusId(4)}>"Copy"</MenuBar::Item></MenuBar::Content>
                    </MenuBar::Menu>
                </MenuBar::Root>
                <Button id={FocusId(5)}>"After"</Button>
            </View>
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
            <DropdownMenu::Root id={FocusId(100)} open={opened.get()} on_open_change={move |next| {
                set_requests.update(|n| *n += 1);
                if next { set_opened.set(true); }
            }}>
                <DropdownMenu::Trigger id={FocusId(1)}>"Open"</DropdownMenu::Trigger>
                <DropdownMenu::Content><DropdownMenu::Item id={FocusId(2)}>"Stay"</DropdownMenu::Item></DropdownMenu::Content>
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
            <View>
                <View height={offset.get()} />
                <Pressable focus_id={FocusId(9)}>
                    <View width=8 height=1>"Anchor"</View>
                </Pressable>
                <DropdownMenu::Root id={FocusId(100)} open=true>
                    <DropdownMenu::Trigger id={FocusId(1)}>"Opener"</DropdownMenu::Trigger>
                    <DropdownMenu::Content anchor={FocusId(9)} width=12><DropdownMenu::Item id={FocusId(2)}>"Item"</DropdownMenu::Item></DropdownMenu::Content>
                </DropdownMenu::Root>
            </View>
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
            <DropdownMenu::Root id={FocusId(100)} default_open=true>
                <DropdownMenu::Trigger id={FocusId(1)}>"Open"</DropdownMenu::Trigger>
                <DropdownMenu::Content>
                    <DropdownMenu::Sub id={FocusId(101)}>
                        <DropdownMenu::SubTrigger id={FocusId(2)}>"Empty"</DropdownMenu::SubTrigger>
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
                <MenuBar::Menu id={FocusId(100)} value="file">
                    <MenuBar::Trigger id={FocusId(1)}>"File"</MenuBar::Trigger>
                    <MenuBar::Content><MenuBar::Item id={FocusId(2)}>"New"</MenuBar::Item></MenuBar::Content>
                </MenuBar::Menu>
                <MenuBar::Menu id={FocusId(101)} value="edit">
                    <MenuBar::Trigger id={FocusId(3)}>"Edit"</MenuBar::Trigger>
                    <MenuBar::Content><MenuBar::Item id={FocusId(4)}>"Copy"</MenuBar::Item></MenuBar::Content>
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
            <View>
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
            </View>
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

#[test]
fn dropdown_submenu_label_and_arrow_share_one_full_width_row() {
    let submenu = FocusId::new();
    let leaf = FocusId::new();
    let build = || {
        view! {
            <DropdownMenu::Root open=true>
                <DropdownMenu::Content width=24 placement={Placement::Anchor { x: 0, y: 0 }}>
                    <DropdownMenu::Sub>
                        <DropdownMenu::SubTrigger id=submenu>"Export a very long document label"</DropdownMenu::SubTrigger>
                        <DropdownMenu::SubContent>
                            <DropdownMenu::Item>"PDF"</DropdownMenu::Item>
                        </DropdownMenu::SubContent>
                    </DropdownMenu::Sub>
                    <DropdownMenu::Item id=leaf>"Close"</DropdownMenu::Item>
                </DropdownMenu::Content>
            </DropdownMenu::Root>
        }
    };
    let initial = build();
    let mut manager = FocusManager::new();
    manager.reconcile(InteractionTree::from_node(&initial));
    manager.focus(submenu);
    let node = manager.with_context(build);

    let (buffer, hits) = manager.with_context(|| render_layout(&node, 80, 24));
    assert_eq!(
        rect(&hits, submenu),
        sveltui::layout::Rect::new(1, 1, 22, 1)
    );
    assert_eq!(rect(&hits, leaf).y, 2);
    assert_eq!(buffer.get(21, 1).unwrap().ch, '›');
    for x in 1..=22 {
        assert_eq!(buffer.get(x, 1).unwrap().bg, Some(Color::White));
        assert_eq!(hits.hit_test(x, 1).unwrap().focus_id, Some(submenu));
    }
    assert_eq!(buffer.get(23, 1).unwrap().ch, '│');
}

#[test]
fn grouped_menu_rows_fill_the_content_viewport_for_every_menu_kind() {
    macro_rules! content {
        ($parts:ident, $width:expr) => {
            view! {
                <$parts::Content width={$width} padding={Padding::symmetric(1, 0)}
                    placement={Placement::Anchor { x: 0, y: 0 }}>
                    <$parts::Group>
                        <$parts::GroupHeading>"Files"</$parts::GroupHeading>
                        <$parts::Item id={FocusId(11)}>"Copy"</$parts::Item>
                        <$parts::Group>
                            <$parts::Item id={FocusId(12)}>"Cut"</$parts::Item>
                        </$parts::Group>
                        <$parts::Sub open=true>
                            <$parts::SubTrigger id={FocusId(13)}>"Export"</$parts::SubTrigger>
                            <$parts::SubContent width={$width} padding={Padding::symmetric(1, 0)}
                                placement={Placement::Anchor { x: 0, y: 12 }}>
                                <$parts::Group>
                                    <$parts::Item id={FocusId(15)}>"JSON"</$parts::Item>
                                </$parts::Group>
                            </$parts::SubContent>
                        </$parts::Sub>
                    </$parts::Group>
                    <$parts::Separator />
                    <$parts::Item id={FocusId(14)}>"Ungrouped"</$parts::Item>
                </$parts::Content>
            }
        };
    }
    for kind in 0..4 {
        for (menu_width, screen_width) in [(24u16, 80u16), (36, 80), (36, 30)] {
            let build = || match kind {
                0 => view! { <Menu::Root open=true>{content!(Menu, menu_width)}</Menu::Root> },
                1 => {
                    view! { <DropdownMenu::Root open=true>{content!(DropdownMenu, menu_width)}</DropdownMenu::Root> }
                }
                2 => {
                    view! { <ContextMenu::Root open=true>{content!(ContextMenu, menu_width)}</ContextMenu::Root> }
                }
                _ => view! {
                    <MenuBar::Root default_value="file">
                        <MenuBar::Menu value="file">{content!(MenuBar, menu_width)}</MenuBar::Menu>
                    </MenuBar::Root>
                },
            };
            sveltui::element::reset_focus_counter();
            begin_frame();
            let initial = build();
            end_frame();
            let mut manager = FocusManager::new();
            manager.reconcile(InteractionTree::from_node(&initial));
            manager.focus(FocusId(11));
            sveltui::element::reset_focus_counter();
            begin_frame();
            let node = manager.with_context(build);
            end_frame();
            let (buffer, hits) = manager.with_context(|| render_layout(&node, screen_width, 24));
            // Border + content padding. A menu wider than the screen scrolls
            // sideways, and its two bars each take a cell from the rows.
            let bars = u16::from(menu_width > screen_width);
            let expected_width = menu_width.min(screen_width) - 4 - bars;
            for id in [11, 12, 13, 14, 15] {
                let row = rect(&hits, FocusId(id));
                if bars > 0 && row.height == 0 {
                    // The sideways bar took the bottom row; scroll to reach it.
                    continue;
                }
                assert_eq!(row.x, 2, "kind {kind}, item {id}");
                assert_eq!(row.width, expected_width, "kind {kind}, item {id}");
                assert_eq!(row.height, 1);
            }
            let focused = rect(&hits, FocusId(11));
            for x in focused.x..focused.x + focused.width {
                assert_eq!(buffer.get(x, focused.y).unwrap().bg, Some(Color::White));
                // An open submenu adds a dismiss backdrop over its parent;
                // menu pointer routing looks through that backdrop.
                let target = hits
                    .targets
                    .iter()
                    .rev()
                    .find(|target| target.dismiss.is_none() && target.rect.contains(x, focused.y))
                    .unwrap();
                assert_eq!(target.focus_id, Some(FocusId(11)));
            }
        }
    }
}

#[test]
fn menu_defaults_blend_borders_and_dim_disabled_custom_children() {
    for (bg, border_override) in [
        (Color::Black, None),
        (Color::DarkBlue, None),
        (Color::DarkBlue, Some(Color::White)),
    ] {
        let node = view! {
            <Menu::Root open=true>
                <Menu::Content bg=bg border_color=border_override width=20
                    placement={Placement::Anchor { x: 0, y: 0 }}>
                    <Menu::Item disabled=true>
                        <View flex_direction=FlexDirection::Row>
                            <TextSpan foreground_color=Color::White bold=true>"Disabled"</TextSpan>
                            <View><TextSpan foreground_color=Color::Red bold=true>"Nested"</TextSpan></View>
                        </View>
                    </Menu::Item>
                    <Menu::Sub>
                        <Menu::SubTrigger disabled=true>
                            <TextSpan foreground_color=Color::White bold=true>"Unavailable"</TextSpan>
                        </Menu::SubTrigger>
                        <Menu::SubContent />
                    </Menu::Sub>
                </Menu::Content>
            </Menu::Root>
        };
        let (buffer, hits) = render_layout(&node, 80, 24);
        assert_eq!(
            buffer.get(0, 0).unwrap().fg,
            Some(border_override.unwrap_or(bg))
        );
        assert_eq!(buffer.get(0, 0).unwrap().bg, Some(bg));
        for node in hits
            .interactions
            .nodes()
            .iter()
            .filter(|node| node.interaction.disabled)
        {
            let row = rect(&hits, node.id);
            for x in row.x..row.x + row.width {
                let cell = buffer.get(x, row.y).unwrap();
                if cell.ch != ' ' {
                    assert_eq!(cell.fg, Some(Color::DarkGrey));
                    assert!(!cell.bold);
                }
            }
        }
        assert_eq!(buffer.get(17, 2).unwrap().ch, '›');
        let trigger = hits
            .interactions
            .nodes()
            .iter()
            .find(|node| node.interaction.semantics.label == "Unavailable")
            .unwrap();
        assert!(trigger.interaction.disabled);
    }
}

#[test]
fn test_dropdown_component_into_node() {
    let dropdown = view! {
        <DropdownMenu::Root open=true>
            <DropdownMenu::Trigger>"File"</DropdownMenu::Trigger>
            <DropdownMenu::Content>
                <DropdownMenu::Item>"New File"</DropdownMenu::Item>
                <DropdownMenu::Item>"Save"</DropdownMenu::Item>
            </DropdownMenu::Content>
        </DropdownMenu::Root>
    };
    let tree = sveltui::element::InteractionTree::from_node(&dropdown);
    assert_eq!(tree.layers().len(), 1);
    assert!(tree.layers()[0].active);
    assert!(tree.layers()[0].opener.is_some());
    assert_eq!(
        tree.nodes()
            .iter()
            .filter(|n| n.interaction.semantics.role == sveltui::element::Role::MenuItem)
            .count(),
        2
    );
}

#[test]
fn test_nested_dropdown_submenus_3_level() {
    let node = view! {
        <DropdownMenu::Root open=true>
            <DropdownMenu::Trigger>"File"</DropdownMenu::Trigger>
            <DropdownMenu::Content>
                <DropdownMenu::Item>"New Project"</DropdownMenu::Item>
                <DropdownMenu::Sub open=true>
                    <DropdownMenu::SubTrigger>"Export As"</DropdownMenu::SubTrigger>
                    <DropdownMenu::SubContent>
                        <DropdownMenu::Item>"Image (PNG)"</DropdownMenu::Item>
                        <DropdownMenu::Sub open=true>
                            <DropdownMenu::SubTrigger>"Document"</DropdownMenu::SubTrigger>
                            <DropdownMenu::SubContent>
                                <DropdownMenu::Item>"PDF Document (*.pdf)"</DropdownMenu::Item>
                                <DropdownMenu::Item>"Markdown (*.md)"</DropdownMenu::Item>
                            </DropdownMenu::SubContent>
                        </DropdownMenu::Sub>
                    </DropdownMenu::SubContent>
                </DropdownMenu::Sub>
            </DropdownMenu::Content>
        </DropdownMenu::Root>
    };
    let tree = sveltui::element::InteractionTree::from_node(&node);
    assert_eq!(tree.layers().len(), 3);
    assert!(tree.layers().iter().all(|layer| layer.active));
    let pdf = tree
        .nodes()
        .iter()
        .find(|n| n.interaction.semantics.label == "PDF Document (*.pdf)")
        .unwrap();
    assert_eq!(
        tree.layers()
            .iter()
            .filter(|layer| tree.contains(pdf.id, layer.id))
            .count(),
        3
    );
}
