use std::any::Any;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;
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
    signal_subscribers: HashMap<SignalId, HashSet<ObserverId>>,
    dirty_observers: HashSet<ObserverId>,
    signal_values: HashMap<SignalId, Box<dyn Any>>,
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

pub struct ReadSignal<T> {
    id: SignalId,
    _marker: PhantomData<T>,
}

impl<T> Clone for ReadSignal<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for ReadSignal<T> {}

impl<T: Clone + 'static> ReadSignal<T> {
    pub fn get(&self) -> T {
        Runtime::track(self.id);
        RUNTIME.with(|rt| {
            let rt = rt.borrow();
            rt.signal_values
                .get(&self.id)
                .and_then(|boxed| boxed.downcast_ref::<T>())
                .cloned()
                .expect("Signal value not found")
        })
    }

    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        Runtime::track(self.id);
        RUNTIME.with(|rt| {
            let rt = rt.borrow();
            let val = rt
                .signal_values
                .get(&self.id)
                .and_then(|boxed| boxed.downcast_ref::<T>())
                .expect("Signal value not found");
            f(val)
        })
    }
}

pub struct WriteSignal<T> {
    id: SignalId,
    _marker: PhantomData<T>,
}

impl<T> Clone for WriteSignal<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for WriteSignal<T> {}

impl<T: 'static> WriteSignal<T> {
    pub fn set(&self, new_val: T) {
        RUNTIME.with(|rt| {
            let mut rt = rt.borrow_mut();
            rt.signal_values.insert(self.id, Box::new(new_val));
        });
        Runtime::trigger(self.id);
    }

    pub fn update(&self, f: impl FnOnce(&mut T)) {
        RUNTIME.with(|rt| {
            let mut rt = rt.borrow_mut();
            if let Some(val) = rt
                .signal_values
                .get_mut(&self.id)
                .and_then(|b| b.downcast_mut::<T>())
            {
                f(val);
            }
        });
        Runtime::trigger(self.id);
    }
}

pub fn create_signal<T: Clone + 'static>(initial: T) -> (ReadSignal<T>, WriteSignal<T>) {
    let id = SignalId(NEXT_SIGNAL_ID.fetch_add(1, Ordering::Relaxed));
    RUNTIME.with(|rt| {
        rt.borrow_mut().signal_values.insert(id, Box::new(initial));
    });

    (
        ReadSignal {
            id,
            _marker: PhantomData,
        },
        WriteSignal {
            id,
            _marker: PhantomData,
        },
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
    fn test_signal_copy() {
        let (name, set_name) = create_signal(String::from("Alice"));
        let n2 = name; // Copy!
        let s2 = set_name; // Copy!
        s2.set(String::from("Bob"));
        assert_eq!(n2.get(), "Bob");
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
