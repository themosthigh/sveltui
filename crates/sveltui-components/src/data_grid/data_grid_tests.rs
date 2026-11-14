use super::*;
use crate::{CellAlign, TableRules};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::cell::Cell;
use std::collections::HashSet;
use std::rc::Rc;
use sveltui::Runtime;
use sveltui::element::{HitMap, InteractionTree, render_node_interactive, reset_focus_counter};
use sveltui::layout::Rect;
use sveltui::prelude::*;
use sveltui::terminal::Buffer;

#[derive(Clone)]
struct Person {
    name: String,
    age: u32,
}

fn people(count: usize) -> Rc<Vec<Person>> {
    Rc::new(
        (0..count)
            .map(|i| Person {
                name: format!("person{i:06}"),
                age: (i * 7 % 90) as u32,
            })
            .collect(),
    )
}

fn columns() -> Vec<GridColumn<Person>> {
    vec![
        GridColumn::new("Name", |p: &Person| p.name.clone())
            .width(fr(1.0))
            .sortable(),
        GridColumn::new("Age", |p: &Person| p.age.to_string())
            .width(length(8))
            .align(CellAlign::End)
            .sort_by(|a, b| a.age.cmp(&b.age)),
    ]
}

const GRID: FocusId = FocusId(800_001);

/// Renders until focus and the grid's sizing settle, like `App` does, and
/// returns the screen and the frame's interaction tree.
struct Harness {
    manager: FocusManager,
    view: Box<dyn Fn() -> VNode>,
    hits: HitMap,
    rows: Vec<String>,
}

impl Harness {
    fn new(view: impl Fn() -> VNode + 'static) -> Self {
        let mut harness = Self {
            manager: FocusManager::new(),
            view: Box::new(view),
            hits: HitMap::new(),
            rows: Vec::new(),
        };
        harness.render();
        harness.press(KeyCode::Tab, KeyModifiers::NONE);
        harness
    }

    fn render(&mut self) -> InteractionTree {
        // Tests share one dirty flag, so a neighbour's signal write can
        // dirty a frame of ours; we give it plenty of frames to catch a
        // clean one.
        for _ in 0..500 {
            Runtime::set_dirty(false);
            reset_focus_counter();
            begin_frame();
            let node = self.manager.with_context(|| (self.view)());
            end_frame();
            let tree = InteractionTree::from_node(&node);
            let mut buffer = Buffer::new(40, 12);
            self.hits = HitMap::new();
            let hits = &mut self.hits;
            self.manager.with_context(|| {
                render_node_interactive(&node, Rect::new(0, 0, 40, 12), &mut buffer, hits)
            });
            let changed = self.manager.reconcile(self.hits.interactions.clone());
            if !changed && !Runtime::is_dirty() {
                self.rows = (0..12)
                    .map(|y| (0..40).map(|x| buffer.get(x, y).unwrap().ch).collect())
                    .collect();
                return tree;
            }
        }
        panic!("the grid did not settle");
    }

    fn press(&mut self, code: KeyCode, modifiers: KeyModifiers) -> InteractionTree {
        self.manager.dispatch(KeyEvent::new(code, modifiers));
        self.render()
    }

    fn key(&mut self, code: KeyCode) -> InteractionTree {
        self.press(code, KeyModifiers::NONE)
    }

    fn click(&mut self, x: u16, y: u16) -> InteractionTree {
        let target = self
            .hits
            .hit_test(x, y)
            .expect("something to click")
            .clone();
        self.manager.pointer_down(&target, x, y);
        self.render()
    }

    fn screen(&self) -> String {
        self.rows.join("\n")
    }
}

/// The cells the frame built, as (row, column, label, selected).
fn cells(tree: &InteractionTree) -> Vec<(usize, usize, String, bool)> {
    tree.nodes()
        .iter()
        .filter(|n| n.interaction.semantics.role == Role::Cell)
        .map(|n| {
            let s = &n.interaction.semantics;
            (
                s.row.unwrap(),
                s.column.unwrap(),
                s.label.clone(),
                s.selected == Some(true),
            )
        })
        .collect()
}

fn grid(rows: Rc<Vec<Person>>) -> impl Fn() -> VNode {
    move || {
        view! {
            <DataGrid rows={GridRows::Shared(rows.clone())} columns={columns()} id=GRID
                title="People" rules=TableRules::Columns />
        }
    }
}

#[test]
fn only_the_rows_that_fit_are_built() {
    let mut grid = Harness::new(grid(people(100_000)));
    let tree = grid.render();
    let shown: HashSet<_> = cells(&tree).iter().map(|c| c.0).collect();
    // 12 rows tall: two border rows, the header and its line leave eight.
    assert_eq!(shown.len(), 8);
    assert!(grid.screen().contains("person000000"));
    assert!(
        grid.screen().contains('█'),
        "a scrollbar shows where we are"
    );
}

#[test]
fn arrows_page_and_ctrl_end_move_the_cursor_and_the_window() {
    let mut grid = Harness::new(grid(people(1_000)));
    grid.key(KeyCode::Down);
    grid.key(KeyCode::PageDown);
    assert!(grid.screen().contains("person000009"), "{}", grid.screen());
    let tree = grid.press(KeyCode::End, KeyModifiers::CONTROL);
    assert!(grid.screen().contains("person000999"));
    let rows: Vec<_> = cells(&tree).iter().map(|c| c.0).collect();
    assert_eq!(
        *rows.iter().max().unwrap(),
        1_000,
        "the last row is on screen"
    );
    grid.press(KeyCode::Home, KeyModifiers::CONTROL);
    assert!(grid.screen().contains("person000000"));
}

#[test]
fn enter_on_a_header_cycles_its_sort() {
    let mut grid = Harness::new(grid(people(50)));
    grid.key(KeyCode::Up); // onto the header
    grid.key(KeyCode::Right); // the Age column
    grid.key(KeyCode::Enter);
    assert!(grid.screen().contains("Age ▲"), "{}", grid.screen());
    let tree = grid.render();
    let ages: Vec<u32> = cells(&tree)
        .iter()
        .filter(|c| c.1 == 1)
        .map(|c| c.2.parse().unwrap())
        .collect();
    assert!(ages.windows(2).all(|w| w[0] <= w[1]), "{ages:?}");

    grid.key(KeyCode::Enter);
    assert!(grid.screen().contains("Age ▼"));
    grid.key(KeyCode::Enter);
    assert!(!grid.screen().contains('▲') && !grid.screen().contains('▼'));
}

#[test]
fn a_selection_stays_with_its_rows_across_a_sort() {
    let selected = state(HashSet::<usize>::new());
    let rows = people(30);
    let mut grid = Harness::new(move || {
        view! {
            <DataGrid rows={GridRows::Shared(rows.clone())} columns={columns()} id=GRID
                bind:selected={selected} />
        }
    });
    grid.key(KeyCode::Down);
    grid.key(KeyCode::Char(' ')); // the second row: person000001
    assert_eq!(selected.get(), HashSet::from([1]));

    grid.key(KeyCode::Up);
    grid.key(KeyCode::Up);
    grid.key(KeyCode::Enter); // sort by name, then again for descending
    let tree = grid.key(KeyCode::Enter);
    assert_eq!(
        selected.get(),
        HashSet::from([1]),
        "sorting doesn't change it"
    );
    let chosen: Vec<_> = cells(&tree)
        .into_iter()
        .filter(|c| c.3)
        .map(|c| c.2)
        .collect();
    assert!(
        chosen
            .iter()
            .all(|label| label == "person000001" || label == "7"),
        "{chosen:?}"
    );
}

#[test]
fn shift_extends_the_selection_and_enter_activates() {
    let selected = state(HashSet::<usize>::new());
    let activated = Rc::new(Cell::new(None));
    let seen = activated.clone();
    let rows = people(30);
    let mut grid = Harness::new(move || {
        let seen = seen.clone();
        view! {
            <DataGrid rows={GridRows::Shared(rows.clone())} columns={columns()} id=GRID
                bind:selected={selected} on_activate={move |cell: CellRef| seen.set(Some(cell))} />
        }
    });
    grid.key(KeyCode::Char(' '));
    grid.press(KeyCode::Down, KeyModifiers::SHIFT);
    grid.press(KeyCode::Down, KeyModifiers::SHIFT);
    assert_eq!(selected.get(), HashSet::from([0, 1, 2]));

    grid.key(KeyCode::Right);
    grid.key(KeyCode::Enter);
    assert_eq!(activated.get(), Some(CellRef { row: 2, column: 1 }));
}

#[test]
fn scrolling_does_not_sort_again() {
    let calls = Rc::new(Cell::new(0usize));
    let counter = calls.clone();
    let rows = people(500);
    let mut grid = Harness::new(move || {
        let counter = counter.clone();
        let columns = vec![
            GridColumn::new("Age", |p: &Person| p.age.to_string()).sort_by(
                move |a: &Person, b: &Person| {
                    counter.set(counter.get() + 1);
                    a.age.cmp(&b.age)
                },
            ),
        ];
        view! {
            <DataGrid rows={GridRows::Shared(rows.clone())} columns=columns id=GRID
                default_sort={SortState { column: 0, direction: SortDirection::Ascending }} />
        }
    });
    let after_sort = calls.get();
    assert!(after_sort > 0);
    for _ in 0..20 {
        grid.key(KeyCode::PageDown);
    }
    assert_eq!(
        calls.get(),
        after_sort,
        "moving through the rows reuses the order"
    );
}

#[test]
fn clicking_a_cell_moves_the_cursor_and_selects_its_row_and_a_header_sorts() {
    let selected = state(HashSet::<usize>::new());
    let rows = people(30);
    let mut grid = Harness::new(move || {
        view! {
            <DataGrid rows={GridRows::Shared(rows.clone())} columns={columns()} id=GRID
                bind:selected={selected} />
        }
    });
    // Row 3 of the screen is the second body row (border, header, line).
    let tree = grid.click(3, 4);
    assert_eq!(selected.get(), HashSet::from([1]));
    let active_rows: Vec<_> = cells(&tree)
        .into_iter()
        .filter(|c| c.3)
        .map(|c| c.0)
        .collect();
    assert_eq!(active_rows, [2, 2]);

    // A column, not a byte offset: the border characters take several bytes.
    let line: Vec<char> = grid.rows[1].chars().collect();
    let age_header = line.windows(3).position(|w| w == ['A', 'g', 'e']).unwrap() as u16;
    grid.click(age_header, 1);
    assert!(grid.screen().contains("Age ▲"), "{}", grid.screen());
}
