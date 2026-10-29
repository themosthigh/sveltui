use crate::reactive::runtime::{EFFECTS, ObserverId, Runtime};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_OBSERVER_ID: AtomicUsize = AtomicUsize::new(1);

pub fn effect<F: Fn() + Send + Sync + 'static>(f: F) {
    crate::reactive::lifecycle::use_hook(|| {
        let observer_id = ObserverId(NEXT_OBSERVER_ID.fetch_add(1, Ordering::Relaxed));
        let arc_f = Arc::new(f);
        if let Ok(mut effects) = EFFECTS.write() {
            effects.insert(observer_id, arc_f);
        }
        Runtime::run_effect(observer_id);
        crate::reactive::lifecycle::ComponentScope::register_cleanup(Box::new(move || {
            if let Ok(mut effects) = EFFECTS.write() {
                effects.remove(&observer_id);
            }
        }));
        observer_id
    });
}

pub fn create_effect<F: Fn() + Send + Sync + 'static>(f: F) {
    effect(f);
}

pub fn batch<R>(f: impl FnOnce() -> R) -> R {
    Runtime::start_batch();
    let result = f();
    Runtime::end_batch();
    result
}
