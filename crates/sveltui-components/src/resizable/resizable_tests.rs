use super::resizable_layout::{Constraints, initial_layout, normalize, resize, toggle_collapse};
use crate::*;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::{cell::RefCell, rc::Rc};
use sveltui::element::*;
use sveltui::layout::Rect;
use sveltui::prelude::*;
use sveltui::terminal::Buffer;

fn free() -> Constraints {
    Constraints::default()
}

fn close(a: &[f32], b: &[f32]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(x, y)| (x - y).abs() < 0.01)
}

#[test]
fn unstated_sizes_share_what_is_left() {
    let sizes = initial_layout(&[Some(20.0), None, None], &[free(); 3]);
    assert!(close(&sizes, &[20.0, 40.0, 40.0]), "{sizes:?}");
}

#[test]
fn sizes_are_scaled_to_a_hundred_and_held_to_their_limits() {
    let c = Constraints {
        max: 30.0,
        ..free()
    };
    let sizes = normalize(&[10.0, 10.0], &[c, free()]);
    assert!(close(&sizes, &[30.0, 70.0]), "{sizes:?}");
    assert!((sizes.iter().sum::<f32>() - 100.0).abs() < 0.01);
}

#[test]
fn resize_moves_space_between_the_neighbours_only() {
    let out = resize(&[25.0, 50.0, 25.0], &[free(); 3], 0, 10.0);
    assert!(close(&out, &[35.0, 40.0, 25.0]), "{out:?}");
}

#[test]
fn resize_stops_at_each_panels_limits() {
    let a = Constraints {
        min: 20.0,
        ..free()
    };
    let b = Constraints {
        max: 60.0,
        ..free()
    };
    // The second panel is already past its maximum, so nothing moves.
    assert!(close(
        &resize(&[30.0, 70.0], &[a, b], 0, -50.0),
        &[30.0, 70.0]
    ));
    // The second panel may not pass 60, so the first can't drop below 40.
    assert!(close(
        &resize(&[50.0, 50.0], &[a, b], 0, -50.0),
        &[40.0, 60.0]
    ));
    assert!(close(
        &resize(&[50.0, 50.0], &[a, b], 0, 50.0),
        &[100.0, 0.0]
    ));
}

#[test]
fn a_collapsible_panel_snaps_to_its_minimum_or_collapses() {
    let c = Constraints {
        min: 20.0,
        collapsible: true,
        ..free()
    };
    let out = resize(&[30.0, 70.0], &[c, free()], 0, -12.0);
    assert!(close(&out, &[20.0, 80.0]), "{out:?}");
    let out = resize(&[30.0, 70.0], &[c, free()], 0, -25.0);
    assert!(close(&out, &[0.0, 100.0]), "{out:?}");
}

#[test]
fn collapse_toggles_and_restores_the_previous_size() {
    let c = Constraints {
        min: 15.0,
        collapsible: true,
        ..free()
    };
    let (collapsed, panel, was) = toggle_collapse(&[30.0, 70.0], &[c, free()], 0, None).unwrap();
    assert_eq!((panel, was), (0, 30.0));
    assert!(close(&collapsed, &[0.0, 100.0]));
    let (restored, _, _) = toggle_collapse(&collapsed, &[c, free()], 0, Some(was)).unwrap();
    assert!(close(&restored, &[30.0, 70.0]), "{restored:?}");
}

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn render(manager: &mut FocusManager, build: &impl Fn() -> VNode) -> HitMap {
    for _ in 0..8 {
        reset_focus_counter();
        begin_frame();
        let node = manager.with_context(build);
        end_frame();
        let mut hits = HitMap::new();
        let mut buffer = Buffer::new(61, 12);
        manager.with_context(|| {
            render_node_interactive(&node, Rect::new(0, 0, 61, 12), &mut buffer, &mut hits)
        });
        if !manager.reconcile(hits.interactions.clone()) {
            return hits;
        }
    }
    panic!("resizable focus did not stabilize");
}

fn width_of(manager: &FocusManager, id: usize) -> u16 {
    manager
        .with_context(|| use_layout(FocusId(id)))
        .unwrap()
        .width
}

/// Panels 1 and 2 with the handle 10 between them, in a 61-cell row:
/// 60 cells for panels, one for the handle.
fn split(layout: State<Vec<f32>>, collapsible: bool) -> impl Fn() -> VNode {
    move || {
        view! {
            <Resizable::PanelGroup id={FocusId(100)} bind:layout=layout>
                <Resizable::Panel id={FocusId(1)} default_size=30.0 min_size=10.0 max_size=80.0
                    collapsible=collapsible>
                    "left"
                </Resizable::Panel>
                <Resizable::Handle id={FocusId(10)} />
                <Resizable::Panel id={FocusId(2)} default_size=70.0>"right"</Resizable::Panel>
            </Resizable::PanelGroup>
        }
    }
}

fn layout_state() -> State<Vec<f32>> {
    begin_frame();
    let layout = state(Vec::<f32>::new());
    end_frame();
    layout
}

#[test]
fn panels_take_their_default_sizes() {
    let build = split(layout_state(), false);
    let mut manager = FocusManager::new();
    render(&mut manager, &build);
    assert_eq!(width_of(&manager, 1), 18);
    assert_eq!(width_of(&manager, 2), 42);
}

#[test]
fn arrows_resize_and_shift_takes_bigger_steps() {
    let layout = layout_state();
    let build = split(layout, false);
    let mut manager = FocusManager::new();
    render(&mut manager, &build);
    manager.focus(FocusId(10));
    manager.dispatch(key(KeyCode::Right));
    render(&mut manager, &build);
    assert!(close(&layout.get(), &[35.0, 65.0]), "{:?}", layout.get());
    manager.dispatch(KeyEvent::new(KeyCode::Left, KeyModifiers::SHIFT));
    render(&mut manager, &build);
    assert!(close(&layout.get(), &[10.0, 90.0]), "{:?}", layout.get());
}

#[test]
fn home_and_end_go_to_the_limits() {
    let layout = layout_state();
    let build = split(layout, false);
    let mut manager = FocusManager::new();
    render(&mut manager, &build);
    manager.focus(FocusId(10));
    manager.dispatch(key(KeyCode::End));
    render(&mut manager, &build);
    assert!(close(&layout.get(), &[80.0, 20.0]), "{:?}", layout.get());
    manager.dispatch(key(KeyCode::Home));
    render(&mut manager, &build);
    assert!(close(&layout.get(), &[10.0, 90.0]), "{:?}", layout.get());
}

#[test]
fn enter_collapses_and_restores_a_collapsible_panel() {
    let layout = layout_state();
    let build = split(layout, true);
    let mut manager = FocusManager::new();
    render(&mut manager, &build);
    manager.focus(FocusId(10));
    manager.dispatch(key(KeyCode::Enter));
    render(&mut manager, &build);
    assert!(close(&layout.get(), &[0.0, 100.0]), "{:?}", layout.get());
    manager.dispatch(key(KeyCode::Enter));
    render(&mut manager, &build);
    assert!(close(&layout.get(), &[30.0, 70.0]), "{:?}", layout.get());
}

#[test]
fn dragging_the_handle_resizes_even_past_the_group() {
    let layout = layout_state();
    let build = split(layout, false);
    let mut manager = FocusManager::new();
    let hits = render(&mut manager, &build);
    let handle = manager.with_context(|| use_layout(FocusId(10))).unwrap();

    manager.pointer_down_at(&hits, handle.x, handle.y);
    let hits = render(&mut manager, &build);
    // The captured handle's shape follows the pointer away from it.
    assert_eq!(manager.pointer_shape(&hits, 0, 0), PointerShape::ColResize);
    // Six cells of the 60 usable: ten percent.
    manager.pointer_drag(&hits, handle.x + 6, handle.y + 4);
    let hits = render(&mut manager, &build);
    assert!(close(&layout.get(), &[40.0, 60.0]), "{:?}", layout.get());

    // Far past the right edge it stops at the first panel's maximum.
    manager.pointer_drag(&hits, 200, 3);
    render(&mut manager, &build);
    assert!(close(&layout.get(), &[80.0, 20.0]), "{:?}", layout.get());
    manager.pointer_up(&hits, 200, 3);
    let hits = render(&mut manager, &build);
    let handle = manager.with_context(|| use_layout(FocusId(10))).unwrap();
    assert_eq!(
        manager.pointer_shape(&hits, handle.x, handle.y),
        PointerShape::ColResize
    );
}

#[test]
fn the_handle_reports_splitter_semantics() {
    let build = split(layout_state(), false);
    let mut manager = FocusManager::new();
    render(&mut manager, &build);
    let node = manager.tree().node(FocusId(10)).unwrap();
    let s = &node.interaction.semantics;
    assert_eq!(s.role, Role::Separator);
    assert_eq!(s.value, Some(30.0));
    assert_eq!((s.value_min, s.value_max), (Some(10.0), Some(80.0)));
    assert_eq!(s.orientation, Some(Orientation::Vertical));
    assert_eq!(s.controls, Some(FocusId(1)));
}

#[test]
fn layout_changes_and_collapses_are_reported() {
    let changes = Rc::new(RefCell::new(Vec::new()));
    let collapsed = Rc::new(RefCell::new(0));
    let build = {
        let changes = changes.clone();
        let collapsed = collapsed.clone();
        move || {
            let changes = changes.clone();
            let collapsed = collapsed.clone();
            let on_collapse: Rc<dyn Fn()> = Rc::new(move || *collapsed.borrow_mut() += 1);
            view! {
                <Resizable::PanelGroup id={FocusId(100)}
                    on_layout_change={move |sizes: Vec<f32>| changes.borrow_mut().push(sizes)}>
                    <Resizable::Panel id={FocusId(1)} default_size=30.0 min_size=10.0
                        collapsible=true on_collapse={on_collapse.clone()}>
                        "left"
                    </Resizable::Panel>
                    <Resizable::Handle id={FocusId(10)} />
                    <Resizable::Panel id={FocusId(2)}>"right"</Resizable::Panel>
                </Resizable::PanelGroup>
            }
        }
    };
    let mut manager = FocusManager::new();
    render(&mut manager, &build);
    manager.focus(FocusId(10));
    manager.dispatch(key(KeyCode::Enter));
    render(&mut manager, &build);
    assert_eq!(*collapsed.borrow(), 1);
    assert_eq!(changes.borrow().len(), 1);
    assert!(close(&changes.borrow()[0], &[0.0, 100.0]));
}

#[test]
fn vertical_groups_resize_with_up_and_down() {
    let layout = layout_state();
    let build = move || {
        view! {
            <Resizable::PanelGroup id={FocusId(100)} direction=Orientation::Vertical bind:layout=layout>
                <Resizable::Panel id={FocusId(1)} default_size=50.0>"top"</Resizable::Panel>
                <Resizable::Handle id={FocusId(10)} />
                <Resizable::Panel id={FocusId(2)} default_size=50.0>"bottom"</Resizable::Panel>
            </Resizable::PanelGroup>
        }
    };
    let mut manager = FocusManager::new();
    render(&mut manager, &build);
    manager.focus(FocusId(10));
    manager.dispatch(key(KeyCode::Left));
    assert!(
        layout.get().is_empty(),
        "left/right don't resize a vertical group"
    );
    manager.dispatch(key(KeyCode::Down));
    render(&mut manager, &build);
    assert!(close(&layout.get(), &[55.0, 45.0]), "{:?}", layout.get());
}

fn screen(build: &impl Fn() -> VNode, focus: Option<usize>) -> Vec<String> {
    let mut manager = FocusManager::new();
    render(&mut manager, build);
    if let Some(id) = focus {
        manager.focus(FocusId(id));
    }
    reset_focus_counter();
    begin_frame();
    let node = manager.with_context(build);
    end_frame();
    let mut hits = HitMap::new();
    let mut buffer = Buffer::new(61, 6);
    manager.with_context(|| {
        render_node_interactive(&node, Rect::new(0, 0, 61, 6), &mut buffer, &mut hits)
    });
    (0..buffer.height)
        .map(|y| {
            (0..buffer.width)
                .map(|x| buffer.get(x, y).unwrap().ch)
                .collect()
        })
        .collect()
}

#[test]
fn a_line_runs_the_height_of_the_group_between_the_panels() {
    let rows = screen(&split(layout_state(), false), None);
    assert!(rows[0].starts_with("left"), "{rows:?}");
    assert!(
        rows.iter().all(|r| r.chars().nth(18) == Some('│')),
        "{rows:?}"
    );
    assert_eq!(
        rows[0].chars().skip(19).take(5).collect::<String>(),
        "right"
    );
    let rows = screen(&split(layout_state(), false), Some(10));
    assert!(
        rows.iter().all(|r| r.chars().nth(18) == Some('║')),
        "{rows:?}"
    );
}
