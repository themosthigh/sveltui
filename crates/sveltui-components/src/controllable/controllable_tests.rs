use super::*;
use std::{cell::RefCell, rc::Rc};
use sveltui::prelude::*;

type Log = Rc<RefCell<Vec<i32>>>;

fn recorder() -> (Log, Rc<dyn Fn(i32)>) {
    let log = Rc::new(RefCell::new(Vec::new()));
    let sink = log.clone();
    (log, Rc::new(move |v| sink.borrow_mut().push(v)))
}

#[test]
fn uncontrolled_keeps_local_state_and_reports() {
    let (log, on_change) = recorder();
    let model = use_controllable(None, None, || 1, Some(on_change));
    assert_eq!(model.get(), 1);
    model.set(2);
    assert_eq!(model.get(), 2);
    assert_eq!(*log.borrow(), vec![2]);
}

#[test]
fn controlled_only_reports() {
    let (log, on_change) = recorder();
    let model = use_controllable(None, Some(5), || 0, Some(on_change));
    model.set(6);
    assert_eq!(model.get(), 5, "a read-only value never changes");
    assert_eq!(*log.borrow(), vec![6]);
}

#[test]
fn bound_state_is_written_and_reported() {
    let (log, on_change) = recorder();
    let bound = state(3);
    let model = use_controllable(
        Some(Binding::State(bound)),
        Some(100),
        || 0,
        Some(on_change),
    );
    assert_eq!(model.get(), 3, "the bound state wins over a plain value");
    model.set(4);
    assert_eq!(bound.get(), 4);
    assert_eq!(*log.borrow(), vec![4]);
}

#[test]
fn setting_the_same_value_does_nothing() {
    let (log, on_change) = recorder();
    let model = use_controllable(None, None, || 7, Some(on_change));
    model.set(7);
    assert!(log.borrow().is_empty());
}
