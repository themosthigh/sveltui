use crossterm::event::KeyEvent;
use std::rc::Rc;
use svtui_element::{EventContext, EventResult, KeyHandler};
use svtui_element::{FocusId, is_layer_open};
use svtui_element::{IntoNode, OverlayElement, VNode};
use svtui_layout::Placement;

/// Anchored floating content that traps Tab focus while open.
/// Escape or outside click requests dismissal and restores focus to the trigger.
#[derive(Clone, Default)]
pub struct Popover {
    id: Option<FocusId>,
    trigger: Option<VNode>,
    content: Option<VNode>,
    open: Option<bool>,
    placement: Placement,
    backdrop: bool,
    on_close: Option<Rc<dyn Fn()>>,
    on_open: Option<Rc<dyn Fn()>>,
    on_key: Option<KeyHandler>,
}

impl Popover {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn id(mut self, id: FocusId) -> Self {
        self.id = Some(id);
        self
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
        self.open = Some(open);
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

    pub fn on_key(
        mut self,
        handler: impl Fn(KeyEvent, &mut EventContext) -> EventResult + 'static,
    ) -> Self {
        self.on_key = Some(Rc::new(handler));
        self
    }

    pub fn on_open(mut self, handler: impl Fn() + 'static) -> Self {
        self.on_open = Some(Rc::new(handler));
        self
    }

    pub fn on_close(mut self, handler: impl Fn() + 'static) -> Self {
        self.on_close = Some(Rc::new(handler));
        self
    }
}

impl IntoNode for Popover {
    fn into_node(self) -> VNode {
        let id = self.id.unwrap_or_default();
        let managed = self.open.is_none();
        let open = self.open.unwrap_or_else(|| is_layer_open(id));
        let mut overlay = OverlayElement::new()
            .id(id)
            .open(open)
            .managed(managed)
            .placement(self.placement)
            .backdrop(self.backdrop);

        if let Some(t) = self.trigger {
            overlay = overlay.trigger(t);
        }
        if let Some(c) = self.content {
            overlay = overlay.content(c);
        }
        overlay.on_key = self.on_key;
        overlay.on_open = self.on_open;
        if let Some(h) = self.on_close {
            overlay.on_close = Some(h);
        }

        overlay.into_node()
    }
}
