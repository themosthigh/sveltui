use super::*;
use crate::element::node::{TextElement, VNode};
use crate::reactive::{create_signal, on_destroy, state};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

fn texts(node: &VNode) -> Vec<String> {
    match node.clone().resolve() {
        VNode::Fragment(items) => items
            .iter()
            .map(|n| match n {
                VNode::Text(t) => t.content.clone(),
                _ => String::new(),
            })
            .collect(),
        _ => Vec::new(),
    }
}

#[test]
fn removing_the_first_row_leaves_the_others_untouched() {
    let builds = Rc::new(RefCell::new(Vec::<u32>::new()));
    let cleaned = Rc::new(Cell::new(Vec::<u32>::new()));
    let (log, gone) = (builds.clone(), cleaned.clone());
    let list = KeyedList::new(move |key: &u32, item: &String| {
        let uses = state(0u32);
        uses.update(|n| *n += 1);
        log.borrow_mut().push(*key);
        let (gone, key) = (gone.clone(), *key);
        on_destroy(move || {
            let mut ids = gone.take();
            ids.push(key);
            gone.set(ids);
        });
        VNode::Text(TextElement::new(format!("{item}#{}", uses.get())))
    });
    let items = |ids: &[u32]| {
        ids.iter()
            .map(|&id| (id, format!("r{id}")))
            .collect::<Vec<_>>()
    };

    let tree = list.update(items(&[1, 2, 3]));
    assert_eq!(texts(&tree), ["r1#1", "r2#1", "r3#1"]);
    assert_eq!(builds.borrow().len(), 3);

    // The old tree holds the rows too; a row lives as long as anything does.
    drop(tree);
    let tree = list.update(items(&[2, 3]));
    assert_eq!(texts(&tree), ["r2#1", "r3#1"]);
    // Rows 2 and 3 were not built again, and row 1 was cleaned up.
    assert_eq!(builds.borrow().len(), 3);
    assert_eq!(cleaned.take(), vec![1]);
    assert_eq!(list.len(), 2);
}

#[test]
fn a_changed_item_rebuilds_its_row_alone_and_keeps_its_state() {
    let builds = Rc::new(RefCell::new(Vec::<u32>::new()));
    let log = builds.clone();
    let list = KeyedList::new(move |key: &u32, item: &String| {
        let uses = state(0u32);
        uses.update(|n| *n += 1);
        log.borrow_mut().push(*key);
        VNode::Text(TextElement::new(format!("{item}#{}", uses.get())))
    });
    let pair = |b: &str| vec![(1, "a".to_string()), (2, b.to_string())];
    texts(&list.update(pair("b")));
    let tree = list.update(pair("B"));
    assert_eq!(texts(&tree), ["a#1", "B#2"]);
    assert_eq!(*builds.borrow(), vec![1, 2, 2]);
}

#[test]
fn a_write_rebuilds_only_the_rows_that_read_it() {
    let (a, set_a) = create_signal(0);
    let (b, _set_b) = create_signal(0);
    let builds = Rc::new(RefCell::new(Vec::<u32>::new()));
    let log = builds.clone();
    let list = KeyedList::new(move |key: &u32, _item: &()| {
        log.borrow_mut().push(*key);
        let value = if *key == 1 { a.get() } else { b.get() };
        VNode::Text(TextElement::new(value.to_string()))
    });
    let items = || vec![(1u32, ()), (2, ())];
    texts(&list.update(items()));
    set_a.set(5);
    assert_eq!(texts(&list.update(items())), ["5", "0"]);
    assert_eq!(*builds.borrow(), vec![1, 2, 1]);
}

#[test]
fn moving_an_item_moves_its_row() {
    let list = KeyedList::new(|key: &u32, _: &()| VNode::Text(TextElement::new(key.to_string())));
    texts(&list.update([(1u32, ()), (2, ()), (3, ())]));
    assert_eq!(
        texts(&list.update([(3u32, ()), (1, ()), (2, ())])),
        ["3", "1", "2"]
    );
}
