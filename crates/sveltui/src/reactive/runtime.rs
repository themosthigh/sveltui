use std::any::Any;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock, Mutex, RwLock};
use std::time::Instant;

/// A signal's value, and its version: a number that changes on every write.
pub(crate) struct AnyVal(pub(crate) Box<dyn Any>, pub(crate) u64);

static NEXT_VERSION: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

/// A version no other write has had.
pub(crate) fn next_version() -> u64 {
    NEXT_VERSION.fetch_add(1, Ordering::Relaxed)
}
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

/// Set when something asked for the terminal bell on the next frame.
static BELL: AtomicBool = AtomicBool::new(false);

/// The earliest moment something asked to be drawn again, if any.
static WAKE_AT: Mutex<Option<Instant>> = Mutex::new(None);

thread_local! {
    static CURRENT_OBSERVER: RefCell<Option<ObserverId>> = const { RefCell::new(None) };
    static BATCH_DEPTH: RefCell<usize> = const { RefCell::new(0) };
    static PENDING_EFFECTS: RefCell<HashSet<ObserverId>> = RefCell::new(HashSet::new());
    /// One list per `collect_reads` call in progress, innermost last.
    static COLLECTORS: RefCell<Vec<Vec<SignalId>>> = const { RefCell::new(Vec::new()) };
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

    /// Asks for a frame once `at` has passed, for things that change with
    /// time rather than input (a toast timing out). The earliest request
    /// wins, so asking again every frame is fine.
    pub fn wake_at(at: Instant) {
        if let Ok(mut wake) = WAKE_AT.lock() {
            *wake = Some(wake.map_or(at, |current| current.min(at)));
        }
    }

    /// When the next requested frame is due, if one is.
    pub fn next_wake() -> Option<Instant> {
        WAKE_AT.lock().ok().and_then(|wake| *wake)
    }

    /// Marks the runtime dirty if the requested frame is due by `now`, and
    /// says whether it was. A due request is used up.
    pub fn wake_if_due(now: Instant) -> bool {
        let Ok(mut wake) = WAKE_AT.lock() else {
            return false;
        };
        match *wake {
            Some(at) if at <= now => {
                *wake = None;
                DIRTY.store(true, Ordering::SeqCst);
                true
            }
            _ => false,
        }
    }

    /// Rings the terminal bell with the next frame, to say "no" without
    /// moving anything (a press outside a dialog that has to be answered).
    pub fn bell() {
        BELL.store(true, Ordering::SeqCst);
        DIRTY.store(true, Ordering::SeqCst);
    }

    /// Whether a bell was asked for since the last call. The terminal asks
    /// once per frame.
    pub fn take_bell() -> bool {
        BELL.swap(false, Ordering::SeqCst)
    }

    /// Runs `f` and says which signals it read, each with the version it
    /// had when read. Comparing those versions later (`reads_changed`) tells
    /// us whether `f` would give a different answer, without running it.
    /// Reads still reach any observer around us as well.
    pub fn collect_reads<R>(f: impl FnOnce() -> R) -> (R, Vec<(SignalId, u64)>) {
        COLLECTORS.with(|c| c.borrow_mut().push(Vec::new()));
        let result = f();
        let ids = COLLECTORS.with(|c| c.borrow_mut().pop().unwrap_or_default());
        let reads = ids
            .into_iter()
            .map(|id| (id, Self::signal_version(id)))
            .collect();
        (result, reads)
    }

    /// Whether any signal in `reads` has been written since its version was
    /// noted.
    pub fn reads_changed(reads: &[(SignalId, u64)]) -> bool {
        reads
            .iter()
            .any(|&(id, version)| Self::signal_version(id) != version)
    }

    fn signal_version(id: SignalId) -> u64 {
        SIGNAL_VALUES
            .read()
            .ok()
            .and_then(|vals| vals.get(&id).map(|entry| entry.1))
            .unwrap_or(0)
    }

    pub fn track(signal_id: SignalId) {
        COLLECTORS.with(|c| {
            if let Some(list) = c.borrow_mut().last_mut()
                && !list.contains(&signal_id)
            {
                list.push(signal_id);
            }
        });
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
