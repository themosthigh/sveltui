use crate::node::{IntoNode, VNode};
use crate::snippet::Snippet;
use std::panic::{AssertUnwindSafe, catch_unwind};

pub struct ErrorBoundary {
    fallback: Option<Snippet<String>>,
    children: Option<Box<dyn FnOnce() -> VNode>>,
}

impl Default for ErrorBoundary {
    fn default() -> Self {
        Self::new()
    }
}

impl ErrorBoundary {
    pub fn new() -> Self {
        Self {
            fallback: None,
            children: None,
        }
    }

    pub fn fallback(mut self, fallback: impl Into<Snippet<String>>) -> Self {
        self.fallback = Some(fallback.into());
        self
    }

    pub fn child(mut self, child: impl IntoNode + 'static) -> Self {
        self.children = Some(Box::new(move || child.into_node()));
        self
    }

    pub fn render_child(mut self, f: impl FnOnce() -> VNode + 'static) -> Self {
        self.children = Some(Box::new(f));
        self
    }
}

impl IntoNode for ErrorBoundary {
    fn into_node(self) -> VNode {
        let fallback = self.fallback;
        let children = self.children;

        let res = catch_unwind(AssertUnwindSafe(|| {
            if let Some(c) = children {
                c()
            } else {
                VNode::Empty
            }
        }));

        match res {
            Ok(node) => node,
            Err(err) => {
                let err_msg = if let Some(s) = err.downcast_ref::<&str>() {
                    s.to_string()
                } else if let Some(s) = err.downcast_ref::<String>() {
                    s.clone()
                } else {
                    "Component crashed".to_string()
                };

                if let Some(fb) = fallback {
                    fb.render(err_msg)
                } else {
                    VNode::Empty
                }
            }
        }
    }
}
