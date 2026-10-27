use crate::runtime::{ObserverId, Runtime};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_OBSERVER_ID: AtomicUsize = AtomicUsize::new(1);

pub fn create_effect<F: Fn() + 'static>(f: F) {
    let observer_id = ObserverId(NEXT_OBSERVER_ID.fetch_add(1, Ordering::Relaxed));
    Runtime::with_observer(observer_id, f);
}
