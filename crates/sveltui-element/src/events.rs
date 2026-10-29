use crate::focus::FocusId;
use crate::interaction::{FocusManager, InteractionTree, KeyHandler};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use sveltui_layout::Rect;

#[derive(Clone)]
pub struct HitTarget {
    pub rect: Rect,
    pub layer: Option<FocusId>,
    pub dismiss: Option<FocusId>,
    pub on_click: Option<Rc<dyn Fn()>>,
    pub on_click_at: Option<Rc<dyn Fn(u16, u16)>>,
    pub focus_id: Option<FocusId>,
    pub on_key: Option<KeyHandler>,
    pub on_scroll: Option<Rc<dyn Fn(i16, i16)>>,
}

#[derive(Default, Clone)]
pub struct HitMap {
    pub targets: Vec<HitTarget>,
    pub interactions: InteractionTree,
    pub layouts: HashMap<FocusId, LayoutTarget>,
    pub ancestors: HashMap<FocusId, HashSet<FocusId>>,
}

impl HitMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.targets.clear();
        self.layouts.clear();
        self.interactions = InteractionTree::default();
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

    /// Default Tab order for this interaction snapshot. The App's manager
    /// retains the active member of each composite across frames.
    pub fn focusable_ids(&self) -> Vec<FocusId> {
        let mut manager = FocusManager::new();
        manager.reconcile(self.interactions.clone());
        manager.tab_order()
    }

    pub fn collect_interactions(&mut self, root: &crate::VNode) {
        self.interactions = InteractionTree::from_node(root);
        self.ancestors = self
            .interactions
            .nodes
            .iter()
            .map(|node| (node.id, node.ancestors.iter().copied().collect()))
            .collect();
    }
}

#[derive(Clone)]
pub struct ScrollRegion {
    pub id: FocusId,
    pub x: i32,
    pub y: i32,
    pub width: u16,
    pub height: u16,
    pub max_x: Option<u16>,
    pub max_y: Option<u16>,
}

#[derive(Clone)]
pub struct LayoutTarget {
    pub x: i32,
    pub y: i32,
    pub width: u16,
    pub height: u16,
    pub scrolls: Vec<ScrollRegion>,
}

impl HitMap {
    pub fn finish_interactions(&mut self) {
        for target in &self.targets {
            if let Some(id) = target.focus_id
                && let Some(node) = self.interactions.node_mut(id)
                && node.on_scroll.is_none()
            {
                node.on_scroll = target.on_scroll.clone();
            }
        }
    }

    /// Reveal a focused logical control from its innermost scroll owner out.
    /// Explicitly controlled scroll offsets remain owned by the caller.
    pub fn reveal(&self, id: FocusId) -> bool {
        let Some(target) = self.layouts.get(&id) else {
            return false;
        };
        let (mut x, mut y) = (target.x, target.y);
        let (mut width, mut height) = (i32::from(target.width), i32::from(target.height));
        let mut changed = false;
        for scroll in target.scrolls.iter().rev() {
            let (old_x, old_y) = crate::get_scroll(scroll.id);
            let dx = if x < scroll.x || width > i32::from(scroll.width) {
                x - scroll.x
            } else {
                (x + width - scroll.x - i32::from(scroll.width)).max(0)
            };
            let dy = if y < scroll.y || height > i32::from(scroll.height) {
                y - scroll.y
            } else {
                (y + height - scroll.y - i32::from(scroll.height)).max(0)
            };
            let new_x = scroll.max_x.map_or(old_x, |max| {
                (i32::from(old_x) + dx).clamp(0, i32::from(max)) as u16
            });
            let new_y = scroll.max_y.map_or(old_y, |max| {
                (i32::from(old_y) + dy).clamp(0, i32::from(max)) as u16
            });
            if (new_x, new_y) != (old_x, old_y) {
                crate::set_scroll(scroll.id, (new_x, new_y));
                changed = true;
                x -= i32::from(new_x) - i32::from(old_x);
                y -= i32::from(new_y) - i32::from(old_y);
            }
            let right = (x + width).min(scroll.x + i32::from(scroll.width));
            let bottom = (y + height).min(scroll.y + i32::from(scroll.height));
            x = x.max(scroll.x);
            y = y.max(scroll.y);
            width = (right - x).max(0);
            height = (bottom - y).max(0);
        }
        changed
    }
}
