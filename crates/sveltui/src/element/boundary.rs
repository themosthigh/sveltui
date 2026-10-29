use crate::element::node::VNode;
use crate::element::snippet::Snippet;
use std::panic::{AssertUnwindSafe, catch_unwind};

#[derive(Default)]
pub struct ErrorBoundaryProps {
    pub fallback: Option<Snippet<String>>,
    pub children: Option<Snippet<()>>,
}

#[derive(Default)]
pub struct ErrorBoundaryPropsBuilder {
    fallback: Option<Snippet<String>>,
    children: Option<Snippet<()>>,
}

impl ErrorBoundaryProps {
    pub fn builder() -> ErrorBoundaryPropsBuilder {
        ErrorBoundaryPropsBuilder::default()
    }
}

impl ErrorBoundaryPropsBuilder {
    pub fn slot_fallback(mut self, s: Snippet<String>) -> Self {
        self.fallback = Some(s);
        self
    }
    pub fn fallback(self, s: Snippet<String>) -> Self {
        self.slot_fallback(s)
    }
    pub fn slot_children(mut self, s: Snippet<()>) -> Self {
        self.children = Some(s);
        self
    }
    pub fn children(self, s: Snippet<()>) -> Self {
        self.slot_children(s)
    }
    pub fn build(self) -> ErrorBoundaryProps {
        ErrorBoundaryProps {
            fallback: self.fallback,
            children: self.children,
        }
    }
}

#[allow(non_snake_case)]
pub fn ErrorBoundary(props: ErrorBoundaryProps) -> VNode {
    let fallback = props.fallback;
    let children = props.children;

    let res = catch_unwind(AssertUnwindSafe(|| {
        if let Some(c) = children {
            c.render(())
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
