use crate::*;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use sveltui::element::*;
use sveltui::layout::*;
use sveltui::prelude::*;
use sveltui::terminal::Buffer;

fn render(manager: &mut FocusManager, build: impl Fn() -> VNode) -> (Buffer, HitMap) {
    for _ in 0..5 {
        reset_focus_counter();
        begin_frame();
        let node = manager.with_context(&build);
        end_frame();
        let mut buffer = Buffer::new(60, 20);
        let mut hits = HitMap::new();
        manager.with_context(|| {
            render_node_interactive(&node, Rect::new(0, 0, 60, 20), &mut buffer, &mut hits);
        });
        if !manager.reconcile(hits.interactions.clone()) {
            return (buffer, hits);
        }
    }
    panic!("input focus did not stabilize");
}

fn row(buffer: &Buffer, y: u16) -> String {
    (0..buffer.width)
        .map(|x| buffer.get(x, y).unwrap().ch)
        .collect()
}

fn press(manager: &mut FocusManager, code: KeyCode) {
    assert!(
        manager
            .dispatch(KeyEvent::new(code, KeyModifiers::NONE))
            .handled
    );
}

#[test]
fn test_text_area_basic_instantiation() {
    let (val, set_val) = create_signal(String::from("Hello\nWorld"));
    let area = TextArea(
        TextAreaProps::builder()
            .value(val.get())
            .on_value_change(move |v: String| set_val.set(v))
            .width(50)
            .height(10)
            .title("Notes")
            .show_line_numbers(true)
            .build(),
    );

    let node = area.into_node();
    match node {
        VNode::View(v) => {
            assert_eq!(v.pressables.len(), 1);
            assert_eq!(
                v.visual.as_ref().and_then(|vis| vis.title.clone()),
                Some("Notes".to_string())
            );
            assert_eq!(v.children.len(), 2);
        }
        _ => panic!("Expected a pressable View node"),
    }
}

#[test]
fn textarea_state_binding_renders_each_keystroke_and_newline() {
    let value = state(String::new());
    let id = FocusId::new();
    let build = || view! { <TextArea id=id bind:value=value placeholder="Notes..." /> };
    let mut manager = FocusManager::new();
    let (buffer, _) = render(&mut manager, build);
    assert!(row(&buffer, 1).contains("Notes..."));
    manager.focus(id);
    for (code, expected, y, visible) in [
        (KeyCode::Char('a'), "a", 1, "a"),
        (KeyCode::Char('é'), "aé", 1, "aé"),
        (KeyCode::Enter, "aé\n", 1, "aé"),
        (KeyCode::Char('b'), "aé\nb", 2, "b"),
        (KeyCode::Backspace, "aé\n", 1, "aé"),
        (KeyCode::Backspace, "aé", 1, "aé"),
    ] {
        press(&mut manager, code);
        let (buffer, _) = render(&mut manager, build);
        assert_eq!(value.get(), expected);
        assert!(row(&buffer, y).contains(visible), "{buffer:?}");
        assert!(!row(&buffer, 1).contains("Notes..."));
    }
}

#[test]
fn textarea_explicit_signal_setter_updates_and_accepts_external_reset() {
    let (value, set_value) = create_signal(String::new());
    let id = FocusId::new();
    let build = || view! { <TextArea id=id value={value.get()} on_value_change={move |v: String| set_value.set(v)} /> };
    let mut manager = FocusManager::new();
    render(&mut manager, build);
    manager.focus(id);
    press(&mut manager, KeyCode::Char('x'));
    let (buffer, _) = render(&mut manager, build);
    assert_eq!(value.get(), "x");
    assert!(row(&buffer, 1).contains('x'));

    set_value.set("Replacement".into());
    let (buffer, _) = render(&mut manager, build);
    assert!(row(&buffer, 1).contains("Replacement"));
    set_value.set(String::new());
    let (buffer, _) = render(&mut manager, build);
    assert!(row(&buffer, 1).contains("Type text here..."));
}

#[test]
fn textarea_buffer_binding_preserves_content_and_cursor() {
    let (buffer, set_buffer) = create_signal(TextBuffer::new(""));
    let id = FocusId::new();
    let build = || view! { <TextArea id=id buffer={buffer.get()} on_buffer_change={move |v: TextBuffer| set_buffer.set(v)} /> };
    let mut manager = FocusManager::new();
    render(&mut manager, build);
    manager.focus(id);
    for code in [
        KeyCode::Char('a'),
        KeyCode::Enter,
        KeyCode::Char('b'),
        KeyCode::Left,
    ] {
        press(&mut manager, code);
        render(&mut manager, build);
    }
    assert_eq!(buffer.get().content(), "a\nb");
    assert_eq!(buffer.get().cursor_pos(), (1, 0));
    let (screen, _) = render(&mut manager, build);
    assert_eq!(screen.cursor(), Some((2, 2)));
}

#[test]
fn textarea_unbound_input_survives_renders_and_scrolls_with_line_numbers() {
    let id = FocusId::new();
    let build = || view! { <TextArea id=id width=16 height=4 show_line_numbers=true /> };
    let mut manager = FocusManager::new();
    render(&mut manager, build);
    manager.focus(id);
    for c in "one\ntwo\nthree".chars() {
        press(
            &mut manager,
            if c == '\n' {
                KeyCode::Enter
            } else {
                KeyCode::Char(c)
            },
        );
        render(&mut manager, build);
    }
    let (buffer, _) = render(&mut manager, build);
    assert!(row(&buffer, 1).contains("2  two"));
    assert!(row(&buffer, 2).contains("3  three"));
    assert_eq!(buffer.cursor(), Some((11, 2)));
    assert_eq!(with_buffer(id, |buf| buf.content()), "one\ntwo\nthree");
}

#[test]
fn textarea_click_focus_and_tab_keep_input_in_the_focused_control() {
    let area = state(String::new());
    let input = state(String::new());
    let area_id = FocusId::new();
    let input_id = FocusId::new();
    let build = || {
        view! {
            <View>
                <TextArea id=area_id bind:value=area />
                <TextInput id=input_id bind:value=input />
            </View>
        }
    };
    let mut manager = FocusManager::new();
    let (_, hits) = render(&mut manager, build);
    let target = hits
        .targets
        .iter()
        .find(|t| t.focus_id == Some(area_id))
        .unwrap();
    manager.pointer_down_at(&hits, target.rect.x + 2, target.rect.y + 1);
    assert_eq!(manager.focused(), Some(area_id));
    press(&mut manager, KeyCode::Char('a'));
    render(&mut manager, build);
    press(&mut manager, KeyCode::Tab);
    assert_eq!(manager.focused(), Some(input_id));
    press(&mut manager, KeyCode::Char('b'));
    render(&mut manager, build);
    assert_eq!(area.get(), "a");
    assert_eq!(input.get(), "b");
}

#[test]
fn textarea_key_repeat_inserts_and_key_release_is_ignored() {
    let value = state(String::new());
    let id = FocusId::new();
    let build = || view! { <TextArea id=id bind:value=value /> };
    let mut manager = FocusManager::new();
    render(&mut manager, build);
    manager.focus(id);
    for kind in [
        KeyEventKind::Press,
        KeyEventKind::Repeat,
        KeyEventKind::Release,
    ] {
        let outcome = manager.dispatch(KeyEvent::new_with_kind(
            KeyCode::Char('x'),
            KeyModifiers::NONE,
            kind,
        ));
        assert_eq!(outcome.handled, kind != KeyEventKind::Release);
        render(&mut manager, build);
    }
    assert_eq!(value.get(), "xx");
}

#[test]
fn text_fills_an_area_whose_width_is_a_percentage() {
    let text = "abcdefghijklmnopqrstuvwxyz0123456789ABCDEF";
    let id = FocusId(4244);
    let mut manager = FocusManager::new();
    render(&mut manager, || {
        view! {
            <View width=60>
                <TextArea value={String::from(text)} width={Dimension::Percent(1.0)} id=id />
            </View>
        }
    });
    let (buffer, _) = render(&mut manager, || {
        view! {
            <View width=60>
                <TextArea value={String::from(text)} width={Dimension::Percent(1.0)} id=id />
            </View>
        }
    });
    let shown: String = (0..20)
        .map(|y| row(&buffer, y))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        shown.contains(text),
        "text area cut the text short:\n{shown}"
    );
}
