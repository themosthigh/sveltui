use crate::runtime::{RUNTIME, Runtime, SignalId};
use std::marker::PhantomData;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_SIGNAL_ID: AtomicUsize = AtomicUsize::new(1);

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
