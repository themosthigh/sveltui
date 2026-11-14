use super::*;
use crate::element::node::{IntoNode, TextElement, VNode};
use crate::reactive::{create_signal, on_destroy, state};
use std::cell::Cell;
use std::rc::Rc;

fn text(content: impl Into<String>) -> VNode {
    VNode::Text(TextElement::new(content))
}

fn content(node: &VNode) -> String {
    match node {
        VNode::Text(t) => t.content.clone(),
        VNode::Fragment(items) => items.iter().map(content).collect::<Vec<_>>().join("|"),
        VNode::View(v) => v.children.iter().map(content).collect::<Vec<_>>().join("|"),
        _ => String::new(),
    }
}

#[test]
fn a_write_rebuilds_only_the_slots_that_read_it() {
    let (a, set_a) = create_signal(1);
    let (b, set_b) = create_signal(10);
    let reads_a = DynamicSlot::new(move || text(format!("a{}", a.get())));
    let reads_b = DynamicSlot::new(move || text(format!("b{}", b.get())));
    let reads_neither = DynamicSlot::new(|| text("fixed"));

    let all = |slots: &[&DynamicSlot]| {
        slots
            .iter()
            .map(|s| content(&s.refresh()))
            .collect::<Vec<_>>()
    };
    let slots = [&reads_a, &reads_b, &reads_neither];
    assert_eq!(all(&slots), ["a1", "b10", "fixed"]);

    // Nothing was written, so nothing rebuilds.
    all(&slots);
    assert_eq!(
        [reads_a.runs(), reads_b.runs(), reads_neither.runs()],
        [1, 1, 1]
    );

    set_a.set(2);
    assert_eq!(all(&slots), ["a2", "b10", "fixed"]);
    assert_eq!(
        [reads_a.runs(), reads_b.runs(), reads_neither.runs()],
        [2, 1, 1]
    );

    set_b.set(11);
    assert_eq!(all(&slots), ["a2", "b11", "fixed"]);
    assert_eq!(
        [reads_a.runs(), reads_b.runs(), reads_neither.runs()],
        [2, 2, 1]
    );
    assert_eq!(reads_a.reads(), vec![a.id()]);
}

#[test]
fn replace_rebuilds_even_when_no_signal_changed() {
    let slot = DynamicSlot::new(|| text("old"));
    assert_eq!(content(&slot.refresh()), "old");
    slot.replace(|| text("new"));
    assert_eq!(content(&slot.refresh()), "new");
    assert_eq!(slot.runs(), 2);
}

#[test]
fn state_made_inside_a_slot_survives_its_rebuilds() {
    let (tick, set_tick) = create_signal(0);
    let slot = DynamicSlot::new(move || {
        let _ = tick.get();
        let count = state(0);
        count.update(|n| *n += 1);
        text(count.get().to_string())
    });
    assert_eq!(content(&slot.refresh()), "1");
    set_tick.set(1);
    assert_eq!(content(&slot.refresh()), "2");
}

#[test]
fn dropping_a_slot_runs_the_cleanups_of_what_it_built() {
    let cleaned = Rc::new(Cell::new(0));
    let seen = cleaned.clone();
    let slot = DynamicSlot::new(move || {
        let seen = seen.clone();
        on_destroy(move || seen.set(seen.get() + 1));
        VNode::Empty
    });
    slot.refresh();
    assert_eq!(cleaned.get(), 0);
    drop(slot);
    assert_eq!(cleaned.get(), 1);
}

#[test]
fn resolve_replaces_slots_at_any_depth() {
    let (n, set_n) = create_signal(1);
    let slot = DynamicSlot::new(move || {
        VNode::Fragment(vec![
            text(format!("n{}", n.get())),
            DynamicSlot::new(|| text("inner")).into_node(),
        ])
    });
    let tree = VNode::Fragment(vec![text("x"), slot.clone().into_node()]);
    assert!(tree.has_dynamic());
    let resolved = tree.clone().resolve();
    assert!(!resolved.has_dynamic());
    assert_eq!(content(&resolved), "x|n1|inner");
    set_n.set(2);
    assert_eq!(content(&tree.resolve()), "x|n2|inner");
}
