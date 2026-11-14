use std::rc::Rc;
use sveltui::prelude::*;

/// A piece of component state with three possible owners. A `Binding`
/// (from `bind:x`, a state or a getter and setter) is read and written directly. A plain value (from `x`)
/// is read-only: changes are only reported, and nothing moves until the
/// parent passes a new value. With neither, the component keeps its own
/// local state.
pub(crate) struct Controllable<T: 'static> {
    bound: Option<Binding<T>>,
    value: Option<T>,
    local: State<T>,
    on_change: Option<Rc<dyn Fn(T)>>,
}

impl<T: Clone + 'static> Clone for Controllable<T> {
    fn clone(&self) -> Self {
        Self {
            bound: self.bound.clone(),
            value: self.value.clone(),
            local: self.local,
            on_change: self.on_change.clone(),
        }
    }
}

/// Sets up the state for one prop. Call it once on every render; the local
/// state is always created, so the hook order never changes.
pub(crate) fn use_controllable<T: Clone + PartialEq + 'static>(
    bound: Option<Binding<T>>,
    value: Option<T>,
    default: impl FnOnce() -> T,
    on_change: Option<Rc<dyn Fn(T)>>,
) -> Controllable<T> {
    let local = state_with(default);
    Controllable {
        bound,
        value,
        local,
        on_change,
    }
}

impl<T: Clone + PartialEq + 'static> Controllable<T> {
    pub(crate) fn get(&self) -> T {
        match (&self.bound, &self.value) {
            (Some(bound), _) => bound.get(),
            (None, Some(value)) => value.clone(),
            (None, None) => self.local.get(),
        }
    }

    /// Changes the value: writes the bound or local state (never a
    /// read-only value), then calls `on_change`. Setting the current value
    /// again does nothing.
    pub(crate) fn set(&self, next: T) {
        if self.get() == next {
            return;
        }
        match (&self.bound, &self.value) {
            (Some(bound), _) => bound.set(next.clone()),
            (None, None) => self.local.set(next.clone()),
            (None, Some(_)) => {}
        }
        if let Some(callback) = &self.on_change {
            callback(next);
        }
    }
}
