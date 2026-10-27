use std::rc::Rc;
use svtui_element::{IntoNode, OverlayElement, VNode};
use svtui_layout::Placement;

/// Declarative popover component that anchors floating content to a trigger element.
#[derive(Clone, Default)]
pub struct Popover {
    trigger: Option<VNode>,
    content: Option<VNode>,
    open: bool,
    placement: Placement,
    backdrop: bool,
    on_close: Option<Rc<dyn Fn()>>,
}

impl Popover {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn trigger(mut self, trigger: impl IntoNode) -> Self {
        self.trigger = Some(trigger.into_node());
        self
    }

    pub fn content(mut self, content: impl IntoNode) -> Self {
        self.content = Some(content.into_node());
        self
    }

    pub fn child(mut self, child: impl IntoNode) -> Self {
        self.content = Some(child.into_node());
        self
    }

    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    pub fn placement(mut self, placement: Placement) -> Self {
        self.placement = placement;
        self
    }

    pub fn backdrop(mut self, backdrop: bool) -> Self {
        self.backdrop = backdrop;
        self
    }

    pub fn on_close(mut self, handler: impl Fn() + 'static) -> Self {
        self.on_close = Some(Rc::new(handler));
        self
    }
}

impl IntoNode for Popover {
    fn into_node(self) -> VNode {
        let mut overlay = OverlayElement::new()
            .open(self.open)
            .placement(self.placement)
            .backdrop(self.backdrop);

        if let Some(t) = self.trigger {
            overlay = overlay.trigger(t);
        }
        if let Some(c) = self.content {
            overlay = overlay.content(c);
        }
        if let Some(h) = self.on_close {
            overlay.on_close = Some(h);
        }

        overlay.into_node()
    }
}
