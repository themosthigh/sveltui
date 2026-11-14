use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use sveltui::element::taffy_bridge::LayoutEngine;
use sveltui::element::{IntoNode, TextElement, VNode, View, ViewProps};
use sveltui::layout::{AlignItems, Dimension, FlexDirection, Margin, Padding};
use sveltui::view;

fn create_view_tree(n: usize) -> VNode {
    let mut children = Vec::with_capacity(n);
    for i in 0..n {
        let child = View(
            ViewProps::builder()
                .width(Dimension::Length(10.0))
                .height(Dimension::Length(1.0))
                .margin(Margin::all(1))
                .flex_grow(if i % 2 == 0 { 1.0 } else { 0.0 })
                .child(VNode::Text(TextElement::new(format!("Item {}", i))))
                .build(),
        )
        .into_node();
        children.push(child);
    }

    View(
        ViewProps::builder()
            .width(Dimension::Length(120.0))
            .height(Dimension::Length(40.0))
            .flex_direction(FlexDirection::Column)
            .padding(Padding::all(1))
            .children(children)
            .build(),
    )
    .into_node()
}

fn bench_layout_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("layout_scaling");

    // Benchmark layout computation for N children
    for &size in &[10, 100, 1_000, 5_000, 10_000] {
        let tree = create_view_tree(size);

        group.bench_with_input(
            BenchmarkId::new("compute_layout_flat", size),
            &size,
            |b, _| {
                b.iter(|| {
                    let mut engine = LayoutEngine::new();
                    let computed = engine.compute_layout(black_box(&tree), 120, 40);
                    black_box(computed);
                });
            },
        );

        group.bench_with_input(BenchmarkId::new("resize_container", size), &size, |b, _| {
            let mut engine = LayoutEngine::new();
            let mut width = 120;
            b.iter(|| {
                width = if width == 120 { 160 } else { 120 };
                let computed = engine.compute_layout(black_box(&tree), black_box(width), 50);
                black_box(computed);
            });
        });
    }

    group.finish();
}

fn bench_capped_fill_alignment(c: &mut Criterion) {
    let mut group = c.benchmark_group("capped_fill_alignment");
    for (name, alignment) in [
        ("stretch", AlignItems::Stretch),
        ("center", AlignItems::Center),
    ] {
        let tree = view! {
            <View width=160 height=60 align_items=alignment>
                <View width={Dimension::Percent(1.0)} max_width=100 height={Dimension::Percent(1.0)} max_height=50 padding=1>
                    <View width={Dimension::Percent(1.0)} height={Dimension::Percent(1.0)} />
                </View>
            </View>
        };
        group.bench_function(name, |b| {
            b.iter(|| {
                let mut engine = LayoutEngine::new();
                black_box(engine.compute_layout(black_box(&tree), 160, 60));
            });
        });
    }
    group.finish();
}

/// The same cells laid out as a 4-column grid and as wrapped flex rows of
/// fixed-width cells, to keep grid within reach of flex.
fn bench_grid_vs_flex(c: &mut Criterion) {
    use sveltui::prelude::{Display, FlexWrap, fr, repeat};
    let mut group = c.benchmark_group("grid_vs_flex");
    for &cells in &[10, 100, 1_000] {
        let grid = View(
            ViewProps::builder()
                .display(Display::Grid)
                .width(120)
                .grid_template_columns([repeat(4, [fr(1.0)])])
                .children((0..cells).map(|i| TextElement::new(format!("Cell {i}"))))
                .build(),
        )
        .into_node();
        let flex = View(
            ViewProps::builder()
                .width(120)
                .flex_direction(FlexDirection::Row)
                .flex_wrap(FlexWrap::Wrap)
                .children((0..cells).map(|i| {
                    View(
                        ViewProps::builder()
                            .width(30)
                            .child(TextElement::new(format!("Cell {i}")))
                            .build(),
                    )
                }))
                .build(),
        )
        .into_node();
        group.bench_with_input(BenchmarkId::new("grid", cells), &grid, |b, tree| {
            b.iter(|| black_box(LayoutEngine::new().compute_layout(black_box(tree), 120, 40)));
        });
        group.bench_with_input(BenchmarkId::new("flex", cells), &flex, |b, tree| {
            b.iter(|| black_box(LayoutEngine::new().compute_layout(black_box(tree), 120, 40)));
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_layout_scaling,
    bench_capped_fill_alignment,
    bench_grid_vs_flex
);
criterion_main!(benches);
