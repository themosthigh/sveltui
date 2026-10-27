use std::any::Any;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

thread_local! {
    pub(crate) static RUNTIME: RefCell<Runtime> = RefCell::new(Runtime::new());
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SignalId(pub(crate) usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ObserverId(pub(crate) usize);

pub struct Runtime {
    pub(crate) current_observer: Option<ObserverId>,
    pub(crate) dirty: bool,
    pub(crate) signal_subscribers: HashMap<SignalId, HashSet<ObserverId>>,
    pub(crate) dirty_observers: HashSet<ObserverId>,
    pub(crate) signal_values: HashMap<SignalId, Box<dyn Any>>,
}

impl Runtime {
    fn new() -> Self {
        Self {
            current_observer: None,
            dirty: true,
            signal_subscribers: HashMap::new(),
            dirty_observers: HashSet::new(),
            signal_values: HashMap::new(),
        }
    }

    pub fn is_dirty() -> bool {
        RUNTIME.with(|rt| rt.borrow().dirty)
    }

    pub fn set_dirty(dirty: bool) {
        RUNTIME.with(|rt| rt.borrow_mut().dirty = dirty);
    }

    pub fn track(signal_id: SignalId) {
        RUNTIME.with(|rt| {
            let mut rt = rt.borrow_mut();
            if let Some(obs) = rt.current_observer {
                rt.signal_subscribers
                    .entry(signal_id)
                    .or_default()
                    .insert(obs);
            }
        });
    }

    pub fn trigger(signal_id: SignalId) {
        RUNTIME.with(|rt| {
            let mut rt = rt.borrow_mut();
            rt.dirty = true;
            if let Some(subscribers) = rt.signal_subscribers.get(&signal_id) {
                let obs_to_insert: Vec<ObserverId> = subscribers.iter().copied().collect();
                for obs in obs_to_insert {
                    rt.dirty_observers.insert(obs);
                }
            }
        });
    }

    pub fn with_observer<R>(observer: ObserverId, f: impl FnOnce() -> R) -> R {
        let prev = RUNTIME.with(|rt| {
            let mut rt = rt.borrow_mut();
            let prev = rt.current_observer;
            rt.current_observer = Some(observer);
            prev
        });

        let result = f();

        RUNTIME.with(|rt| {
            rt.borrow_mut().current_observer = prev;
        });

        result
    }
}
