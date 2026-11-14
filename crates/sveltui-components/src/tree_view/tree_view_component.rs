use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crossterm::style::Color;
use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::rc::Rc;
use std::time::{Duration, Instant};
use sveltui::element::get_viewport;
use sveltui::prelude::*;

use super::tree_view_types::*;
use crate::clock::now;
use crate::controllable::{Controllable, use_controllable};

/// One visible item, in the order shown.
#[derive(Clone)]
pub(crate) struct Row<T> {
    pub(crate) key: String,
    pub(crate) label: String,
    pub(crate) depth: usize,
    pub(crate) branch: bool,
    pub(crate) expanded: bool,
    /// The index of the parent row.
    pub(crate) parent: Option<usize>,
    pub(crate) item: T,
}

/// The visible rows, cached until the roots or the open branches change.
struct Flat<T> {
    version: u64,
    expanded: HashSet<String>,
    rows: Rc<Vec<Row<T>>>,
}

/// Lists the items on show, depth first, going into open branches only.
/// A key seen twice is skipped, so data that loops back on itself can't
/// loop forever.
pub(crate) fn flatten<T: Clone>(
    roots: &[T],
    model: &TreeModel<T>,
    expanded: &HashSet<String>,
) -> Vec<Row<T>> {
    let mut rows = Vec::new();
    let mut seen = HashSet::new();
    let mut stack: Vec<(T, usize, Option<usize>)> =
        roots.iter().rev().map(|r| (r.clone(), 0, None)).collect();
    while let Some((item, depth, parent)) = stack.pop() {
        let key = (model.key)(&item);
        if !seen.insert(key.clone()) {
            continue;
        }
        let open = expanded.contains(&key);
        let children = open.then(|| (model.children)(&item));
        let branch = match (&model.is_branch, &children) {
            (Some(is_branch), _) => is_branch(&item),
            (None, Some(children)) => !children.is_empty(),
            (None, None) => !(model.children)(&item).is_empty(),
        };
        let index = rows.len();
        rows.push(Row {
            label: (model.label)(&item),
            key,
            depth,
            branch,
            expanded: open && branch,
            parent,
            item,
        });
        for child in children.into_iter().flatten().rev() {
            stack.push((child, depth + 1, Some(index)));
        }
    }
    rows
}

/// The keys from the top of the tree down to `index`.
fn path_to<T>(rows: &[Row<T>], index: usize) -> Vec<String> {
    let mut path = Vec::new();
    let mut at = Some(index);
    while let Some(i) = at {
        path.push(rows[i].key.clone());
        at = rows[i].parent;
    }
    path.reverse();
    path
}

/// Where the cursor is: the deepest item on its path that's still on
/// show, so closing a branch moves the cursor up to it.
fn cursor_index<T>(rows: &[Row<T>], path: &[String]) -> usize {
    path.iter()
        .rev()
        .find_map(|key| rows.iter().position(|r| &r.key == key))
        .unwrap_or(0)
}

/// Two clicks on the same row closer together than this toggle it.
const DOUBLE_CLICK: Duration = Duration::from_millis(400);
/// Typing a letter after this long starts a new search.
const TYPEAHEAD_RESET: Duration = Duration::from_millis(700);

/// The next visible row after `from` whose label starts with `prefix`,
/// wrapping around.
fn search<T>(rows: &[Row<T>], from: usize, prefix: &str) -> Option<usize> {
    let prefix = prefix.to_lowercase();
    (1..=rows.len())
        .map(|step| (from + step) % rows.len())
        .find(|&i| rows[i].label.to_lowercase().starts_with(&prefix))
}

fn toggle(set: &Controllable<HashSet<String>>, key: &str, open: bool) {
    let mut keys = set.get();
    if open {
        keys.insert(key.to_owned());
    } else {
        keys.remove(key);
    }
    set.set(keys);
}

/// A tree of items with branches that open and close. Only the rows on
/// screen are built, and children are only read for open branches. It's
/// one tab stop: arrows move a cursor (Right opens and goes in, Left closes
/// and goes up), Space selects, Enter activates, and typing jumps to a
/// matching item.
#[component]
pub fn TreeView<T: Clone + 'static>(props: TreeViewProps<T>) -> VNode {
    let own_id = use_hook(FocusId::new);
    let id = props.id.unwrap_or(own_id);
    let focused = use_focus(id).is_focused();
    let model = props.model;
    let roots = props.roots;

    let expanded = use_controllable(
        props.expanded.bound(),
        props.expanded.value(),
        || props.default_expanded.clone().unwrap_or_default(),
        props.on_expanded_change.clone(),
    );
    let selected = use_controllable(
        props.selected.bound(),
        props.selected.value(),
        HashSet::new,
        props.on_selected_change.clone(),
    );
    let cursor = state(Vec::<String>::new());
    let top = state(0usize);
    let typed = use_hook(|| Rc::new(RefCell::new((String::new(), None::<Instant>))));
    let last_click = use_hook(|| Rc::new(Cell::new(None::<(usize, Instant)>)));

    // Flattening reads every open branch, so it only runs again when the
    // roots or the open branches change.
    let cache = use_hook(|| Rc::new(RefCell::new(None::<Flat<T>>)));
    let open = expanded.get();
    let version = roots.version();
    let rows = {
        let mut cache = cache.borrow_mut();
        match cache.as_ref() {
            Some(flat) if flat.version == version && flat.expanded == open => flat.rows.clone(),
            _ => {
                let rows = Rc::new(roots.with(|all| flatten(all, &model, &open)));
                *cache = Some(Flat {
                    version,
                    expanded: open.clone(),
                    rows: rows.clone(),
                });
                rows
            }
        }
    };
    let len = rows.len();

    // How many rows fit, from the space the tree had last frame.
    let edge = u16::from(props.border != BorderStyle::None);
    let inner_height = get_viewport(id)
        .map(|(_, h)| h)
        .unwrap_or(match props.height {
            Dimension::Length(cells) => (cells as u16).saturating_sub(edge * 2),
            _ => 10,
        });
    let fit = usize::from(inner_height).max(1);
    let shown_fit = state(usize::MAX);
    if shown_fit.get() != fit {
        shown_fit.set(fit);
    }

    // Keep the cursor's row on screen.
    let at = cursor_index(&rows, &cursor.get());
    let mut first = top.get().min(len.saturating_sub(fit));
    if at < first {
        first = at;
    } else if at >= first + fit {
        first = at + 1 - fit;
    }
    if first != top.get() {
        top.set(first);
    }
    let last = (first + fit).min(len);
    let chosen = selected.get();

    let move_to = {
        let rows = rows.clone();
        move |index: usize| {
            if index < rows.len() {
                cursor.set(path_to(&rows, index));
            }
        }
    };
    let select = {
        let selected = selected.clone();
        let mode = props.selection_mode;
        move |key: &str, toggle_off: bool| {
            let mut keys = selected.get();
            let had = keys.contains(key);
            if mode == SelectionMode::Single {
                keys.clear();
            }
            if had && toggle_off {
                keys.remove(key);
            } else {
                keys.insert(key.to_owned());
            }
            selected.set(keys);
        }
    };

    let mut items: Vec<VNode> = Vec::with_capacity(last - first);
    for (index, row) in rows.iter().enumerate().take(last).skip(first) {
        let active = focused && index == at;
        let is_selected = chosen.contains(&row.key);
        let content = match &model.render {
            Some(render) => render(
                &row.item,
                TreeItemState {
                    key: row.key.clone(),
                    depth: row.depth,
                    branch: row.branch,
                    expanded: row.expanded,
                    active,
                    selected: is_selected,
                },
            ),
            None => view! { <TextSpan>{row.label.clone()}</TextSpan> }.into_node(),
        };
        let chevron = match (row.branch, row.expanded) {
            (true, true) => "▾ ",
            (true, false) => "▸ ",
            (false, _) => "  ",
        };
        let on_chevron = {
            let expanded = expanded.clone();
            let key = row.key.clone();
            let (branch, open) = (row.branch, row.expanded);
            let move_to = move_to.clone();
            move || {
                move_to(index);
                if branch {
                    toggle(&expanded, &key, !open);
                }
            }
        };
        let on_row = {
            let expanded = expanded.clone();
            let key = row.key.clone();
            let (branch, open) = (row.branch, row.expanded);
            let move_to = move_to.clone();
            let select = select.clone();
            let last_click = last_click.clone();
            move || {
                move_to(index);
                select(&key, false);
                let time = now();
                let double = last_click
                    .get()
                    .is_some_and(|(i, at)| i == index && time.duration_since(at) < DOUBLE_CLICK);
                last_click.set((!double).then_some((index, time)));
                if double && branch {
                    toggle(&expanded, &key, !open);
                }
            }
        };
        let mut semantics = Semantics::new(Role::TreeItem, row.label.clone())
            .selected(is_selected)
            .level(row.depth + 1);
        if row.branch {
            semantics = semantics.expanded(row.expanded);
        }
        let (fg, bg) = match (active, is_selected) {
            (true, _) => (Some(Color::Black), Some(Color::White)),
            (false, true) => (Some(Color::White), Some(Color::DarkGrey)),
            (false, false) => (None, None),
        };
        let indent = "  ".repeat(row.depth);
        items.push(
            view! {
                <Pressable semantics=semantics>
                    // Clicks carry the tree's id, so they focus the tree.
                    <Pressable focus_id=id on_click=on_row>
                        <View flex_direction=FlexDirection::Row width={Dimension::Percent(1.0)}
                            overflow_x=Overflow::Hidden foreground_color=fg background_color=bg>
                            <TextSpan>{indent}</TextSpan>
                            <Pressable focus_id=id on_click=on_chevron>
                                <TextSpan>{chevron}</TextSpan>
                            </Pressable>
                            {content}
                        </View>
                    </Pressable>
                </Pressable>
            }
            .into_node(),
        );
    }

    let scrollbar = (len > fit).then(|| {
        let bar = crate::scrollbar::column(fit, fit, len, first);
        view! {
            <View position=Position::Absolute inset={Inset::new().top(0).right(0)}>
                <TextSpan foreground_color={props.border_color}>{bar}</TextSpan>
            </View>
        }
    });

    let on_key = {
        let rows = rows.clone();
        let expanded = expanded.clone();
        let select = select.clone();
        let on_activate = props.on_activate.clone();
        move |key: KeyEvent, _: &mut EventContext| -> EventResult {
            if rows.is_empty() {
                return EventResult::Ignored;
            }
            let row = &rows[at];
            let end = rows.len() - 1;
            match key.code {
                KeyCode::Up => move_to(at.saturating_sub(1)),
                KeyCode::Down => move_to((at + 1).min(end)),
                KeyCode::Home => move_to(0),
                KeyCode::End => move_to(end),
                KeyCode::PageUp => move_to(at.saturating_sub(fit)),
                KeyCode::PageDown => move_to((at + fit).min(end)),
                KeyCode::Right if row.branch && !row.expanded => toggle(&expanded, &row.key, true),
                KeyCode::Right if row.expanded => {
                    if rows.get(at + 1).is_some_and(|next| next.parent == Some(at)) {
                        move_to(at + 1);
                    }
                }
                KeyCode::Left if row.expanded => toggle(&expanded, &row.key, false),
                KeyCode::Left => {
                    if let Some(parent) = row.parent {
                        move_to(parent);
                    }
                }
                KeyCode::Char(' ') => select(&row.key, true),
                KeyCode::Enter => {
                    select(&row.key, false);
                    if let Some(activate) = &on_activate {
                        activate(row.key.clone());
                    }
                }
                KeyCode::Char(ch)
                    if !key
                        .modifiers
                        .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
                {
                    let prefix = {
                        let mut typed = typed.borrow_mut();
                        let time = now();
                        if typed
                            .1
                            .is_none_or(|last| time.duration_since(last) > TYPEAHEAD_RESET)
                        {
                            typed.0.clear();
                        }
                        typed.1 = Some(time);
                        typed.0.push(ch);
                        typed.0.clone()
                    };
                    // A run of one letter cycles through the items that
                    // start with it, from the next one on. A longer prefix
                    // may still match the current item, so it looks there
                    // first.
                    let repeat = prefix.chars().all(|c| c == ch);
                    let query = if repeat {
                        &prefix[..ch.len_utf8()]
                    } else {
                        &prefix
                    };
                    let from = if repeat {
                        at
                    } else {
                        (at + rows.len() - 1) % rows.len()
                    };
                    match search(&rows, from, query) {
                        Some(found) => move_to(found),
                        None => return EventResult::Ignored,
                    }
                }
                _ => return EventResult::Ignored,
            }
            EventResult::Handled
        }
    };
    let on_scroll = {
        let rows = rows.clone();
        move |_dx: i16, dy: i16| {
            let target = (at as i64 + i64::from(dy) * 3).clamp(0, rows.len() as i64 - 1);
            if !rows.is_empty() {
                cursor.set(path_to(&rows, target as usize));
            }
        }
    };

    let label = props
        .label
        .or_else(|| props.title.clone())
        .unwrap_or_default();
    view! {
        <Pressable focus_id=id tab_stop=true semantics={Semantics::new(Role::Tree, label)}
            on_key=on_key>
            <View height={props.height} width={props.width} border={props.border} title={props.title}
                border_color={if focused { Color::White } else { props.border_color }}
                padding={Padding::only(0, 1, 0, 0)}
                overflow_y=Overflow::Scroll scrollbar_visibility=ScrollbarVisibility::Never
                scroll_id=id on_scroll=on_scroll>
                {items}
                {scrollbar}
            </View>
        </Pressable>
    }
    .into_node()
}
