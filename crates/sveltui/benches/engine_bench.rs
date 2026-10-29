use criterion::{Criterion, black_box, criterion_group, criterion_main};
use sveltui::prelude::*;
use sveltui::{
    Buffer,
    element::{measure_node, render_node_interactive},
    layout::Rect,
};

fn bench_layout_solver(c: &mut Criterion) {
    let mut group = c.benchmark_group("layout_solver");

    // Realistic dashboard layout with mixed flex rows, columns, and fill sizes
    let dashboard = view! {
        <FlexBox width=120 height=40 border=BorderStyle::Single flex_direction=FlexDirection::Row>
            <FlexBox width=25 height=Size::Fill border=BorderStyle::Single>
                <TextSpan>"Sidebar Navigation"</TextSpan>
            </FlexBox>
            <FlexBox width=Size::Fill height=Size::Fill flex_direction=FlexDirection::Column gap=1>
                <FlexBox height=3 border=BorderStyle::Single>
                    <TextSpan>"Header Bar"</TextSpan>
                </FlexBox>
                <FlexBox height=Size::Fill flex_direction=FlexDirection::Row gap=1>
                    <FlexBox width=Size::Fill border=BorderStyle::Single>
                        <TextSpan>"Main Metric Card"</TextSpan>
                    </FlexBox>
                    <FlexBox width=Size::Fill border=BorderStyle::Single>
                        <TextSpan>"Secondary Metric Card"</TextSpan>
                    </FlexBox>
                </FlexBox>
            </FlexBox>
        </FlexBox>
    };

    group.bench_function("measure_complex_dashboard", |b| {
        b.iter(|| {
            black_box(measure_node(black_box(&dashboard), 120, 40));
        });
    });

    // Deeply nested flex containers (20 levels)
    let mut deep_node = view! { <TextSpan>"Leaf Node"</TextSpan> };
    for _ in 0..20 {
        deep_node = view! {
            <FlexBox width=Size::Fill height=Size::Fill padding=1 border=BorderStyle::Single>
                {deep_node}
            </FlexBox>
        };
    }

    group.bench_function("measure_deeply_nested_20_levels", |b| {
        b.iter(|| {
            black_box(measure_node(black_box(&deep_node), 100, 50));
        });
    });

    group.finish();
}

fn bench_buffer_paint(c: &mut Criterion) {
    let mut group = c.benchmark_group("buffer_paint");
    let mut buffer = Buffer::new(120, 40);
    let mut hit_map = HitMap::new();
    let rect = Rect::new(0, 0, 120, 40);

    let list_view = view! {
        <FlexBox width=120 height=40 border=BorderStyle::Double>
            {#each 0..30 as i}
                <TextSpan>{format!("Row {i}: performance and reactive throughput in terminal cells")}</TextSpan>
            {/each}
        </FlexBox>
    };

    group.bench_function("render_30_row_flexbox_interactive", |b| {
        b.iter(|| {
            buffer.clear();
            hit_map.clear();
            render_node_interactive(
                black_box(&list_view),
                black_box(rect),
                black_box(&mut buffer),
                black_box(&mut hit_map),
            );
        });
    });

    group.finish();
}

fn bench_terminal_diff(c: &mut Criterion) {
    let mut group = c.benchmark_group("terminal_diff");

    let prev = Buffer::new(160, 50);
    let curr_noop = Buffer::new(160, 50);
    let mut curr_partial = Buffer::new(160, 50);
    let mut curr_full = Buffer::new(160, 50);

    // Partial diff: ~5% modified cells
    curr_partial.set_string(10, 5, "STATUS: CONNECTED (0.42ms)", None, None, false, 40);
    curr_partial.set_string(10, 6, "METRIC: 99.98% SUCCESS", None, None, false, 40);

    // Full diff: 100% modified cells
    for y in 0..50 {
        curr_full.set_string(0, y, &"#".repeat(160), None, None, false, 160);
    }

    group.bench_function("diff_0_percent_noop", |b| {
        let mut out = Vec::with_capacity(4096);
        b.iter(|| {
            out.clear();
            black_box(sveltui::terminal::write_buffer_diff(
                black_box(&mut out),
                black_box(&curr_noop),
                black_box(&prev),
                false,
            ))
            .unwrap();
        });
    });

    group.bench_function("diff_5_percent_partial", |b| {
        let mut out = Vec::with_capacity(4096);
        b.iter(|| {
            out.clear();
            black_box(sveltui::terminal::write_buffer_diff(
                black_box(&mut out),
                black_box(&curr_partial),
                black_box(&prev),
                false,
            ))
            .unwrap();
        });
    });

    group.bench_function("diff_100_percent_full_damage", |b| {
        let mut out = Vec::with_capacity(65536);
        b.iter(|| {
            out.clear();
            black_box(sveltui::terminal::write_buffer_diff(
                black_box(&mut out),
                black_box(&curr_full),
                black_box(&prev),
                false,
            ))
            .unwrap();
        });
    });

    group.finish();
}

fn bench_reactivity_and_frame_loop(c: &mut Criterion) {
    let mut group = c.benchmark_group("reactivity");

    group.bench_function("signal_write_and_batch_1k", |b| {
        b.iter(|| {
            let (read, write) = sveltui::reactive::create_signal(0usize);
            sveltui::reactive::batch(|| {
                for _ in 0..1000 {
                    write.update(|v| *v += 1);
                }
            });
            black_box(read.get());
        });
    });

    group.bench_function("derived_signal_chain_evaluation", |b| {
        b.iter(|| {
            let (base, set_base) = sveltui::reactive::create_signal(10i32);
            let b_sig = base;
            let d1 = sveltui::reactive::derived(move || b_sig.get() * 2);
            let d1_clone = d1.clone();
            let d2 = sveltui::reactive::derived(move || d1_clone.get() + 5);

            set_base.set(20);
            black_box(d2.get());
        });
    });

    // End-to-End Reactive Frame Loop
    group.bench_function("end_to_end_reactive_frame", |b| {
        let (count, set_count) = sveltui::reactive::create_signal(0usize);
        let mut buffer = Buffer::new(80, 24);
        let mut prev_buffer = Buffer::new(80, 24);
        let mut hit_map = HitMap::new();
        let screen_rect = Rect::new(0, 0, 80, 24);
        let mut out = Vec::with_capacity(4096);

        b.iter(|| {
            set_count.update(|c| *c += 1);
            sveltui::element::reset_focus_counter();
            sveltui::reactive::begin_frame();
            let root = view! {
                <FlexBox width=80 height=24 border=BorderStyle::Single>
                    <TextSpan>{format!("Frame Count: {}", count.get())}</TextSpan>
                </FlexBox>
            };
            sveltui::reactive::end_frame();

            buffer.clear();
            hit_map.clear();
            render_node_interactive(
                black_box(&root),
                black_box(screen_rect),
                black_box(&mut buffer),
                black_box(&mut hit_map),
            );

            out.clear();
            black_box(sveltui::terminal::write_buffer_diff(
                black_box(&mut out),
                black_box(&buffer),
                black_box(&prev_buffer),
                false,
            ))
            .unwrap();
            std::mem::swap(&mut buffer, &mut prev_buffer);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_layout_solver,
    bench_buffer_paint,
    bench_terminal_diff,
    bench_reactivity_and_frame_loop
);
criterion_main!(benches);
