use super::{RootProps, RootPropsBuilder};
use crate::*;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::{cell::Cell, rc::Rc};
use sveltui::element::*;
use sveltui::layout::Rect;
use sveltui::prelude::*;
use sveltui::terminal::Buffer;

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

/// Renders until focus and layout settle, like a real frame loop.
fn render(manager: &mut FocusManager, build: &impl Fn() -> VNode) -> (String, HitMap) {
    for _ in 0..8 {
        reset_focus_counter();
        begin_frame();
        let node = manager.with_context(build);
        end_frame();
        let mut hits = HitMap::new();
        let mut buffer = Buffer::new(60, 12);
        manager.with_context(|| {
            render_node_interactive(&node, Rect::new(0, 0, 60, 12), &mut buffer, &mut hits)
        });
        let changed = manager.reconcile(hits.interactions.clone());
        if !changed {
            let text = (0..buffer.height)
                .map(|y| {
                    (0..buffer.width)
                        .map(|x| buffer.get(x, y).unwrap().ch)
                        .collect::<String>()
                })
                .collect::<Vec<_>>()
                .join("\n");
            return (text, hits);
        }
    }
    panic!("tabs focus did not stabilize");
}

/// Sends `code` and re-renders; returns the visible text.
fn press(manager: &mut FocusManager, build: &impl Fn() -> VNode, code: KeyCode) -> String {
    manager.dispatch(key(code));
    render(manager, build).0
}

/// Three tabs `a`, `b`, `c` with trigger ids 1-3.
fn three_tabs(root: RootPropsBuilder) -> VNode {
    Tabs::Root(
        root.children(Snippet::new(|_| {
            view! {
                <View>
                <Tabs::List>
                    <Tabs::Trigger value="a" id={FocusId(1)}>"Alpha"</Tabs::Trigger>
                    <Tabs::Trigger value="b" id={FocusId(2)}>"Beta"</Tabs::Trigger>
                    <Tabs::Trigger value="c" id={FocusId(3)}>"Gamma"</Tabs::Trigger>
                </Tabs::List>
                <Tabs::Content value="a"><TextSpan>"panel-a"</TextSpan></Tabs::Content>
                <Tabs::Content value="b"><TextSpan>"panel-b"</TextSpan></Tabs::Content>
                <Tabs::Content value="c"><TextSpan>"panel-c"</TextSpan></Tabs::Content>
                </View>
            }
        }))
        .build(),
    )
}

/// The panel id a tab points at through `controls`.
fn panel_of(manager: &FocusManager, tab: usize) -> FocusId {
    manager
        .tree()
        .node(FocusId(tab))
        .and_then(|n| n.interaction.semantics.controls)
        .expect("tab controls a panel")
}

fn visible_panel(text: &str) -> &'static str {
    let shown: Vec<_> = ["panel-a", "panel-b", "panel-c"]
        .into_iter()
        .filter(|p| text.contains(p))
        .collect();
    assert_eq!(shown.len(), 1, "exactly one panel should render: {text}");
    shown[0]
}

#[test]
fn tablist_is_one_tab_stop_that_lands_on_the_selected_tab_then_the_panel() {
    let build = || {
        view! {
            <View>
                <Button id={FocusId(100)}>"Before"</Button>
                {three_tabs(RootProps::builder().default_value("b"))}
            </View>
        }
    };
    let mut manager = FocusManager::new();
    render(&mut manager, &build);
    press(&mut manager, &build, KeyCode::Tab);
    assert_eq!(manager.focused(), Some(FocusId(100)));
    press(&mut manager, &build, KeyCode::Tab);
    assert_eq!(
        manager.focused(),
        Some(FocusId(2)),
        "Tab enters on the selected tab"
    );
    press(&mut manager, &build, KeyCode::Tab);
    assert_eq!(
        manager.focused(),
        Some(panel_of(&manager, 2)),
        "next Tab enters the panel"
    );
    press(&mut manager, &build, KeyCode::BackTab);
    assert_eq!(manager.focused(), Some(FocusId(2)));
    press(&mut manager, &build, KeyCode::BackTab);
    assert_eq!(
        manager.focused(),
        Some(FocusId(100)),
        "the other tabs are not tab stops"
    );
}

#[test]
fn automatic_mode_selects_on_arrows_with_wrap_and_home_end() {
    let build = || three_tabs(RootProps::builder());
    let mut manager = FocusManager::new();
    let (text, _) = render(&mut manager, &build);
    assert_eq!(visible_panel(&text), "panel-a");
    manager.focus(FocusId(1));
    render(&mut manager, &build);

    for (code, focus, panel) in [
        (KeyCode::Right, 2, "panel-b"),
        (KeyCode::Right, 3, "panel-c"),
        (KeyCode::Right, 1, "panel-a"),
        (KeyCode::Left, 3, "panel-c"),
        (KeyCode::Home, 1, "panel-a"),
        (KeyCode::End, 3, "panel-c"),
    ] {
        let text = press(&mut manager, &build, code);
        assert_eq!(manager.focused(), Some(FocusId(focus)), "{code:?}");
        assert_eq!(visible_panel(&text), panel, "{code:?}");
    }
    // Up/Down belong to vertical tabs.
    press(&mut manager, &build, KeyCode::Down);
    assert_eq!(manager.focused(), Some(FocusId(3)));
}

#[test]
fn loop_focus_false_stops_at_the_ends() {
    let build = || three_tabs(RootProps::builder().loop_focus(false));
    let mut manager = FocusManager::new();
    render(&mut manager, &build);
    manager.focus(FocusId(3));
    render(&mut manager, &build);
    press(&mut manager, &build, KeyCode::Right);
    assert_eq!(manager.focused(), Some(FocusId(3)));
}

#[test]
fn vertical_tabs_use_up_and_down() {
    let build = || three_tabs(RootProps::builder().orientation(Orientation::Vertical));
    let mut manager = FocusManager::new();
    render(&mut manager, &build);
    manager.focus(FocusId(1));
    render(&mut manager, &build);
    press(&mut manager, &build, KeyCode::Right);
    assert_eq!(manager.focused(), Some(FocusId(1)), "Right is ignored");
    let text = press(&mut manager, &build, KeyCode::Down);
    assert_eq!(manager.focused(), Some(FocusId(2)));
    assert_eq!(visible_panel(&text), "panel-b");
    press(&mut manager, &build, KeyCode::Up);
    let text = press(&mut manager, &build, KeyCode::Up);
    assert_eq!(
        manager.focused(),
        Some(FocusId(3)),
        "Up wraps to the last tab"
    );
    assert_eq!(visible_panel(&text), "panel-c");
}

#[test]
fn manual_mode_moves_focus_without_selecting_until_enter_or_space() {
    let build = || three_tabs(RootProps::builder().activation_mode(TabsActivation::Manual));
    let mut manager = FocusManager::new();
    render(&mut manager, &build);
    manager.focus(FocusId(1));
    render(&mut manager, &build);

    let text = press(&mut manager, &build, KeyCode::Right);
    assert_eq!(manager.focused(), Some(FocusId(2)));
    assert_eq!(
        visible_panel(&text),
        "panel-a",
        "focus alone does not select"
    );
    let text = press(&mut manager, &build, KeyCode::Enter);
    assert_eq!(visible_panel(&text), "panel-b");
    press(&mut manager, &build, KeyCode::Right);
    let text = press(&mut manager, &build, KeyCode::Char(' '));
    assert_eq!(visible_panel(&text), "panel-c");
}

#[test]
fn clicking_a_tab_focuses_and_selects_it() {
    let build = || three_tabs(RootProps::builder().activation_mode(TabsActivation::Manual));
    let mut manager = FocusManager::new();
    let (_, hits) = render(&mut manager, &build);
    let target = hits
        .targets
        .iter()
        .rev()
        .find(|t| t.focus_id == Some(FocusId(3)))
        .expect("tab c is visible")
        .clone();
    manager.pointer_down(&target, target.rect.x, target.rect.y);
    let (text, _) = render(&mut manager, &build);
    assert_eq!(manager.focused(), Some(FocusId(3)));
    assert_eq!(visible_panel(&text), "panel-c");
}

#[test]
fn first_enabled_tab_is_selected_and_disabled_tabs_are_skipped() {
    let build = || {
        view! {
            <Tabs::Root>
                <Tabs::List>
                    <Tabs::Trigger value="a" id={FocusId(1)} disabled=true>"Alpha"</Tabs::Trigger>
                    <Tabs::Trigger value="b" id={FocusId(2)}>"Beta"</Tabs::Trigger>
                    <Tabs::Trigger value="c" id={FocusId(3)}>"Gamma"</Tabs::Trigger>
                </Tabs::List>
                <Tabs::Content value="a"><TextSpan>"panel-a"</TextSpan></Tabs::Content>
                <Tabs::Content value="b"><TextSpan>"panel-b"</TextSpan></Tabs::Content>
                <Tabs::Content value="c"><TextSpan>"panel-c"</TextSpan></Tabs::Content>
            </Tabs::Root>
        }
    };
    let mut manager = FocusManager::new();
    let (text, _) = render(&mut manager, &build);
    assert_eq!(visible_panel(&text), "panel-b");
    press(&mut manager, &build, KeyCode::Tab);
    assert_eq!(manager.focused(), Some(FocusId(2)));
    press(&mut manager, &build, KeyCode::Right);
    let text = press(&mut manager, &build, KeyCode::Right);
    assert_eq!(
        manager.focused(),
        Some(FocusId(2)),
        "wrap skips disabled tab a"
    );
    assert_eq!(visible_panel(&text), "panel-b");
}

#[test]
fn controlled_tabs_report_changes_once_and_follow_the_parent() {
    let calls = Rc::new(Cell::new(0));
    let (value, set_value) = create_signal(String::from("a"));
    let reject = Rc::new(Cell::new(true));
    let build = {
        let calls = calls.clone();
        let reject = reject.clone();
        move || {
            let calls = calls.clone();
            let reject = reject.clone();
            three_tabs(RootProps::builder().value(value.get()).on_value_change(
                move |next: String| {
                    calls.set(calls.get() + 1);
                    if !reject.get() {
                        set_value.set(next);
                    }
                },
            ))
        }
    };
    let mut manager = FocusManager::new();
    render(&mut manager, &build);
    manager.focus(FocusId(1));
    render(&mut manager, &build);

    // The parent ignores the change: selection stays and the callback is
    // not repeated by later renders.
    let text = press(&mut manager, &build, KeyCode::Right);
    assert_eq!(manager.focused(), Some(FocusId(2)));
    assert_eq!(visible_panel(&text), "panel-a");
    render(&mut manager, &build);
    render(&mut manager, &build);
    assert_eq!(calls.get(), 1);

    // The parent accepts the change.
    reject.set(false);
    let text = press(&mut manager, &build, KeyCode::Right);
    assert_eq!(calls.get(), 2);
    assert_eq!(value.get(), "c");
    assert_eq!(visible_panel(&text), "panel-c");
}

#[test]
fn tabs_and_panels_expose_linked_semantics() {
    let build = || three_tabs(RootProps::builder().default_value("b"));
    let mut manager = FocusManager::new();
    render(&mut manager, &build);
    let tree = manager.tree();
    let semantics = |id: usize| {
        tree.node(FocusId(id))
            .unwrap()
            .interaction
            .semantics
            .clone()
    };

    let list = tree
        .nodes()
        .iter()
        .find(|n| n.interaction.semantics.role == Role::TabList)
        .expect("a tablist is registered");
    assert!(
        list.interaction.group.is_some(),
        "tabs share one focus group"
    );
    assert!(tree.node(FocusId(1)).unwrap().ancestors.contains(&list.id));
    for (id, selected) in [(1, false), (2, true), (3, false)] {
        let tab = semantics(id);
        assert_eq!(tab.role, Role::Tab);
        assert_eq!(tab.selected, Some(selected));
    }
    let panels: Vec<_> = [1, 2, 3].map(|id| semantics(id).controls.unwrap()).into();
    assert!(
        panels[0] != panels[1] && panels[1] != panels[2],
        "each tab owns a panel"
    );
    let panel = tree.node(panels[1]).expect("the selected panel is mounted");
    assert_eq!(panel.interaction.semantics.role, Role::TabPanel);
    assert_eq!(panel.interaction.semantics.labelled_by, Some(FocusId(2)));
    assert!(
        tree.node(panels[0]).is_none(),
        "inactive panels are unmounted"
    );
    assert!(
        tree.node(panels[2]).is_none(),
        "inactive panels are unmounted"
    );
}

#[test]
fn panel_with_focusable_content_is_not_an_extra_tab_stop() {
    let build = || {
        view! {
            <Tabs::Root>
                <View>
                    <Tabs::List>
                        <Tabs::Trigger value="a" id={FocusId(1)}>"Alpha"</Tabs::Trigger>
                    </Tabs::List>
                    <Tabs::Content value="a">
                        <Button id={FocusId(50)}>"Inside"</Button>
                    </Tabs::Content>
                </View>
            </Tabs::Root>
        }
    };
    let mut manager = FocusManager::new();
    render(&mut manager, &build);
    press(&mut manager, &build, KeyCode::Tab);
    assert_eq!(manager.focused(), Some(FocusId(1)));
    press(&mut manager, &build, KeyCode::Tab);
    assert_eq!(
        manager.focused(),
        Some(FocusId(50)),
        "Tab goes straight to the content"
    );
}
