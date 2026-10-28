use crate::node::{IntoNode, VNode};
use crate::snippet::Snippet;
use svtui_reactive::provide_context;

#[derive(Clone, Default)]
pub struct Provider<T: 'static> {
    value: Option<T>,
    children: Option<Snippet<()>>,
}

impl<T: 'static> Provider<T> {
    pub fn new(value: T) -> Self {
        Self {
            value: Some(value),
            children: None,
        }
    }

    pub fn value(mut self, value: T) -> Self {
        self.value = Some(value);
        self
    }

    pub fn child(mut self, child: impl IntoNode) -> Self {
        let node = child.into_node();
        self.children = Some(Snippet::new(move |_| node.clone()));
        self
    }

    pub fn snippet(mut self, snippet: Snippet<()>) -> Self {
        self.children = Some(snippet);
        self
    }
}

impl<T: 'static> IntoNode for Provider<T> {
    fn into_node(self) -> VNode {
        if let Some(val) = self.value {
            provide_context(val, || {
                if let Some(children) = self.children {
                    children.render(())
                } else {
                    VNode::Empty
                }
            })
        } else if let Some(children) = self.children {
            children.render(())
        } else {
            VNode::Empty
        }
    }
}
