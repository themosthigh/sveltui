use crate::*;
use sveltui::element::*;
use sveltui::layout::*;
use sveltui::prelude::*;
use sveltui::terminal::Buffer;

fn render(node: &VNode, width: u16, height: u16) -> (Buffer, HitMap) {
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
fn cursor_and_focus_ancestry_follow_nested_row_layout() {
    let parent = FocusId::new();
    let input = FocusId::new();
    let (value, set_value) = create_signal(String::from("abc"));
    let build = || {
        view! {
            <Pressable focus_id=parent tab_stop=true>
                <View flex_direction=FlexDirection::Row width=30 height=5 gap=1>
                    <View width=4 height=3 flex_shrink=0.0 />
                    <TextInput value={value.get()} on_value_change={move |v: String| set_value.set(v)} width={Dimension::Percent(1.0)} id=input />
                </View>
            </Pressable>
        }
    };
    let initial = build();
    let mut manager = FocusManager::new();
    manager.reconcile(InteractionTree::from_node(&initial));
    manager.focus(input);
    let node = manager.with_context(build);

    let (buffer, hits) = manager.with_context(|| render(&node, 40, 10));
    let target = rect(&hits, input);
    assert_eq!(target.x, 5);
    assert_eq!(target.width, 25);
    assert!(
        buffer
            .cursor()
            .is_some_and(|(x, y)| (6..30).contains(&x) && y == 1)
    );
    assert!(hits.ancestors.get(&input).unwrap().contains(&parent));
    assert_eq!(hits.focusable_ids(), vec![parent, input]);
}

#[test]
fn test_text_input_cursor_forward_typing() {
    let (val, set_val) = create_signal(String::new());
    let id = FocusId(999);

    // Initially cursor should be at 0
    assert_eq!(get_cursor(id, 0), 0);

    // Type 'a'
    let mut text: String = val.get();
    let cur = get_cursor(id, text.len());
    text.insert(cur, 'a');
    set_cursor(id, cur + 1);
    set_val.set(text);

    assert_eq!(val.get(), "a");
    assert_eq!(get_cursor(id, 1), 1);

    // Type 'b'
    let mut text: String = val.get();
    let cur = get_cursor(id, text.len());
    text.insert(cur, 'b');
    set_cursor(id, cur + 1);
    set_val.set(text);

    // Value must be "ab", NOT "ba"!
    assert_eq!(val.get(), "ab");
    assert_eq!(get_cursor(id, 2), 2);

    // Type 'c'
    let mut text: String = val.get();
    let cur = get_cursor(id, text.len());
    text.insert(cur, 'c');
    set_cursor(id, cur + 1);
    set_val.set(text);

    // Value must be "abc"!
    assert_eq!(val.get(), "abc");
    assert_eq!(get_cursor(id, 3), 3);
}

#[test]
fn an_outside_value_change_moves_the_cursor_to_the_end() {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    let id = FocusId::new();
    let text = state(String::new());
    let build = || view! { <TextInput id=id bind:value=text /> };
    let mut manager = FocusManager::new();
    let frame = |manager: &mut FocusManager| {
        let node = manager.with_context(build);
        let (_, hits) = render(&node, 40, 3);
        manager.reconcile(hits.interactions);
    };
    frame(&mut manager);
    manager.focus(id);

    text.set(String::from("Rust"));
    frame(&mut manager);
    manager.dispatch(KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE));
    frame(&mut manager);
    assert_eq!(text.get(), "Rusty");
}

mod bindings {
    use crate::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use std::{cell::RefCell, rc::Rc};
    use sveltui::element::{render_node_interactive, reset_focus_counter};
    use sveltui::layout::Rect;
    use sveltui::prelude::*;
    use sveltui::terminal::Buffer;

    const ID: FocusId = FocusId(100_201);

    fn frame(manager: &mut FocusManager, build: &impl Fn() -> VNode) -> String {
        for _ in 0..5 {
            reset_focus_counter();
            begin_frame();
            let node = manager.with_context(build);
            end_frame();
            let mut buffer = Buffer::new(30, 3);
            let mut hits = HitMap::new();
            manager.with_context(|| {
                render_node_interactive(&node, Rect::new(0, 0, 30, 3), &mut buffer, &mut hits)
            });
            if !manager.reconcile(hits.interactions) {
                return (0..30).map(|x| buffer.get(x, 1).unwrap().ch).collect();
            }
        }
        panic!("input focus did not settle");
    }

    fn type_char(manager: &mut FocusManager, build: &impl Fn() -> VNode, ch: char) -> String {
        manager.dispatch(KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE));
        frame(manager, build)
    }

    #[test]
    fn a_read_only_value_reports_edits_but_keeps_its_text() {
        let reported = Rc::new(RefCell::new(Vec::new()));
        let build = {
            let reported = reported.clone();
            move || {
                let reported = reported.clone();
                view! {
                    <TextInput id=ID value="abc"
                        on_value_change={move |next: String| reported.borrow_mut().push(next)} />
                }
            }
        };
        let mut manager = FocusManager::new();
        frame(&mut manager, &build);
        manager.focus(ID);
        let row = type_char(&mut manager, &build, 'd');
        assert!(row.contains("abc") && !row.contains("abcd"), "{row}");
        assert_eq!(*reported.borrow(), vec![String::from("abcd")]);
    }

    #[test]
    fn bind_value_writes_the_parents_state() {
        let name = state(String::from("Ada"));
        let build = move || view! { <TextInput id=ID bind:value={name} /> };
        let mut manager = FocusManager::new();
        frame(&mut manager, &build);
        manager.focus(ID);
        let row = type_char(&mut manager, &build, '!');
        assert!(row.contains("Ada!"), "{row}");
        assert_eq!(name.get(), "Ada!");
    }

    #[test]
    fn default_value_seeds_text_that_is_then_edited_locally() {
        let build = || view! { <TextInput id=ID default_value="hi" /> };
        let mut manager = FocusManager::new();
        assert!(frame(&mut manager, &build).contains("hi"));
        manager.focus(ID);
        assert!(type_char(&mut manager, &build, '!').contains("hi!"));
    }
}
