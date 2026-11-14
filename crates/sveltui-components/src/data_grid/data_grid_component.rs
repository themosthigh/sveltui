use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crossterm::style::Color;
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
use sveltui::element::get_viewport;
use sveltui::prelude::*;

use super::data_grid_types::*;
use crate::controllable::use_controllable;
use crate::table::{CellAlign, TableRules};

/// The grid's cursor: a row (0 is the header, then 1 per body row in the
/// order shown) and a column.
type Cursor = (usize, usize);

/// The order rows are shown in, cached until the rows or the sort change.
#[derive(Clone)]
struct Order {
    version: u64,
    sort: Option<SortState>,
    rows: Rc<[u32]>,
}

/// The next sort when a column's header is pressed: ascending, then
/// descending, then unsorted.
fn next_sort(current: Option<SortState>, column: usize) -> Option<SortState> {
    match current {
        Some(SortState {
            column: c,
            direction: SortDirection::Ascending,
        }) if c == column => Some(SortState {
            column,
            direction: SortDirection::Descending,
        }),
        Some(SortState {
            column: c,
            direction: SortDirection::Descending,
        }) if c == column => None,
        _ => Some(SortState {
            column,
            direction: SortDirection::Ascending,
        }),
    }
}

/// How many cells a track takes when it's sized from content: the header
/// plus its padding (and the sort arrow), so columns never move as rows
/// scroll past.
fn resolve_width(track: &Track, header: &str, padding: u16) -> Track {
    match track {
        Track::Auto | Track::MinContent | Track::MaxContent => {
            length(header.chars().count() as u16 + 2 + padding * 2)
        }
        other => other.clone(),
    }
}

/// A grid of data: a header row, then as many rows as fit, read from
/// `rows` through each column's accessor. Only the rows on screen are
/// built, so it costs the same for ten rows or a million. It's one tab stop:
/// arrows move a cursor between cells, the header sorts, Space and Shift
/// select rows, Enter activates.
#[component]
pub fn DataGrid<T: Clone + 'static>(props: DataGridProps<T>) -> VNode {
    let own_id = use_hook(FocusId::new);
    let id = props.id.unwrap_or(own_id);
    let focused = use_focus(id).is_focused();
    let columns = Rc::new(props.columns);
    let column_count = columns.len();
    let rows = props.rows;
    let len = rows.len();

    let sort = use_controllable(
        props.sort.bound(),
        props.sort.value(),
        || props.default_sort,
        props.on_sort_change.clone(),
    );
    let selected = use_controllable(
        props.selected.bound(),
        props.selected.value(),
        HashSet::new,
        props.on_selected_change.clone(),
    );
    let cursor = state::<Cursor>((usize::from(len > 0), 0));
    let top = state(0usize);
    let anchor = state(None::<usize>);

    // The order rows are shown in. Sorting is the only step that touches
    // every row, so it runs only when the rows or the sort change.
    let cache = use_hook(|| Rc::new(RefCell::new(None::<Order>)));
    let current_sort = sort.get().filter(|s| s.column < column_count);
    let version = rows.version();
    let order = {
        let mut cache = cache.borrow_mut();
        match cache.as_ref() {
            Some(o) if o.version == version && o.sort == current_sort && o.rows.len() == len => {
                o.rows.clone()
            }
            _ => {
                let mut order: Vec<u32> = (0..len as u32).collect();
                if let Some(s) = current_sort
                    && let Some(compare) = &columns[s.column].compare
                {
                    rows.with(|all| {
                        order.sort_by(|&a, &b| {
                            let ordering = compare(&all[a as usize], &all[b as usize]);
                            match s.direction {
                                SortDirection::Ascending => ordering,
                                SortDirection::Descending => ordering.reverse(),
                            }
                        });
                    });
                }
                let rows: Rc<[u32]> = order.into();
                *cache = Some(Order {
                    version,
                    sort: current_sort,
                    rows: rows.clone(),
                });
                rows
            }
        }
    };

    // How many body rows fit, from the space the grid had last frame. On the
    // first frame (or after a resize) we guess, and correct on the next one.
    let rules = props.rules;
    let row_lines = matches!(rules, TableRules::Rows | TableRules::All);
    let header_line = rules != TableRules::None;
    let edge = u16::from(props.border != BorderStyle::None);
    let inner_height = get_viewport(id)
        .map(|(_, h)| h)
        .unwrap_or(match props.height {
            Dimension::Length(cells) => (cells as u16).saturating_sub(edge * 2),
            _ => 10,
        });
    let space = usize::from(inner_height.saturating_sub(1 + u16::from(header_line)));
    let fit = if row_lines { space.div_ceil(2) } else { space }.max(1);
    let shown_fit = state(usize::MAX);
    if shown_fit.get() != fit {
        // A new size needs another frame to settle on what it shows.
        shown_fit.set(fit);
    }

    // Keep the cursor on the grid and its row on screen.
    let (mut row, mut column) = cursor.get();
    row = row.min(len);
    column = column.min(column_count.saturating_sub(1));
    if (row, column) != cursor.get() {
        cursor.set((row, column));
    }
    let mut first = top.get().min(len.saturating_sub(fit));
    if row > 0 {
        let position = row - 1;
        if position < first {
            first = position;
        } else if position >= first + fit {
            first = position + 1 - fit;
        }
    }
    if first != top.get() {
        top.set(first);
    }
    let last = (first + fit).min(len);

    // Copy out just the rows on screen; everything after reads the copies,
    // so no callback runs while the rows are borrowed.
    let window: Vec<(usize, T)> = rows.with(|all| {
        order[first..last]
            .iter()
            .map(|&index| (index as usize, all[index as usize].clone()))
            .collect()
    });

    let padding = u16::from(matches!(rules, TableRules::Columns | TableRules::All));
    let tracks: Vec<Track> = columns
        .iter()
        .map(|c| resolve_width(&c.width, &c.header, padding))
        .collect();
    let chosen = selected.get();
    let mode = props.selection_mode;

    // Selecting by pointer: a click selects the row it lands on.
    let select_only = {
        let selected = selected.clone();
        move |source: usize| selected.set(HashSet::from([source]))
    };

    let mut cells: Vec<VNode> = Vec::with_capacity((window.len() + 2) * column_count);
    for (index, col) in columns.iter().enumerate() {
        let arrow = match current_sort {
            Some(s) if s.column == index => match s.direction {
                SortDirection::Ascending => " ▲",
                SortDirection::Descending => " ▼",
            },
            _ => "",
        };
        let active = focused && cursor.get() == (0, index);
        let sortable = col.is_sortable();
        let sort = sort.clone();
        let on_click = move || {
            cursor.set((0, index));
            if sortable {
                sort.set(next_sort(sort.get(), index));
            }
        };
        let mut semantics = Semantics::new(Role::ColumnHeader, col.header.clone());
        semantics.column = Some(index);
        semantics.row = Some(0);
        let text = view! { <TextSpan bold=true>{format!("{}{arrow}", col.header)}</TextSpan> };
        cells.push(grid_cell(
            id, semantics, col.align, padding, active, false, text, on_click,
        ));
    }
    if matches!(rules, TableRules::Header | TableRules::Columns) {
        cells.push(
            view! { <View separator=true grid_column={lines(1, -1)} height=1 /> }.into_node(),
        );
    }
    for (offset, (source, item)) in window.iter().enumerate() {
        let grid_row = first + offset + 1;
        let is_selected = chosen.contains(source);
        for (index, col) in columns.iter().enumerate() {
            let active = focused && cursor.get() == (grid_row, index);
            let label = (col.text)(item);
            let content = match &col.render {
                Some(render) => render(
                    item,
                    CellState {
                        row: *source,
                        column: index,
                        active,
                        selected: is_selected,
                    },
                ),
                None => view! { <TextSpan>{label.clone()}</TextSpan> }.into_node(),
            };
            let source = *source;
            let select = select_only.clone();
            let on_click = move || {
                cursor.set((grid_row, index));
                anchor.set(Some(grid_row - 1));
                select(source);
            };
            let mut semantics = Semantics::new(Role::Cell, label).selected(is_selected);
            semantics.row = Some(grid_row);
            semantics.column = Some(index);
            cells.push(grid_cell(
                id,
                semantics,
                col.align,
                padding,
                active,
                is_selected,
                content,
                on_click,
            ));
        }
    }

    // The scrollbar sits in the gap between the last column and the border.
    let scrollbar = (len > fit).then(|| {
        let track = fit + if row_lines { fit.saturating_sub(1) } else { 0 };
        let bar = crate::scrollbar::column(track, fit, len, first);
        let header_rows = 1 + u16::from(header_line);
        view! {
            <View position=Position::Absolute inset={Inset::new().top(header_rows).right(0)}>
                <TextSpan foreground_color={props.border_color}>{bar}</TextSpan>
            </View>
        }
    });

    let on_key = {
        let order = order.clone();
        let sortable: Vec<bool> = columns.iter().map(|c| c.is_sortable()).collect();
        let on_activate = props.on_activate.clone();
        let sort = sort.clone();
        let selected = selected.clone();
        move |key: KeyEvent, _ctx: &mut EventContext| -> EventResult {
            let (row, column) = cursor.get();
            let last_column = column_count.saturating_sub(1);
            let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
            let shift = key.modifiers.contains(KeyModifiers::SHIFT);
            let first_row = usize::from(len > 0);
            let next = match key.code {
                KeyCode::Left => (row, column.saturating_sub(1)),
                KeyCode::Right => (row, (column + 1).min(last_column)),
                KeyCode::Up => (row.saturating_sub(1), column),
                KeyCode::Down => ((row + 1).min(len), column),
                KeyCode::Home if ctrl => (first_row, 0),
                KeyCode::End if ctrl => (len, last_column),
                KeyCode::Home => (row, 0),
                KeyCode::End => (row, last_column),
                KeyCode::PageUp => (row.saturating_sub(fit).max(first_row.min(row)), column),
                KeyCode::PageDown => ((row + fit).min(len), column),
                KeyCode::Char(' ') if row > 0 => {
                    let source = order[row - 1] as usize;
                    let mut set = selected.get();
                    if mode == SelectionMode::Single {
                        set = if set.contains(&source) {
                            HashSet::new()
                        } else {
                            HashSet::from([source])
                        };
                    } else if !set.remove(&source) {
                        set.insert(source);
                    }
                    selected.set(set);
                    anchor.set(Some(row - 1));
                    return EventResult::Handled;
                }
                KeyCode::Enter if row == 0 => {
                    if sortable.get(column).copied().unwrap_or(false) {
                        sort.set(next_sort(sort.get(), column));
                    }
                    return EventResult::Handled;
                }
                KeyCode::Enter => {
                    if let Some(activate) = &on_activate {
                        activate(CellRef {
                            row: order[row - 1] as usize,
                            column,
                        });
                    }
                    return EventResult::Handled;
                }
                _ => return EventResult::Ignored,
            };
            // Shift extends the selection from the anchor to the new row.
            if shift && mode == SelectionMode::Multiple && next.0 > 0 && next.0 != row {
                let from = anchor
                    .get()
                    .unwrap_or(row.saturating_sub(1).min(next.0 - 1));
                let to = next.0 - 1;
                let range = from.min(to)..=from.max(to);
                selected.set(range.map(|position| order[position] as usize).collect());
                anchor.set(Some(from));
            }
            cursor.set(next);
            EventResult::Handled
        }
    };
    let on_scroll = move |_dx: i16, dy: i16| {
        let (row, column) = cursor.get();
        let moved = (row as i64 + i64::from(dy) * 3).clamp(i64::from(len > 0), len as i64);
        cursor.set((moved as usize, column));
    };

    let label = props
        .label
        .or_else(|| props.title.clone())
        .unwrap_or_default();
    let mut semantics = Semantics::new(Role::Grid, label);
    semantics.value = Some(len as f64);
    let rule_style = if rules == TableRules::None {
        BorderStyle::None
    } else {
        props.rule_style
    };
    let axes = match rules {
        TableRules::All => RuleAxes::Both,
        TableRules::Rows => RuleAxes::Rows,
        TableRules::Columns => RuleAxes::Columns,
        TableRules::Header | TableRules::None => RuleAxes::None,
    };
    view! {
        <Pressable focus_id=id tab_stop=true semantics=semantics on_key=on_key>
            <View display=Display::Grid grid_template_columns=tracks grid_auto_rows={length(1)}
                height={props.height} border={props.border} title={props.title}
                border_color={if focused { Color::White } else { props.border_color }}
                grid_rules=rule_style grid_rule_axes=axes
                column_gap={if padding == 1 { 1 } else { 2 }}
                padding={Padding::only(0, 1, 0, 0)}
                overflow_y=Overflow::Scroll scrollbar_visibility=ScrollbarVisibility::Never
                scroll_id=id on_scroll=on_scroll>
                {cells}
                {scrollbar}
            </View>
        </Pressable>
    }
}

/// One cell: its meaning for assistive tech, a click that focuses the grid
/// and moves the cursor here, and its content, highlighted when it has the
/// cursor or its row is selected.
#[allow(clippy::too_many_arguments)]
fn grid_cell(
    grid: FocusId,
    semantics: Semantics,
    align: CellAlign,
    padding: u16,
    active: bool,
    selected: bool,
    content: impl IntoNode,
    on_click: impl Fn() + 'static,
) -> VNode {
    let align = match align {
        CellAlign::Start => AlignItems::Start,
        CellAlign::Center => AlignItems::Center,
        CellAlign::End => AlignItems::End,
    };
    let (fg, bg) = match (active, selected) {
        (true, _) => (Some(Color::Black), Some(Color::White)),
        (false, true) => (Some(Color::White), Some(Color::DarkGrey)),
        (false, false) => (None, None),
    };
    view! {
        <Pressable semantics=semantics>
            // Clicks carry the grid's id, so they focus the grid.
            <Pressable focus_id=grid on_click=on_click>
                <View align_items=align overflow_x=Overflow::Hidden padding={(padding, 0)}
                    foreground_color=fg background_color=bg>
                    {content}
                </View>
            </Pressable>
        </Pressable>
    }
}
