use super::{RootProps, RootPropsBuilder};
use crate::*;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::{cell::RefCell, rc::Rc};
use sveltui::element::*;
use sveltui::layout::Rect;
use sveltui::prelude::*;
use sveltui::terminal::Buffer;

const INPUT: FocusId = FocusId(100_001);
const AFTER: FocusId = FocusId(100_002);

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

/// Renders frames until focus settles, always rendering at least twice so
/// labels that depend on the previous frame catch up.
fn render(manager: &mut FocusManager, build: &impl Fn() -> VNode) -> (String, HitMap) {
    for frame in 0..10 {
        reset_focus_counter();
        begin_frame();
        let node = manager.with_context(build);
        end_frame();
        let mut hits = HitMap::new();
        let mut buffer = Buffer::new(50, 18);
        manager.with_context(|| {
            render_node_interactive(&node, Rect::new(0, 0, 50, 18), &mut buffer, &mut hits)
        });
        let changed = manager.reconcile(hits.interactions.clone());
        if frame > 0 && !changed {
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
    panic!("combobox focus did not settle");
}

fn press(manager: &mut FocusManager, build: &impl Fn() -> VNode, code: KeyCode) -> String {
    manager.dispatch(key(code));
    render(manager, build).0
}

fn type_text(manager: &mut FocusManager, build: &impl Fn() -> VNode, text: &str) -> String {
    let mut screen = String::new();
    for ch in text.chars() {
        screen = press(manager, build, KeyCode::Char(ch));
    }
    screen
}

/// A combobox over Svelte, Solid, Rust, Zig (disabled) and React, with a
/// button after it to tab to.
fn frameworks(builder: RootPropsBuilder) -> VNode {
    view! {
        <View>
            {Combobox::Root(
                builder
                    .id(INPUT)
                    .children(Snippet::new(|_| {
                        view! {
                            <View>
                                <View flex_direction=FlexDirection::Row>
                                    <Combobox::Input placeholder="Framework" width=24 />
                                    <Combobox::Trigger />
                                </View>
                                <Combobox::Content>
                                    <Combobox::Item value="svelte">"Svelte"</Combobox::Item>
                                    <Combobox::Item value="solid">"Solid"</Combobox::Item>
                                    <Combobox::Item value="rust">"Rust"</Combobox::Item>
                                    <Combobox::Item value="zig" disabled=true>"Zig"</Combobox::Item>
                                    <Combobox::Item value="react">"React"</Combobox::Item>
                                    <Combobox::Empty>"Nothing matches"</Combobox::Empty>
                                </Combobox::Content>
                            </View>
                        }
                    }))
                    .build(),
            )}
            <Button id=AFTER>"After"</Button>
        </View>
    }
}

fn input_semantics(manager: &FocusManager) -> Semantics {
    manager
        .tree()
        .node(INPUT)
        .unwrap()
        .interaction
        .semantics
        .clone()
}

fn is_open(manager: &FocusManager) -> bool {
    input_semantics(manager).expanded == Some(true)
}

fn highlighted(manager: &FocusManager) -> Option<String> {
    let id = input_semantics(manager).active_descendant?;
    Some(manager.tree().node(id)?.interaction.semantics.label.clone())
}

fn focused(manager: &mut FocusManager, build: &impl Fn() -> VNode) {
    render(manager, build);
    manager.focus(INPUT);
    render(manager, build);
}

#[test]
fn typing_opens_and_filters_case_insensitively() {
    let build = || frameworks(RootProps::builder());
    let mut manager = FocusManager::new();
    focused(&mut manager, &build);
    assert!(!is_open(&manager));

    let screen = type_text(&mut manager, &build, "s");
    assert!(is_open(&manager), "typing opens the list");
    assert!(screen.contains("Svelte") && screen.contains("Solid") && screen.contains("Rust"));
    assert!(!screen.contains("React"));

    let screen = type_text(&mut manager, &build, "VE");
    assert!(screen.contains("Svelte"));
    assert!(!screen.contains("Solid") && !screen.contains("Rust"));
    assert_eq!(manager.focused(), Some(INPUT), "focus stays in the input");
}

#[test]
fn arrows_highlight_and_enter_puts_the_label_in_the_input() {
    let reported = Rc::new(RefCell::new(Vec::new()));
    let build = {
        let reported = reported.clone();
        move || {
            let reported = reported.clone();
            frameworks(
                RootProps::builder()
                    .on_value_change(move |value: String| reported.borrow_mut().push(value)),
            )
        }
    };
    let mut manager = FocusManager::new();
    focused(&mut manager, &build);

    press(&mut manager, &build, KeyCode::Down);
    assert!(is_open(&manager));
    assert_eq!(highlighted(&manager).as_deref(), Some("Svelte"));
    for expected in ["Solid", "Rust", "React", "Svelte"] {
        press(&mut manager, &build, KeyCode::Down);
        assert_eq!(
            highlighted(&manager).as_deref(),
            Some(expected),
            "Zig is skipped"
        );
    }
    press(&mut manager, &build, KeyCode::Up);
    let screen = press(&mut manager, &build, KeyCode::Enter);
    assert!(!is_open(&manager));
    assert_eq!(*reported.borrow(), vec![String::from("react")]);
    assert!(screen.contains("React"), "the label is in the input");
    assert_eq!(manager.focused(), Some(INPUT));

    // With the selected label in the input, reopening lists everything.
    let screen = press(&mut manager, &build, KeyCode::Down);
    assert!(screen.contains("Svelte") && screen.contains("Rust"));
}

#[test]
fn up_opens_with_the_last_option_highlighted() {
    let build = || frameworks(RootProps::builder());
    let mut manager = FocusManager::new();
    focused(&mut manager, &build);
    press(&mut manager, &build, KeyCode::Up);
    assert_eq!(highlighted(&manager).as_deref(), Some("React"));
}

#[test]
fn escape_closes_and_puts_back_the_earlier_text() {
    let build = || frameworks(RootProps::builder().default_value("rust"));
    let mut manager = FocusManager::new();
    focused(&mut manager, &build);
    let screen = render(&mut manager, &build).0;
    assert!(
        screen.contains("Rust"),
        "the selected label starts in the input"
    );

    press(&mut manager, &build, KeyCode::Backspace);
    let screen = type_text(&mut manager, &build, "eact");
    assert!(screen.contains("Ruseact"));
    assert!(is_open(&manager));
    let screen = press(&mut manager, &build, KeyCode::Esc);
    assert!(!is_open(&manager));
    assert!(screen.contains("Rust") && !screen.contains("Ruseact"));
}

#[test]
fn tab_closes_the_list_and_moves_on() {
    let build = || frameworks(RootProps::builder());
    let mut manager = FocusManager::new();
    focused(&mut manager, &build);
    press(&mut manager, &build, KeyCode::Down);
    press(&mut manager, &build, KeyCode::Tab);
    assert!(!is_open(&manager));
    assert_eq!(manager.focused(), Some(AFTER));
}

#[test]
fn empty_shows_when_nothing_matches() {
    let build = || frameworks(RootProps::builder());
    let mut manager = FocusManager::new();
    focused(&mut manager, &build);
    let screen = type_text(&mut manager, &build, "s");
    assert!(!screen.contains("Nothing matches"));
    let screen = type_text(&mut manager, &build, "xyz");
    assert!(screen.contains("Nothing matches"));
    press(&mut manager, &build, KeyCode::Down);
    assert_eq!(highlighted(&manager), None, "there's nothing to highlight");
}

#[test]
fn clicks_open_choose_and_dismiss() {
    let build = || frameworks(RootProps::builder());
    let mut manager = FocusManager::new();
    let (_, hits) = render(&mut manager, &build);
    let target = |hits: &HitMap, id: FocusId| {
        hits.targets
            .iter()
            .rev()
            .find(|t| t.focus_id == Some(id))
            .cloned()
            .expect("target is visible")
    };

    let input = target(&hits, INPUT);
    manager.pointer_down(&input, input.rect.x + 2, input.rect.y + 1);
    let (_, hits) = render(&mut manager, &build);
    assert!(is_open(&manager), "clicking the input opens the list");
    assert_eq!(manager.focused(), Some(INPUT));

    let solid = manager
        .tree()
        .nodes()
        .iter()
        .find(|n| n.interaction.semantics.label == "Solid")
        .unwrap()
        .id;
    let row = target(&hits, solid);
    manager.pointer_down(&row, row.rect.x + 1, row.rect.y);
    let (screen, _) = render(&mut manager, &build);
    assert!(!is_open(&manager));
    assert!(screen.contains("Solid"));

    press(&mut manager, &build, KeyCode::Down);
    let (_, hits) = render(&mut manager, &build);
    manager.pointer_down_at(&hits, 48, 17);
    render(&mut manager, &build);
    assert!(!is_open(&manager), "clicking outside closes the list");
}

#[test]
fn input_exposes_combobox_semantics() {
    let build = || frameworks(RootProps::builder());
    let mut manager = FocusManager::new();
    focused(&mut manager, &build);
    press(&mut manager, &build, KeyCode::Down);
    let input = input_semantics(&manager);
    assert_eq!(input.role, Role::Combobox);
    assert_eq!(input.has_popup, Some(Role::Listbox));
    let list = input.controls.expect("the input points at its list");
    assert_eq!(
        manager
            .tree()
            .node(list)
            .unwrap()
            .interaction
            .semantics
            .role,
        Role::Listbox
    );
    let active = input.active_descendant.expect("an option is highlighted");
    assert_eq!(
        manager
            .tree()
            .node(active)
            .unwrap()
            .interaction
            .semantics
            .role,
        Role::Option
    );
}

#[test]
fn options_list_shortcut_filters_and_chooses() {
    let options = vec![
        ListboxOption::new("ams", "Amsterdam"),
        ListboxOption::new("ber", "Berlin"),
        ListboxOption::new("bru", "Brussels"),
    ];
    let build = move || {
        view! { <Combobox id=INPUT options={options.clone()} placeholder="City" empty="No city" /> }
    };
    let mut manager = FocusManager::new();
    focused(&mut manager, &build);
    let screen = type_text(&mut manager, &build, "br");
    assert!(screen.contains("Brussels") && !screen.contains("Berlin"));
    press(&mut manager, &build, KeyCode::Down);
    let screen = press(&mut manager, &build, KeyCode::Enter);
    assert!(screen.contains("Brussels"));
    let screen = type_text(&mut manager, &build, "q");
    assert!(screen.contains("No city"));
}
