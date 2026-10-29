use crate::runtime::{AnyVal, Runtime, SIGNAL_VALUES, SignalId};
use std::fmt;
use std::marker::PhantomData;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_SIGNAL_ID: AtomicUsize = AtomicUsize::new(1);

pub struct ReadSignal<T> {
    id: SignalId,
    _marker: PhantomData<T>,
}

impl<T> fmt::Debug for ReadSignal<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ReadSignal").field(&self.id).finish()
    }
}

impl<T> Clone for ReadSignal<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for ReadSignal<T> {}

impl<T> PartialEq for ReadSignal<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T> Eq for ReadSignal<T> {}

impl<T: Clone + 'static> ReadSignal<T> {
    pub fn get(&self) -> T {
        Runtime::track(self.id);
        let vals = SIGNAL_VALUES.read().expect("SIGNAL_VALUES lock poisoned");
        vals.get(&self.id)
            .and_then(|any_val| any_val.0.downcast_ref::<T>())
            .cloned()
            .expect("Signal value not found")
    }

    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        Runtime::track(self.id);
        let vals = SIGNAL_VALUES.read().expect("SIGNAL_VALUES lock poisoned");
        let val = vals
            .get(&self.id)
            .and_then(|any_val| any_val.0.downcast_ref::<T>())
            .expect("Signal value not found");
        f(val)
    }

    pub fn id(&self) -> SignalId {
        self.id
    }
}

pub struct WriteSignal<T> {
    id: SignalId,
    _marker: PhantomData<T>,
}

impl<T> fmt::Debug for WriteSignal<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("WriteSignal").field(&self.id).finish()
    }
}

impl<T> Clone for WriteSignal<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for WriteSignal<T> {}

impl<T> PartialEq for WriteSignal<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T> Eq for WriteSignal<T> {}

impl<T: 'static> WriteSignal<T> {
    pub fn set(&self, new_val: T) {
        {
            let mut vals = SIGNAL_VALUES.write().expect("SIGNAL_VALUES lock poisoned");
            vals.insert(self.id, AnyVal(Box::new(new_val)));
        }
        Runtime::trigger(self.id);
    }

    pub fn update(&self, f: impl FnOnce(&mut T)) {
        {
            let mut vals = SIGNAL_VALUES.write().expect("SIGNAL_VALUES lock poisoned");
            if let Some(val) = vals.get_mut(&self.id).and_then(|b| b.0.downcast_mut::<T>()) {
                f(val);
            }
        }
        Runtime::trigger(self.id);
    }

    pub fn id(&self) -> SignalId {
        self.id
    }
}

pub fn create_signal<T: Clone + 'static>(initial: T) -> (ReadSignal<T>, WriteSignal<T>) {
    let id = SignalId(NEXT_SIGNAL_ID.fetch_add(1, Ordering::Relaxed));
    {
        let mut vals = SIGNAL_VALUES.write().expect("SIGNAL_VALUES lock poisoned");
        vals.insert(id, AnyVal(Box::new(initial)));
    }

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

pub struct State<T> {
    read: ReadSignal<T>,
    write: WriteSignal<T>,
}

impl<T> fmt::Debug for State<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("State").field("id", &self.read.id).finish()
    }
}

impl<T> Clone for State<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for State<T> {}

impl<T> PartialEq for State<T> {
    fn eq(&self, other: &Self) -> bool {
        self.read.id == other.read.id
    }
}

impl<T> Eq for State<T> {}

impl<T: Clone + 'static> State<T> {
    pub fn get(&self) -> T {
        self.read.get()
    }

    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        self.read.with(f)
    }

    pub fn set(&self, new_val: T) {
        self.write.set(new_val);
    }

    pub fn update(&self, f: impl FnOnce(&mut T)) {
        self.write.update(f);
    }

    pub fn read(&self) -> ReadSignal<T> {
        self.read
    }

    pub fn write(&self) -> WriteSignal<T> {
        self.write
    }

    pub fn id(&self) -> SignalId {
        self.read.id
    }
}

impl<T> From<State<T>> for ReadSignal<T> {
    fn from(s: State<T>) -> Self {
        s.read
    }
}

impl<T> From<State<T>> for WriteSignal<T> {
    fn from(s: State<T>) -> Self {
        s.write
    }
}

pub fn state<T: Clone + 'static>(initial: T) -> State<T> {
    state_with(move || initial)
}

pub fn state_with<T: Clone + 'static>(init: impl FnOnce() -> T) -> State<T> {
    crate::lifecycle::use_hook(|| {
        let (read, write) = create_signal(init());
        State { read, write }
    })
}

#[derive(Clone)]
pub struct Derived<T> {
    calc: Arc<dyn Fn() -> T + Send + Sync>,
}

impl<T: Clone + 'static> Derived<T> {
    pub fn get(&self) -> T {
        (self.calc)()
    }
}

impl<T: fmt::Debug + Clone + 'static> fmt::Debug for Derived<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Derived").field(&self.get()).finish()
    }
}

pub fn derived<T: Clone + 'static>(f: impl Fn() -> T + Send + Sync + 'static) -> Derived<T> {
    Derived { calc: Arc::new(f) }
}
