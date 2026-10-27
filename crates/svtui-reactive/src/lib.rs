use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_SIGNAL_ID: AtomicUsize = AtomicUsize::new(1);
static NEXT_OBSERVER_ID: AtomicUsize = AtomicUsize::new(1);

thread_local! {
    static RUNTIME: RefCell<Runtime> = RefCell::new(Runtime::new());
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SignalId(usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ObserverId(usize);

pub struct Runtime {
    current_observer: Option<ObserverId>,
    dirty: bool,
    signal_subscribers: std::collections::HashMap<SignalId, HashSet<ObserverId>>,
    dirty_observers: HashSet<ObserverId>,
}

impl Runtime {
    fn new() -> Self {
        Self {
            current_observer: None,
            dirty: true,
            signal_subscribers: std::collections::HashMap::new(),
            dirty_observers: HashSet::new(),
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

pub struct ReadSignal<T> {
    id: SignalId,
    value: Rc<RefCell<T>>,
}

impl<T> Clone for ReadSignal<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            value: Rc::clone(&self.value),
        }
    }
}

impl<T: Clone + 'static> ReadSignal<T> {
    pub fn get(&self) -> T {
        Runtime::track(self.id);
        self.value.borrow().clone()
    }

    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        Runtime::track(self.id);
        f(&self.value.borrow())
    }
}

pub struct WriteSignal<T> {
    id: SignalId,
    value: Rc<RefCell<T>>,
}

impl<T> Clone for WriteSignal<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            value: Rc::clone(&self.value),
        }
    }
}

impl<T: 'static> WriteSignal<T> {
    pub fn set(&self, new_val: T) {
        *self.value.borrow_mut() = new_val;
        Runtime::trigger(self.id);
    }

    pub fn update(&self, f: impl FnOnce(&mut T)) {
        f(&mut self.value.borrow_mut());
        Runtime::trigger(self.id);
    }
}

pub fn create_signal<T: Clone + 'static>(initial: T) -> (ReadSignal<T>, WriteSignal<T>) {
    let id = SignalId(NEXT_SIGNAL_ID.fetch_add(1, Ordering::Relaxed));
    let value = Rc::new(RefCell::new(initial));

    (
        ReadSignal {
            id,
            value: Rc::clone(&value),
        },
        WriteSignal { id, value },
    )
}

pub fn create_effect<F: Fn() + 'static>(f: F) {
    let observer_id = ObserverId(NEXT_OBSERVER_ID.fetch_add(1, Ordering::Relaxed));
    Runtime::with_observer(observer_id, f);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_read_write() {
        let (count, set_count) = create_signal(0);
        assert_eq!(count.get(), 0);
        set_count.set(42);
        assert_eq!(count.get(), 42);
        set_count.update(|c| *c += 1);
        assert_eq!(count.get(), 43);
    }

    #[test]
    fn test_runtime_dirty_tracking() {
        Runtime::set_dirty(false);
        assert!(!Runtime::is_dirty());

        let (count, set_count) = create_signal("hello");
        let _ = count.get();
        set_count.set("world");
        assert!(Runtime::is_dirty());
    }
}
