use super::*;
use crate::TreeView as Parts;
use crate::clock::TEST_NOW;
use crate::data_grid::GridRows;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::cell::Cell;
use std::collections::HashSet;
use std::rc::Rc;
use std::time::{Duration, Instant};
use sveltui::Runtime;
use sveltui::element::{HitMap, InteractionTree, render_node_interactive, reset_focus_counter};
use sveltui::layout::Rect;
use sveltui::prelude::*;
use sveltui::terminal::Buffer;

const WIDTH: u16 = 40;
const HEIGHT: u16 = 14;

#[derive(Clone)]
struct Node {
    path: &'static str,
    name: &'static str,
    children: Vec<Node>,
}

fn leaf(path: &'static str, name: &'static str) -> Node {
    Node {
        path,
        name,
        children: Vec::new(),
    }
}

fn dir(path: &'static str, name: &'static str, children: Vec<Node>) -> Node {
    Node {
        path,
        name,
        children,
    }
}

/// src/{lib.rs, tree/{mod.rs}}, docs/{guide.md}, README.md
fn files() -> Rc<Vec<Node>> {
    Rc::new(vec![
        dir(
            "src",
            "src",
            vec![
                leaf("src/lib.rs", "lib.rs"),
                dir("src/tree", "tree", vec![leaf("src/tree/mod.rs", "mod.rs")]),
            ],
        ),
        dir("docs", "docs", vec![leaf("docs/guide.md", "guide.md")]),
        leaf("README.md", "README.md"),
    ])
}

/// Marks the row with the cursor with `>`, so tests can read it off the
/// screen.
fn model() -> TreeModel<Node> {
    TreeModel::new(
        |n: &Node| n.path.to_owned(),
        |n: &Node| n.name.to_owned(),
        |n: &Node| n.children.clone(),
    )
    .render(|n: &Node, s: TreeItemState| {
        let mark = if s.active { ">" } else { "" };
        view! { <TextSpan>{format!("{mark}{}", n.name)}</TextSpan> }.into_node()
    })
}

struct Harness {
    focus: FocusManager,
    view: Box<dyn Fn() -> VNode>,
    hits: HitMap,
    tree: InteractionTree,
    screen: String,
}

impl Harness {
    fn new(view: impl Fn() -> VNode + 'static) -> Self {
        TEST_NOW.with(|now| now.set(Some(Instant::now())));
        let mut harness = Self {
            focus: FocusManager::new(),
            view: Box::new(view),
            hits: HitMap::new(),
            tree: InteractionTree::default(),
            screen: String::new(),
        };
        harness.render();
        harness.key(KeyCode::Tab);
        harness
    }

    /// Renders until focus holds still and two frames look the same.
    fn render(&mut self) {
        for _ in 0..10 {
            Runtime::set_dirty(false);
            reset_focus_counter();
            begin_frame();
            let node = self.focus.with_context(|| (self.view)());
            end_frame();
            self.tree = InteractionTree::from_node(&node);
            let mut buffer = Buffer::new(WIDTH, HEIGHT);
            self.hits = HitMap::new();
            let hits = &mut self.hits;
            self.focus.with_context(|| {
                render_node_interactive(&node, Rect::new(0, 0, WIDTH, HEIGHT), &mut buffer, hits)
            });
            let changed = self.focus.reconcile(self.hits.interactions.clone());
            let screen = (0..HEIGHT)
                .map(|y| (0..WIDTH).map(|x| buffer.get(x, y).unwrap().ch).collect())
                .collect::<Vec<String>>()
                .join("\n");
            if !changed && screen == self.screen {
                return;
            }
            self.screen = screen;
        }
        panic!("the tree did not settle");
    }

    fn key(&mut self, code: KeyCode) {
        self.focus.dispatch(KeyEvent::new(code, KeyModifiers::NONE));
        self.render();
    }

    fn keys(&mut self, codes: &[KeyCode]) {
        for &code in codes {
            self.key(code);
        }
    }

    fn type_text(&mut self, text: &str) {
        for ch in text.chars() {
            self.key(KeyCode::Char(ch));
        }
    }

    fn wait(&mut self, by: Duration) {
        TEST_NOW.with(|now| now.set(now.get().map(|t| t + by)));
    }

    /// The label with the cursor.
    fn cursor(&self) -> String {
        let line = self
            .screen
            .lines()
            .find(|l| l.contains('>'))
            .unwrap_or_else(|| panic!("no cursor:\n{}", self.screen));
        let rest = &line[line.find('>').unwrap() + 1..];
        rest.split(|c: char| c.is_whitespace() || c == '│')
            .next()
            .unwrap()
            .to_owned()
    }

    /// The visible rows as (label, level, expanded, selected).
    fn rows(&self) -> Vec<(String, usize, Option<bool>, bool)> {
        self.tree
            .nodes()
            .iter()
            .map(|n| &n.interaction.semantics)
            .filter(|s| s.role == Role::TreeItem)
            .map(|s| {
                (
                    s.label.clone(),
                    s.level.unwrap(),
                    s.expanded,
                    s.selected == Some(true),
                )
            })
            .collect()
    }

    fn labels(&self) -> Vec<String> {
        self.rows().into_iter().map(|r| r.0).collect()
    }

    /// Clicks the first cell of `text` on screen.
    fn click(&mut self, text: &str) {
        let (x, y) = self
            .screen
            .lines()
            .enumerate()
            .find_map(|(y, l)| {
                l.find(text)
                    .map(|b| (l[..b].chars().count() as u16, y as u16))
            })
            .unwrap_or_else(|| panic!("{text:?} not on screen:\n{}", self.screen));
        let target = self.hits.hit_test(x, y).expect("a target").clone();
        self.focus.pointer_down(&target, x, y);
        self.render();
    }
}

fn tree(
    expanded: State<HashSet<String>>,
    selected: State<HashSet<String>>,
    mode: SelectionMode,
    activated: State<Vec<String>>,
) -> impl Fn() -> VNode {
    let roots = files();
    move || {
        view! {
            <TreeView roots={GridRows::Shared(roots.clone())} model={model()} title="Files"
                height={Dimension::Length(f32::from(HEIGHT))} selection_mode=mode
                bind:expanded={expanded} bind:selected={selected}
                on_activate={move |key: String| activated.update(|a| a.push(key))} />
        }
        .into_node()
    }
}

/// A harness on the file tree, with its expanded keys, selected keys and
/// activated keys.
type Simple = (
    Harness,
    State<HashSet<String>>,
    State<HashSet<String>>,
    State<Vec<String>>,
);

fn simple() -> Simple {
    let expanded = state(HashSet::new());
    let selected = state(HashSet::new());
    let activated = state(Vec::new());
    let harness = Harness::new(tree(expanded, selected, SelectionMode::Single, activated));
    (harness, expanded, selected, activated)
}

#[test]
fn arrows_walk_visible_items_and_open_and_close_branches() {
    let (mut t, expanded, _, _) = simple();
    assert_eq!(t.labels(), ["src", "docs", "README.md"]);
    assert_eq!(t.cursor(), "src");

    t.key(KeyCode::Down);
    assert_eq!(t.cursor(), "docs", "closed branches' children are skipped");

    t.key(KeyCode::Up);
    t.key(KeyCode::Right);
    assert_eq!(t.labels(), ["src", "lib.rs", "tree", "docs", "README.md"]);
    assert!(expanded.get().contains("src"));
    assert_eq!(t.cursor(), "src", "the first Right only opens");
    t.key(KeyCode::Right);
    assert_eq!(t.cursor(), "lib.rs", "the second goes in");
    t.key(KeyCode::Right);
    assert_eq!(t.cursor(), "lib.rs", "a leaf has nowhere to go");

    t.key(KeyCode::Left);
    assert_eq!(t.cursor(), "src", "Left goes up to the parent");
    t.key(KeyCode::Left);
    assert_eq!(t.labels(), ["src", "docs", "README.md"], "then closes it");

    t.key(KeyCode::End);
    assert_eq!(t.cursor(), "README.md");
    t.key(KeyCode::Home);
    assert_eq!(t.cursor(), "src");
}

#[test]
fn typing_jumps_to_a_matching_item() {
    let (mut t, _, _, _) = simple();
    t.type_text("r");
    assert_eq!(t.cursor(), "README.md");
    t.wait(Duration::from_secs(1));
    t.type_text("d");
    assert_eq!(t.cursor(), "docs");
    t.wait(Duration::from_secs(1));
    t.type_text("s");
    assert_eq!(t.cursor(), "src", "the search wraps around");

    t.key(KeyCode::Right);
    t.wait(Duration::from_secs(1));
    t.type_text("tr");
    assert_eq!(t.cursor(), "tree", "letters typed together build a prefix");
}

#[test]
fn the_cursor_stays_with_its_item_as_branches_open_and_close() {
    let (mut t, expanded, _, _) = simple();
    t.keys(&[KeyCode::Right, KeyCode::End]);
    assert_eq!(t.cursor(), "README.md");
    expanded.set(HashSet::new());
    t.render();
    assert_eq!(
        t.cursor(),
        "README.md",
        "closing a branch above doesn't move it"
    );

    t.keys(&[KeyCode::Home, KeyCode::Right, KeyCode::Right]);
    assert_eq!(t.cursor(), "lib.rs");
    expanded.set(HashSet::new());
    t.render();
    assert_eq!(t.cursor(), "src", "its branch closing moves it up");
}

#[test]
fn space_selects_and_enter_activates() {
    let (mut t, _, selected, activated) = simple();
    t.key(KeyCode::Char(' '));
    assert_eq!(selected.get(), HashSet::from(["src".to_owned()]));
    t.key(KeyCode::Down);
    t.key(KeyCode::Char(' '));
    assert_eq!(
        selected.get(),
        HashSet::from(["docs".to_owned()]),
        "one at a time"
    );
    t.key(KeyCode::Char(' '));
    assert!(selected.get().is_empty(), "Space again deselects");

    t.key(KeyCode::Enter);
    assert_eq!(activated.get(), ["docs"]);
    assert_eq!(selected.get(), HashSet::from(["docs".to_owned()]));

    let expanded = state(HashSet::new());
    let selected = state(HashSet::new());
    let mut t = Harness::new(tree(
        expanded,
        selected,
        SelectionMode::Multiple,
        state(Vec::new()),
    ));
    t.keys(&[KeyCode::Char(' '), KeyCode::Down, KeyCode::Char(' ')]);
    assert_eq!(selected.get().len(), 2, "any number in Multiple mode");
}

#[test]
fn clicks_select_and_the_chevron_or_a_double_click_toggles() {
    let (mut t, expanded, selected, _) = simple();
    t.click("docs");
    assert_eq!(t.cursor(), "docs");
    assert_eq!(selected.get(), HashSet::from(["docs".to_owned()]));

    // The chevron sits two cells left of the label.
    t.click("▸ src");
    assert!(expanded.get().contains("src"));
    assert_eq!(
        selected.get(),
        HashSet::from(["docs".to_owned()]),
        "no selection change"
    );

    t.wait(Duration::from_secs(1));
    t.click("docs");
    t.wait(Duration::from_millis(200));
    t.click("docs");
    assert!(expanded.get().contains("docs"), "a double click opens");

    t.wait(Duration::from_secs(1));
    t.click("guide.md");
    t.wait(Duration::from_secs(1));
    t.click("guide.md");
    assert!(
        expanded.get().contains("docs"),
        "two slow clicks don't toggle"
    );
}

#[test]
fn rows_say_their_level_and_state() {
    let (mut t, _, _, _) = simple();
    t.keys(&[KeyCode::Right, KeyCode::Down, KeyCode::Char(' ')]);
    assert_eq!(
        t.rows(),
        [
            ("src".to_owned(), 1, Some(true), false),
            ("lib.rs".to_owned(), 2, None, true),
            ("tree".to_owned(), 2, Some(false), false),
            ("docs".to_owned(), 1, Some(false), false),
            ("README.md".to_owned(), 1, None, false),
        ]
    );
    assert!(
        t.tree
            .nodes()
            .iter()
            .any(|n| n.interaction.semantics.role == Role::Tree
                && n.interaction.semantics.label == "Files")
    );
}

#[test]
fn a_huge_tree_builds_only_what_shows_and_reads_only_open_branches() {
    let reads = Rc::new(Cell::new(0usize));
    let counted = reads.clone();
    let roots: Rc<Vec<u32>> = Rc::new((0..100_000).collect());
    let model = TreeModel::new(
        |n: &u32| n.to_string(),
        |n: &u32| format!("item {n}"),
        move |n: &u32| {
            counted.set(counted.get() + 1);
            if *n < 100_000 {
                vec![n + 100_000]
            } else {
                Vec::new()
            }
        },
    )
    .is_branch(|n: &u32| *n < 100_000);
    let mut t = Harness::new(move || {
        view! {
            <TreeView roots={GridRows::Shared(roots.clone())} model={model.clone()}
                height={Dimension::Length(f32::from(HEIGHT))} />
        }
        .into_node()
    });
    assert_eq!(reads.get(), 0, "no branch is open");
    assert_eq!(t.rows().len(), usize::from(HEIGHT) - 2);

    t.key(KeyCode::Right);
    assert_eq!(reads.get(), 1, "opening one branch reads just it");
    t.key(KeyCode::End);
    assert!(t.labels().contains(&"item 99999".to_owned()));
    assert_eq!(t.rows().len(), usize::from(HEIGHT) - 2);
}

#[test]
fn parts_describe_the_same_tree() {
    let selected = state(HashSet::new());
    let mut t = Harness::new(move || {
        view! {
            <Parts::Root title="Files" bind:selected={selected}
                height={Dimension::Length(f32::from(HEIGHT))}>
                <Parts::Branch value="src" label="src">
                    <Parts::Item value="src/lib.rs" label="lib.rs" />
                    <Parts::Branch value="src/tree" label="tree">
                        <Parts::Item value="src/tree/mod.rs" label="mod.rs" />
                    </Parts::Branch>
                </Parts::Branch>
                <Parts::Branch value="docs" label="docs">
                    <Parts::Item value="docs/guide.md" label="guide.md" />
                </Parts::Branch>
                <Parts::Item value="README.md" label="README.md" />
            </Parts::Root>
        }
        .into_node()
    });
    assert_eq!(t.labels(), ["src", "docs", "README.md"]);
    t.keys(&[KeyCode::Right, KeyCode::Down, KeyCode::Right, KeyCode::Down]);
    t.key(KeyCode::Right);
    assert_eq!(
        t.labels(),
        ["src", "lib.rs", "tree", "mod.rs", "docs", "README.md"]
    );
    t.key(KeyCode::Char(' '));
    assert_eq!(selected.get(), HashSet::from(["src/tree".to_owned()]));
}
