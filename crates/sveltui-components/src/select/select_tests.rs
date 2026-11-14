use super::RootProps;
use crate::*;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::{cell::RefCell, rc::Rc};
use sveltui::element::*;
use sveltui::layout::Rect;
use sveltui::prelude::*;
use sveltui::terminal::Buffer;

const TRIGGER: FocusId = FocusId(100_001);

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
        let mut buffer = Buffer::new(50, 16);
        manager.with_context(|| {
            render_node_interactive(&node, Rect::new(0, 0, 50, 16), &mut buffer, &mut hits)
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
    panic!("select focus did not settle");
}

fn press(manager: &mut FocusManager, build: &impl Fn() -> VNode, code: KeyCode) -> String {
    manager.dispatch(key(code));
    render(manager, build).0
}

fn fruits(root: RootProps) -> VNode {
    Select::Root(root)
}

/// A select over Apple, Banana, Blueberry, Cherry (disabled) and Date.
fn fruit_select(builder: super::RootPropsBuilder) -> VNode {
    fruits(
        builder
            .id(TRIGGER)
            .children(Snippet::new(|_| {
                view! {
                    <View>
                        <Select::Trigger width=20>
                            <Select::Value placeholder="Pick a fruit" />
                        </Select::Trigger>
                        <Select::Content>
                            <Select::Item value="apple">"Apple"</Select::Item>
                            <Select::Item value="banana">"Banana"</Select::Item>
                            <Select::Item value="blueberry">"Blueberry"</Select::Item>
                            <Select::Item value="cherry" disabled=true>"Cherry"</Select::Item>
                            <Select::Item value="date">"Date"</Select::Item>
                        </Select::Content>
                    </View>
                }
            }))
            .build(),
    )
}

fn trigger_semantics(manager: &FocusManager) -> Semantics {
    manager
        .tree()
        .node(TRIGGER)
        .unwrap()
        .interaction
        .semantics
        .clone()
}

/// The label of the highlighted option, read through `active_descendant`.
fn highlighted(manager: &FocusManager) -> Option<String> {
    let id = trigger_semantics(manager).active_descendant?;
    Some(manager.tree().node(id)?.interaction.semantics.label.clone())
}

fn open_with_focus(manager: &mut FocusManager, build: &impl Fn() -> VNode) {
    render(manager, build);
    manager.focus(TRIGGER);
    render(manager, build);
}

#[test]
fn shows_placeholder_then_the_chosen_label_and_keeps_focus_on_the_trigger() {
    let build = || fruit_select(RootProps::builder());
    let mut manager = FocusManager::new();
    let (text, _) = render(&mut manager, &build);
    assert!(text.contains("Pick a fruit"));
    assert!(!text.contains("Banana"), "the list starts closed");

    manager.focus(TRIGGER);
    let text = press(&mut manager, &build, KeyCode::Down);
    assert!(text.contains("Banana"), "Down opens the list");
    assert_eq!(trigger_semantics(&manager).expanded, Some(true));
    assert_eq!(highlighted(&manager).as_deref(), Some("Apple"));

    press(&mut manager, &build, KeyCode::Down);
    let text = press(&mut manager, &build, KeyCode::Enter);
    assert_eq!(manager.focused(), Some(TRIGGER));
    assert_eq!(trigger_semantics(&manager).expanded, Some(false));
    assert!(text.contains("Banana") && !text.contains("Blueberry"));
}

#[test]
fn arrows_wrap_skip_disabled_and_home_end_jump() {
    let build = || fruit_select(RootProps::builder());
    let mut manager = FocusManager::new();
    open_with_focus(&mut manager, &build);
    press(&mut manager, &build, KeyCode::Enter);
    assert_eq!(highlighted(&manager).as_deref(), Some("Apple"));

    for (code, expected) in [
        (KeyCode::Up, "Date"),
        (KeyCode::Up, "Blueberry"),
        (KeyCode::Down, "Date"),
        (KeyCode::Down, "Apple"),
        (KeyCode::End, "Date"),
        (KeyCode::Home, "Apple"),
    ] {
        press(&mut manager, &build, code);
        assert_eq!(highlighted(&manager).as_deref(), Some(expected), "{code:?}");
        assert_eq!(manager.focused(), Some(TRIGGER));
    }
}

#[test]
fn opening_highlights_the_selected_option() {
    let build = || fruit_select(RootProps::builder().default_value("blueberry"));
    let mut manager = FocusManager::new();
    open_with_focus(&mut manager, &build);
    press(&mut manager, &build, KeyCode::Char(' '));
    assert_eq!(highlighted(&manager).as_deref(), Some("Blueberry"));
}

#[test]
fn up_opens_with_the_last_option_highlighted() {
    let build = || fruit_select(RootProps::builder());
    let mut manager = FocusManager::new();
    open_with_focus(&mut manager, &build);
    press(&mut manager, &build, KeyCode::Up);
    assert_eq!(highlighted(&manager).as_deref(), Some("Date"));
}

#[test]
fn escape_and_tab_close_without_changing_the_value() {
    let build = || fruit_select(RootProps::builder().default_value("apple"));
    let mut manager = FocusManager::new();
    open_with_focus(&mut manager, &build);
    press(&mut manager, &build, KeyCode::Down);
    press(&mut manager, &build, KeyCode::Down);
    let text = press(&mut manager, &build, KeyCode::Esc);
    assert_eq!(trigger_semantics(&manager).expanded, Some(false));
    assert_eq!(manager.focused(), Some(TRIGGER));
    assert!(text.contains("Apple") && !text.contains("Banana"));

    press(&mut manager, &build, KeyCode::Down);
    press(&mut manager, &build, KeyCode::Tab);
    assert_eq!(trigger_semantics(&manager).expanded, Some(false));
}

#[test]
fn typeahead_matches_labels_and_cycles_on_a_repeated_letter() {
    let build = || fruit_select(RootProps::builder());
    let mut manager = FocusManager::new();
    open_with_focus(&mut manager, &build);
    press(&mut manager, &build, KeyCode::Char('b'));
    assert_eq!(
        trigger_semantics(&manager).expanded,
        Some(true),
        "typing opens"
    );
    assert_eq!(highlighted(&manager).as_deref(), Some("Banana"));
    press(&mut manager, &build, KeyCode::Char('b'));
    assert_eq!(highlighted(&manager).as_deref(), Some("Blueberry"));
    press(&mut manager, &build, KeyCode::Char('b'));
    assert_eq!(highlighted(&manager).as_deref(), Some("Banana"));
}

#[test]
fn pointer_toggles_hovers_and_chooses() {
    let build = || fruit_select(RootProps::builder());
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

    let trigger = target(&hits, TRIGGER);
    manager.pointer_down(&trigger, trigger.rect.x, trigger.rect.y);
    let (_, hits) = render(&mut manager, &build);
    assert_eq!(trigger_semantics(&manager).expanded, Some(true));

    let date = manager
        .tree()
        .nodes()
        .iter()
        .find(|n| n.interaction.semantics.label == "Date")
        .unwrap()
        .id;
    let row = target(&hits, date);
    manager.pointer_move(&hits, row.rect.x + 1, row.rect.y);
    render(&mut manager, &build);
    assert_eq!(highlighted(&manager).as_deref(), Some("Date"));

    manager.pointer_down(&row, row.rect.x + 1, row.rect.y);
    let (text, _) = render(&mut manager, &build);
    assert_eq!(trigger_semantics(&manager).expanded, Some(false));
    assert!(text.contains("Date"));
}

#[test]
fn clicking_outside_closes_the_list() {
    let build = || fruit_select(RootProps::builder().default_open(true));
    let mut manager = FocusManager::new();
    let (_, hits) = render(&mut manager, &build);
    assert_eq!(trigger_semantics(&manager).expanded, Some(true));
    manager.pointer_down_at(&hits, 45, 15);
    render(&mut manager, &build);
    assert_eq!(trigger_semantics(&manager).expanded, Some(false));
}

#[test]
fn options_expose_listbox_semantics() {
    let build = || fruit_select(RootProps::builder().default_value("banana"));
    let mut manager = FocusManager::new();
    open_with_focus(&mut manager, &build);
    press(&mut manager, &build, KeyCode::Enter);
    let tree = manager.tree();
    let trigger = trigger_semantics(&manager);
    assert_eq!(trigger.role, Role::Combobox);
    assert_eq!(trigger.has_popup, Some(Role::Listbox));
    let list = trigger.controls.expect("the trigger points at its list");
    assert_eq!(
        tree.node(list).unwrap().interaction.semantics.role,
        Role::Listbox
    );
    let option = |label: &str| {
        tree.nodes()
            .iter()
            .find(|n| n.interaction.semantics.label == label)
            .unwrap()
            .clone()
    };
    assert_eq!(option("Banana").interaction.semantics.role, Role::Option);
    assert_eq!(option("Banana").interaction.semantics.selected, Some(true));
    assert_eq!(option("Apple").interaction.semantics.selected, Some(false));
    assert!(option("Cherry").interaction.disabled);
}

#[test]
fn controlled_value_reports_changes_and_follows_the_parent() {
    let reported = Rc::new(RefCell::new(Vec::new()));
    let build = {
        let reported = reported.clone();
        move || {
            let reported = reported.clone();
            fruit_select(
                RootProps::builder()
                    .value("apple")
                    .on_value_change(move |next: String| reported.borrow_mut().push(next)),
            )
        }
    };
    let mut manager = FocusManager::new();
    open_with_focus(&mut manager, &build);
    press(&mut manager, &build, KeyCode::Enter);
    press(&mut manager, &build, KeyCode::Down);
    let text = press(&mut manager, &build, KeyCode::Enter);
    assert_eq!(*reported.borrow(), vec![String::from("banana")]);
    assert!(text.contains("Apple"), "the parent kept its value");
}

#[test]
fn options_list_shortcut_renders_and_selects() {
    let options = vec![
        ListboxOption::new("sm", "Small"),
        ListboxOption::new("md", "Medium"),
        ListboxOption::new("lg", "Large").disabled(true),
    ];
    let build = move || {
        view! { <Select id=TRIGGER options={options.clone()} placeholder="Size" width=16 /> }
    };
    let mut manager = FocusManager::new();
    let (text, _) = render(&mut manager, &build);
    assert!(text.contains("Size"));
    manager.focus(TRIGGER);
    render(&mut manager, &build);
    press(&mut manager, &build, KeyCode::Enter);
    press(&mut manager, &build, KeyCode::Down);
    let text = press(&mut manager, &build, KeyCode::Enter);
    assert!(text.contains("Medium"));
}

#[test]
fn bind_value_and_bind_open_follow_the_list() {
    let fruit = state(String::new());
    let open = state(false);
    let build = move || fruit_select(RootProps::builder().bind_value(fruit).bind_open(open));
    let mut manager = FocusManager::new();
    open_with_focus(&mut manager, &build);
    press(&mut manager, &build, KeyCode::Down);
    assert!(open.get(), "opening writes the bound open state");
    press(&mut manager, &build, KeyCode::Down);
    let text = press(&mut manager, &build, KeyCode::Enter);
    assert_eq!(fruit.get(), "banana");
    assert!(!open.get());
    assert!(text.contains("Banana"));
}
