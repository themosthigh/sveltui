//! Baseline for the cost of one dirty frame. Every dirty frame re-runs the
//! whole view, so typing in one field pays for every row on screen. Run it
//! with `cargo test --release -p sveltui-components --test frame_cost --
//! --ignored --nocapture` and read the numbers off the output.

use std::time::Instant;
use sveltui::{
    Buffer,
    element::{render_node_interactive, reset_focus_counter},
    layout::Rect,
    prelude::*,
};
use sveltui_components::*;

thread_local! {
    static ROWS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[component]
fn Screen() -> impl IntoNode {
    let rows = ROWS.with(|r| r.get());
    let query = state(String::new());
    let items: Vec<usize> = (0..rows).collect();
    view! {
        <View>
            <TextInput bind:value=query placeholder="Type..." />
            <View overflow_y=Overflow::Scroll height=20>
                {#each items as n (n)}
                    <Button>{format!("Row {n}")}</Button>
                {/each}
            </View>
        </View>
    }
}

fn frame(manager: &mut FocusManager, rows: usize) -> (u128, u128) {
    ROWS.with(|r| r.set(rows));
    reset_focus_counter();
    let started = Instant::now();
    begin_frame();
    let tree = manager.with_context(|| Screen(ScreenProps::builder().build()).into_node());
    end_frame();
    let built = started.elapsed().as_micros();
    let mut buffer = Buffer::new(80, 24);
    let mut hits = HitMap::new();
    let started = Instant::now();
    manager.with_context(|| {
        render_node_interactive(&tree, Rect::new(0, 0, 80, 24), &mut buffer, &mut hits)
    });
    (built, started.elapsed().as_micros())
}

#[test]
#[ignore = "measurement, not a check"]
fn per_frame_cost_by_list_size() {
    for rows in [10, 200, 2000] {
        let mut manager = FocusManager::new();
        frame(&mut manager, rows);
        let runs = 10;
        let (mut built, mut painted) = (0, 0);
        for _ in 0..runs {
            let (b, p) = frame(&mut manager, rows);
            built += b;
            painted += p;
        }
        println!(
            "rows={rows:5}  build={:7}us  layout+paint={:7}us per frame",
            built / runs,
            painted / runs
        );
    }
}

#[test]
#[ignore = "measurement, not a check"]
fn where_the_frame_goes() {
    use sveltui::element::taffy_bridge::layout;
    for rows in [200, 2000] {
        let manager = FocusManager::new();
        ROWS.with(|r| r.set(rows));
        begin_frame();
        let tree = manager.with_context(|| Screen(ScreenProps::builder().build()).into_node());
        end_frame();
        let runs = 10u128;
        let (mut lay, mut col, mut full) = (0, 0, 0);
        for _ in 0..runs {
            let t = Instant::now();
            let _ = layout(&tree, 80, 24);
            lay += t.elapsed().as_micros();
            let t = Instant::now();
            HitMap::new().collect_interactions(&tree);
            col += t.elapsed().as_micros();
            let mut buffer = Buffer::new(80, 24);
            let mut hits = HitMap::new();
            let t = Instant::now();
            render_node_interactive(&tree, Rect::new(0, 0, 80, 24), &mut buffer, &mut hits);
            full += t.elapsed().as_micros();
        }
        println!(
            "rows={rows:5}  layout={}us  collect={}us  render_total={}us",
            lay / runs,
            col / runs,
            full / runs
        );
    }
}
