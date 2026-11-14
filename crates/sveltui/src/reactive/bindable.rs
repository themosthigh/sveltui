use crate::reactive::State;
use std::rc::Rc;

/// Where a bound prop reads and writes its value: a `State`, or a getter
/// and setter the parent wrote itself. A component only calls `get` and
/// `set`, so it never needs to know which one it was given.
pub enum Binding<T: 'static> {
    State(State<T>),
    Fn {
        get: Rc<dyn Fn() -> T>,
        set: Rc<dyn Fn(T)>,
    },
}

impl<T: Clone + 'static> Binding<T> {
    pub fn get(&self) -> T {
        match self {
            Binding::State(state) => state.get(),
            Binding::Fn { get, .. } => get(),
        }
    }

    pub fn set(&self, value: T) {
        match self {
            Binding::State(state) => state.set(value),
            Binding::Fn { set, .. } => set(value),
        }
    }

    /// Reads the value, lets `f` change it, and writes it back.
    pub fn update(&self, f: impl FnOnce(&mut T)) {
        let mut value = self.get();
        f(&mut value);
        self.set(value);
    }

    /// A binding made of a getter and a setter, as `bind:x={(get, set)}`
    /// writes it.
    pub fn from_fns(get: impl Fn() -> T + 'static, set: impl Fn(T) + 'static) -> Self {
        Binding::Fn {
            get: Rc::new(get),
            set: Rc::new(set),
        }
    }
}

// Not derived: a derive would ask `T: Clone`, but only the `Rc`s and the
// `State` handle are copied.
impl<T: 'static> Clone for Binding<T> {
    fn clone(&self) -> Self {
        match self {
            Binding::State(state) => Binding::State(*state),
            Binding::Fn { get, set } => Binding::Fn {
                get: get.clone(),
                set: set.clone(),
            },
        }
    }
}

impl<T: 'static> PartialEq for Binding<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Binding::State(a), Binding::State(b)) => a == b,
            (Binding::Fn { get: g1, set: s1 }, Binding::Fn { get: g2, set: s2 }) => {
                Rc::ptr_eq(g1, g2) && Rc::ptr_eq(s1, s2)
            }
            _ => false,
        }
    }
}

impl<T: 'static> std::fmt::Debug for Binding<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Binding::State(state) => f.debug_tuple("Binding::State").field(state).finish(),
            Binding::Fn { .. } => f.write_str("Binding::Fn"),
        }
    }
}

/// A prop that can be passed read-only (`x={value}`) or bound two-way
/// (`bind:x={state}`). It only carries what the parent passed; the component
/// decides what to do with it. Mark the field `#[prop(bindable)]` and the
/// `Props` derive gives the builder an `x(..)` setter for the value and a
/// `bind_x(..)` setter for the binding.
///
/// When a parent passes both `x=` and `bind:x=`, the binding wins: the
/// component reads and writes through it and ignores the read-only value.
pub struct Bindable<T: 'static> {
    value: Option<T>,
    bound: Option<Binding<T>>,
}

impl<T: 'static> Bindable<T> {
    /// The read-only value, if one was passed.
    pub fn value(&self) -> Option<T>
    where
        T: Clone,
    {
        self.value.clone()
    }

    /// The binding, if one was passed.
    pub fn bound(&self) -> Option<Binding<T>> {
        self.bound.clone()
    }

    /// Whether the parent passed anything at all.
    pub fn is_set(&self) -> bool {
        self.value.is_some() || self.bound.is_some()
    }

    /// Splits into the read-only value and the binding.
    pub fn into_parts(self) -> (Option<T>, Option<Binding<T>>) {
        (self.value, self.bound)
    }

    /// Replaces the read-only value. Used by the generated `x(..)` setter.
    pub fn set_value(&mut self, value: Option<T>) {
        self.value = value;
    }

    /// Replaces the binding. Used by the generated `bind_x(..)` setter.
    pub fn set_bound(&mut self, bound: Option<Binding<T>>) {
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
            bound: self.bound.clone(),
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

/// What `bind:x={..}` accepts. A `State<T>` or a `(get, set)` pair of
/// closures binds the prop. A `Bindable<T>` (usually a parent's own
/// `props.x`, passed on as it is) carries over whatever it holds: the
/// binding, the read-only value, or both. This way a component hands its
/// prop to a child without checking which kind the parent gave.
pub trait IntoBinding<T: 'static> {
    fn into_binding(self) -> Bindable<T>;
}

impl<T: 'static> IntoBinding<T> for Binding<T> {
    fn into_binding(self) -> Bindable<T> {
        Bindable {
            value: None,
            bound: Some(self),
        }
    }
}

impl<T: 'static> IntoBinding<T> for Option<Binding<T>> {
    fn into_binding(self) -> Bindable<T> {
        Bindable {
            value: None,
            bound: self,
        }
    }
}

impl<T: 'static> IntoBinding<T> for State<T> {
    fn into_binding(self) -> Bindable<T> {
        Binding::State(self).into_binding()
    }
}

impl<T: 'static> IntoBinding<T> for Option<State<T>> {
    fn into_binding(self) -> Bindable<T> {
        self.map(Binding::State).into_binding()
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

impl<T, G, S> IntoBinding<T> for (G, S)
where
    T: Clone + 'static,
    G: Fn() -> T + 'static,
    S: Fn(T) + 'static,
{
    fn into_binding(self) -> Bindable<T> {
        Binding::from_fns(self.0, self.1).into_binding()
    }
}
