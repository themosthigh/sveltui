use std::any::Any;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock, RwLock};

pub(crate) struct AnyVal(pub(crate) Box<dyn Any>);
unsafe impl Send for AnyVal {}
unsafe impl Sync for AnyVal {}

type EffectFn = Arc<dyn Fn() + Send + Sync>;

pub(crate) static SIGNAL_VALUES: LazyLock<RwLock<HashMap<SignalId, AnyVal>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

pub(crate) static SIGNAL_SUBSCRIBERS: LazyLock<RwLock<HashMap<SignalId, HashSet<ObserverId>>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

pub(crate) static EFFECTS: LazyLock<RwLock<HashMap<ObserverId, EffectFn>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

static DIRTY: AtomicBool = AtomicBool::new(true);

thread_local! {
    static CURRENT_OBSERVER: RefCell<Option<ObserverId>> = const { RefCell::new(None) };
    static BATCH_DEPTH: RefCell<usize> = const { RefCell::new(0) };
    static PENDING_EFFECTS: RefCell<HashSet<ObserverId>> = RefCell::new(HashSet::new());
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SignalId(pub(crate) usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ObserverId(pub(crate) usize);

pub struct Runtime;

impl Runtime {
    pub fn is_dirty() -> bool {
        DIRTY.load(Ordering::SeqCst)
    }

    pub fn set_dirty(dirty: bool) {
        DIRTY.store(dirty, Ordering::SeqCst);
    }

    pub fn track(signal_id: SignalId) {
        CURRENT_OBSERVER.with(|obs| {
            if let Some(obs_id) = *obs.borrow()
                && let Ok(mut subs) = SIGNAL_SUBSCRIBERS.write()
            {
                subs.entry(signal_id).or_default().insert(obs_id);
            }
        });
    }

    pub fn trigger(signal_id: SignalId) {
        DIRTY.store(true, Ordering::SeqCst);

        let observers: Vec<ObserverId> = if let Ok(subs) = SIGNAL_SUBSCRIBERS.read() {
            subs.get(&signal_id)
                .map(|set| set.iter().copied().collect())
                .unwrap_or_default()
        } else {
            Vec::new()
        };

        if observers.is_empty() {
            return;
        }

        let in_batch = BATCH_DEPTH.with(|d| *d.borrow() > 0);
        if in_batch {
            PENDING_EFFECTS.with(|pending| {
                let mut set = pending.borrow_mut();
                for obs in observers {
                    set.insert(obs);
                }
            });
        } else {
            for obs_id in observers {
                Self::run_effect(obs_id);
            }
        }
    }

    pub(crate) fn run_effect(observer_id: ObserverId) {
        let effect_fn = if let Ok(effects) = EFFECTS.read() {
            effects.get(&observer_id).cloned()
        } else {
            None
        };

        if let Some(f) = effect_fn {
            Self::with_observer(observer_id, move || {
                f();
            });
        }
    }

    pub fn with_observer<R>(observer: ObserverId, f: impl FnOnce() -> R) -> R {
        let prev = CURRENT_OBSERVER.with(|obs| {
            let prev = *obs.borrow();
            *obs.borrow_mut() = Some(observer);
            prev
        });

        let result = f();

        CURRENT_OBSERVER.with(|obs| {
            *obs.borrow_mut() = prev;
        });

        result
    }

    pub fn start_batch() {
        BATCH_DEPTH.with(|d| {
            *d.borrow_mut() += 1;
        });
    }

    pub fn end_batch() {
        let should_run = BATCH_DEPTH.with(|d| {
            let mut depth = d.borrow_mut();
            if *depth > 0 {
                *depth -= 1;
            }
            *depth == 0
        });

        if should_run {
            let pending: Vec<ObserverId> =
                PENDING_EFFECTS.with(|p| p.borrow_mut().drain().collect());
            for obs_id in pending {
                Self::run_effect(obs_id);
            }
        }
    }
}
