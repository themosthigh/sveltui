use std::cell::{Cell, RefCell};
use sveltui::prelude::*;

thread_local! {
    static SHOW_FIRST: Cell<bool> = const { Cell::new(true) };
    static LOG: RefCell<Vec<(&'static str, u32)>> = const { RefCell::new(Vec::new()) };
}

#[derive(Props)]
struct CounterProps {
    name: &'static str,
}

/// Reports how many frames it has been drawn for, from state of its own.
#[component]
fn Counter(props: CounterProps) -> impl IntoNode {
    let frames = state(0u32);
    frames.update(|n| *n += 1);
    LOG.with(|log| log.borrow_mut().push((props.name, frames.get())));
    view! { <TextSpan>{props.name}</TextSpan> }
}

#[component]
fn Switch() -> impl IntoNode {
    let first = SHOW_FIRST.with(Cell::get);
    view! {
        <View>
            {#if first}
                <Counter name="first" />
            {:else}
                <Counter name="second" />
            {/if}
        </View>
    }
}

fn frame() -> Vec<(&'static str, u32)> {
    LOG.with(|log| log.borrow_mut().clear());
    begin_frame();
    let _ = Switch(SwitchProps::builder().build()).into_node();
    end_frame();
    LOG.with(|log| log.borrow().clone())
}

#[test]
fn flipping_an_if_gives_the_new_branch_fresh_state() {
    SHOW_FIRST.with(|s| s.set(true));
    assert_eq!(frame(), [("first", 1)]);
    assert_eq!(frame(), [("first", 2)]);
    // Both branches put a Counter at the same position; the second must
    // not inherit the first one's count.
    SHOW_FIRST.with(|s| s.set(false));
    assert_eq!(frame(), [("second", 1)]);
    assert_eq!(frame(), [("second", 2)]);
    SHOW_FIRST.with(|s| s.set(true));
    assert_eq!(frame(), [("first", 1)]);
}
