use std::time::Duration;
use sveltui_element::*;

fn item(id: usize, label: &str) -> CollectionItem {
    CollectionItem {
        id: FocusId(id),
        label: label.into(),
        ..CollectionItem::default()
    }
}

#[test]
fn logical_order_and_disabled_policy_drive_navigation() {
    let mut disabled = item(2, "disabled");
    disabled.disabled = true;
    let collection = Collection::new([item(1, "Alpha"), disabled, item(3, "Beta")]);
    assert_eq!(
        collection.step(FocusId(1), false, false, false),
        Some(FocusId(3))
    );
    assert_eq!(
        collection.step(FocusId(1), false, false, true),
        Some(FocusId(2))
    );
    assert_eq!(
        collection.step(FocusId(3), false, false, false),
        Some(FocusId(3))
    );
    assert_eq!(
        collection.step(FocusId(3), false, true, false),
        Some(FocusId(1))
    );
    assert_eq!(
        collection.search(Some(FocusId(3)), "al", false),
        Some(FocusId(1))
    );
}

#[test]
fn tree_visibility_and_expand_actions_use_stable_parent_keys() {
    let root = item(1, "Root");
    let mut child = item(2, "Child");
    child.parent = Some(root.id);
    let closed = Collection::new([root.clone(), child.clone()]);
    assert_eq!(closed.visible(false).len(), 1);
    assert_eq!(
        closed.tree_move(root.id, true),
        NavigationAction::Expand(root.id, true)
    );
    let mut root = root;
    root.expanded = true;
    let open = Collection::new([root, child]);
    assert_eq!(open.visible(false).len(), 2);
    assert_eq!(
        open.tree_move(FocusId(1), true),
        NavigationAction::Focus(FocusId(2))
    );
    assert_eq!(
        open.tree_move(FocusId(2), false),
        NavigationAction::Focus(FocusId(1))
    );
}

#[test]
fn grid_stops_at_edges_and_missing_cells() {
    let mut b = item(2, "B");
    b.column = 1;
    let mut c = item(3, "C");
    c.row = 1;
    let grid = Collection::new([item(1, "A"), b, c]);
    assert_eq!(grid.grid_move(FocusId(1), 1, 0), Some(FocusId(2)));
    assert_eq!(grid.grid_move(FocusId(2), 0, 1), Some(FocusId(2)));
    assert_eq!(grid.grid_move(FocusId(1), -1, 0), Some(FocusId(1)));
}

#[test]
fn selection_is_independent_and_reconciles_by_key() {
    let collection = Collection::new([item(1, "A"), item(2, "B"), item(3, "C")]);
    let mut selection = SelectionModel::new(SelectionMode::Multiple);
    selection.toggle(FocusId(1));
    selection.select_range(&collection, FocusId(3));
    assert_eq!(selection.selected().len(), 3);
    let updated = Collection::new([item(3, "C"), item(1, "A")]);
    selection.reconcile(&updated);
    assert!(selection.contains(FocusId(1)));
    assert!(selection.contains(FocusId(3)));
    assert!(!selection.contains(FocusId(2)));
    let mut single = SelectionModel::default();
    single.toggle(FocusId(1));
    single.toggle(FocusId(3));
    assert_eq!(single.selected().len(), 1);
    assert!(single.contains(FocusId(3)));
}

#[test]
fn typeahead_buffers_prefixes_cycles_repeats_and_expires() {
    let mut state = Typeahead::default();
    assert_eq!(state.push('a', Duration::ZERO), "a");
    assert_eq!(state.push('a', Duration::from_millis(100)), "a");
    assert_eq!(state.push('l', Duration::from_millis(200)), "al");
    assert_eq!(state.push('b', Duration::from_millis(1000)), "b");
}
