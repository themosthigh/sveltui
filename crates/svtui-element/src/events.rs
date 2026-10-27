use crate::focus::FocusId;
use crossterm::event::KeyEvent;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use svtui_layout::Rect;

#[derive(Clone)]
pub struct HitTarget {
    pub rect: Rect,
    pub on_click: Option<Rc<dyn Fn()>>,
    pub focus_id: Option<FocusId>,
    pub on_key: Option<Rc<dyn Fn(KeyEvent)>>,
    pub on_scroll: Option<Rc<dyn Fn(i16, i16)>>,
}

#[derive(Default, Clone)]
pub struct HitMap {
    pub targets: Vec<HitTarget>,
    pub focus_stack: Vec<FocusId>,
    pub ancestors: HashMap<FocusId, HashSet<FocusId>>,
}

impl HitMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.targets.clear();
        self.focus_stack.clear();
        self.ancestors.clear();
    }

    pub fn register(&mut self, target: HitTarget) {
        self.targets.push(target);
    }

    /// Finds the innermost matching HitTarget containing (x, y).
    pub fn hit_test(&self, x: u16, y: u16) -> Option<&HitTarget> {
        self.targets
            .iter()
            .rev()
            .find(|target| target.rect.contains(x, y))
    }

    /// Finds the innermost matching HitTarget containing (x, y) that supports scrolling.
    pub fn scroll_target(&self, x: u16, y: u16) -> Option<&HitTarget> {
        self.targets
            .iter()
            .rev()
            .find(|target| target.rect.contains(x, y) && target.on_scroll.is_some())
    }

    /// Returns all registered focusable IDs in DOM order.
    pub fn focusable_ids(&self) -> Vec<FocusId> {
        let mut ids = Vec::new();
        for target in &self.targets {
            if let Some(id) = target.focus_id.filter(|id| !ids.contains(id)) {
                ids.push(id);
            }
        }
        ids
    }
}
