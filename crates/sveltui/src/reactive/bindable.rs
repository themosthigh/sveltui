use crate::reactive::State;

/// A prop that can be passed read-only (`x={value}`) or bound two-way
/// (`bind:x={state}`). It only carries what the parent passed; the component
/// decides what to do with it. Mark the field `#[prop(bindable)]` and the
/// `Props` derive gives the builder an `x(..)` setter for the value and a
/// `bind_x(..)` setter for the state.
pub struct Bindable<T: 'static> {
    value: Option<T>,
    bound: Option<State<T>>,
}

impl<T: 'static> Bindable<T> {
    /// The read-only value, if one was passed.
    pub fn value(&self) -> Option<T>
    where
        T: Clone,
    {
        self.value.clone()
    }

    /// The bound state, if one was passed.
    pub fn bound(&self) -> Option<State<T>> {
        self.bound
    }

    /// Whether the parent passed anything at all.
    pub fn is_set(&self) -> bool {
        self.value.is_some() || self.bound.is_some()
    }

    /// Splits into the read-only value and the bound state.
    pub fn into_parts(self) -> (Option<T>, Option<State<T>>) {
        (self.value, self.bound)
    }

    /// Replaces the read-only value. Used by the generated `x(..)` setter.
    pub fn set_value(&mut self, value: Option<T>) {
        self.value = value;
    }

    /// Replaces the bound state. Used by the generated `bind_x(..)` setter.
    pub fn set_bound(&mut self, bound: Option<State<T>>) {
        self.bound = bound;
    }
}

impl<T: 'static> Default for Bindable<T> {
    fn default() -> Self {
        Self {
            value: None,
            bound: None,
        }
    }
}

impl<T: Clone + 'static> Clone for Bindable<T> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            bound: self.bound,
        }
    }
}

impl<T: std::fmt::Debug + 'static> std::fmt::Debug for Bindable<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Bindable")
            .field("value", &self.value)
            .field("bound", &self.bound)
            .finish()
    }
}

/// What `bind:x={..}` accepts. A `State<T>` binds the prop. A `Bindable<T>`
/// (usually a parent's own `props.x`, passed on as it is) carries over
/// whatever it holds: the bound state, the read-only value, or both. This
/// way a component hands its prop to a child without checking which kind
/// the parent gave.
pub trait IntoBinding<T: 'static> {
    fn into_binding(self) -> Bindable<T>;
}

impl<T: 'static> IntoBinding<T> for State<T> {
    fn into_binding(self) -> Bindable<T> {
        Bindable {
            value: None,
            bound: Some(self),
        }
    }
}

impl<T: 'static> IntoBinding<T> for Option<State<T>> {
    fn into_binding(self) -> Bindable<T> {
        Bindable {
            value: None,
            bound: self,
        }
    }
}

impl<T: 'static> IntoBinding<T> for Bindable<T> {
    fn into_binding(self) -> Bindable<T> {
        self
    }
}

impl<T: Clone + 'static> IntoBinding<T> for &Bindable<T> {
    fn into_binding(self) -> Bindable<T> {
        self.clone()
    }
}
