use crate::element::node::VNode;
use crate::element::snippet::Snippet;
use crate::reactive::provide_context;

pub struct ContextProviderProps<T: 'static> {
    pub value: Option<T>,
    pub children: Option<Snippet<()>>,
}

impl<T: 'static> Default for ContextProviderProps<T> {
    fn default() -> Self {
        Self {
            value: None,
            children: None,
        }
    }
}

pub struct ContextProviderPropsBuilder<T: 'static> {
    value: Option<T>,
    children: Option<Snippet<()>>,
}

impl<T: 'static> Default for ContextProviderPropsBuilder<T> {
    fn default() -> Self {
        Self {
            value: None,
            children: None,
        }
    }
}

impl<T: 'static> ContextProviderProps<T> {
    pub fn builder() -> ContextProviderPropsBuilder<T> {
        ContextProviderPropsBuilder::default()
    }
}

impl<T: 'static> ContextProviderPropsBuilder<T> {
    pub fn value(mut self, value: T) -> Self {
        self.value = Some(value);
        self
    }
    pub fn slot_children(mut self, s: Snippet<()>) -> Self {
        self.children = Some(s);
        self
    }
    pub fn children(self, s: Snippet<()>) -> Self {
        self.slot_children(s)
    }
    pub fn build(self) -> ContextProviderProps<T> {
        ContextProviderProps {
            value: self.value,
            children: self.children,
        }
    }
}

#[allow(non_snake_case)]
pub fn ContextProvider<T: 'static>(props: ContextProviderProps<T>) -> VNode {
    if let Some(val) = props.value {
        provide_context(val, || {
            if let Some(children) = props.children {
                children.render(())
            } else {
                VNode::Empty
            }
        })
    } else if let Some(children) = props.children {
        children.render(())
    } else {
        VNode::Empty
    }
}
