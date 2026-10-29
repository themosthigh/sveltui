//! Stable logical collections and selection, independent of rendering.
use crate::element::FocusId;
use std::collections::HashSet;
use std::time::Duration;

#[derive(Clone, Debug, Default)]
pub struct CollectionItem {
    pub id: FocusId,
    pub label: String,
    pub disabled: bool,
    pub hidden: bool,
    pub parent: Option<FocusId>,
    pub expanded: bool,
    pub row: usize,
    pub column: usize,
}

#[derive(Clone, Debug, Default)]
pub struct Collection {
    items: Vec<CollectionItem>,
}

impl Collection {
    pub fn new(items: impl IntoIterator<Item = CollectionItem>) -> Self {
        let items: Vec<_> = items.into_iter().collect();
        let mut keys = HashSet::new();
        for item in &items {
            assert!(
                keys.insert(item.id),
                "duplicate collection key {:?}",
                item.id
            );
        }
        Self { items }
    }
    pub fn items(&self) -> &[CollectionItem] {
        &self.items
    }
    pub fn item(&self, id: FocusId) -> Option<&CollectionItem> {
        self.items.iter().find(|i| i.id == id)
    }
    pub fn visible(&self, disabled_focusable: bool) -> Vec<&CollectionItem> {
        self.items
            .iter()
            .filter(|item| {
                if item.hidden || (item.disabled && !disabled_focusable) {
                    return false;
                }
                let mut parent = item.parent;
                let mut visited = HashSet::new();
                while let Some(id) = parent {
                    if !visited.insert(id) {
                        return false;
                    }
                    let Some(owner) = self.item(id) else {
                        return false;
                    };
                    if owner.hidden || !owner.expanded {
                        return false;
                    }
                    parent = owner.parent;
                }
                true
            })
            .collect()
    }
    pub fn step(
        &self,
        current: FocusId,
        back: bool,
        wrap: bool,
        disabled_focusable: bool,
    ) -> Option<FocusId> {
        let items = self.visible(disabled_focusable);
        if items.is_empty() {
            return None;
        }
        let index = items.iter().position(|i| i.id == current);
        let next = match index {
            Some(0) if back => {
                if wrap {
                    items.len() - 1
                } else {
                    0
                }
            }
            Some(i) if back => i - 1,
            Some(i) if i + 1 < items.len() => i + 1,
            Some(i) => {
                if wrap {
                    0
                } else {
                    i
                }
            }
            None if back => items.len() - 1,
            None => 0,
        };
        Some(items[next].id)
    }
    /// Data-grid movement stops at edges and supports ragged rows.
    pub fn grid_move(&self, current: FocusId, dx: i32, dy: i32) -> Option<FocusId> {
        let source = self.item(current)?;
        let x = source.column as i64 + i64::from(dx);
        let y = source.row as i64 + i64::from(dy);
        self.visible(false)
            .into_iter()
            .find(|i| i.column as i64 == x && i.row as i64 == y)
            .map(|i| i.id)
            .or(Some(current))
    }
    pub fn tree_move(&self, current: FocusId, right: bool) -> NavigationAction {
        let Some(item) = self.item(current) else {
            return NavigationAction::None;
        };
        if right {
            let child = self
                .items
                .iter()
                .find(|i| i.parent == Some(current) && !i.hidden);
            if child.is_some() && !item.expanded {
                NavigationAction::Expand(current, true)
            } else {
                child.map_or(NavigationAction::None, |i| NavigationAction::Focus(i.id))
            }
        } else if item.expanded {
            NavigationAction::Expand(current, false)
        } else {
            item.parent
                .map_or(NavigationAction::None, NavigationAction::Focus)
        }
    }
    pub fn search(
        &self,
        current: Option<FocusId>,
        prefix: &str,
        disabled_focusable: bool,
    ) -> Option<FocusId> {
        let items = self.visible(disabled_focusable);
        let start = current
            .and_then(|id| items.iter().position(|i| i.id == id))
            .map_or(0, |i| i + 1);
        (0..items.len())
            .map(|offset| items[(start + offset) % items.len()])
            .find(|item| {
                item.label
                    .to_lowercase()
                    .starts_with(&prefix.to_lowercase())
            })
            .map(|i| i.id)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NavigationAction {
    None,
    Focus(FocusId),
    Expand(FocusId, bool),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SelectionMode {
    #[default]
    Single,
    Multiple,
}

#[derive(Clone, Debug, Default)]
pub struct SelectionModel {
    pub mode: SelectionMode,
    selected: HashSet<FocusId>,
    anchor: Option<FocusId>,
}

impl SelectionModel {
    pub fn new(mode: SelectionMode) -> Self {
        Self {
            mode,
            ..Self::default()
        }
    }
    pub fn selected(&self) -> &HashSet<FocusId> {
        &self.selected
    }
    pub fn contains(&self, id: FocusId) -> bool {
        self.selected.contains(&id)
    }
    pub fn toggle(&mut self, id: FocusId) {
        if self.mode == SelectionMode::Single {
            self.selected.clear();
            self.selected.insert(id);
        } else if !self.selected.remove(&id) {
            self.selected.insert(id);
        }
        self.anchor = Some(id);
    }
    pub fn select_range(&mut self, collection: &Collection, end: FocusId) {
        if self.mode == SelectionMode::Single {
            self.toggle(end);
            return;
        }
        let visible = collection.visible(false);
        let Some(last) = visible.iter().position(|i| i.id == end) else {
            return;
        };
        let first = self
            .anchor
            .and_then(|id| visible.iter().position(|i| i.id == id))
            .unwrap_or(last);
        self.anchor = Some(visible[first].id);
        self.selected = visible[first.min(last)..=first.max(last)]
            .iter()
            .map(|i| i.id)
            .collect();
    }
    pub fn reconcile(&mut self, collection: &Collection) {
        self.selected.retain(|&id| collection.item(id).is_some());
        if self.anchor.is_some_and(|id| collection.item(id).is_none()) {
            self.anchor = None;
        }
    }
}

#[derive(Default)]
pub struct Typeahead {
    prefix: String,
    last: Option<Duration>,
}
impl Typeahead {
    pub fn push(&mut self, ch: char, time: Duration) -> &str {
        if self
            .last
            .is_none_or(|last| time.saturating_sub(last) > Duration::from_millis(700))
        {
            self.prefix.clear();
        }
        self.last = Some(time);
        if !self.prefix.chars().all(|c| c == ch) || self.prefix.is_empty() {
            self.prefix.push(ch);
        }
        &self.prefix
    }
}
