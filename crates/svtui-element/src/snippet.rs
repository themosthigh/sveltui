use crate::node::{IntoNode, VNode};
use std::fmt;
use std::rc::Rc;

pub trait Props: Sized {
    type Builder: Default;
}

#[derive(Clone)]
pub struct Snippet<T = ()>(Rc<dyn Fn(T) -> VNode>);

impl<T> fmt::Debug for Snippet<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Snippet").finish()
    }
}

impl<T> Snippet<T> {
    pub fn new(f: impl Fn(T) -> VNode + 'static) -> Self {
        Self(Rc::new(f))
    }

    pub fn render(&self, arg: T) -> VNode {
        (self.0)(arg)
    }
}

impl Snippet<()> {
    pub fn render_empty(&self) -> VNode {
        (self.0)(())
    }
}

impl IntoNode for Snippet<()> {
    fn into_node(self) -> VNode {
        (self.0)(())
    }
}

impl IntoNode for &Snippet<()> {
    fn into_node(self) -> VNode {
        (self.0)(())
    }
}
