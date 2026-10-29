use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

use sveltui::prelude::*;
use sveltui::{
    Buffer,
    element::{measure_node, render_node_interactive},
    layout::Rect,
};

struct TrackingAlloc;

static CURRENT_BYTES: AtomicUsize = AtomicUsize::new(0);
static PEAK_BYTES: AtomicUsize = AtomicUsize::new(0);
static ALLOC_COUNT: AtomicUsize = AtomicUsize::new(0);
static DEALLOC_COUNT: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for TrackingAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() {
            let cur = CURRENT_BYTES.fetch_add(layout.size(), Ordering::SeqCst) + layout.size();
            PEAK_BYTES.fetch_max(cur, Ordering::SeqCst);
            ALLOC_COUNT.fetch_add(1, Ordering::SeqCst);
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) };
        CURRENT_BYTES.fetch_sub(layout.size(), Ordering::SeqCst);
        DEALLOC_COUNT.fetch_add(1, Ordering::SeqCst);
    }
}

#[global_allocator]
static GLOBAL: TrackingAlloc = TrackingAlloc;

struct Stats {
    alloc_ops: usize,
    dealloc_ops: usize,
    allocated_bytes: usize,
    peak_bytes: usize,
}

fn measure_block<F: FnOnce() -> R, R>(f: F) -> (R, Stats) {
    let start_alloc = ALLOC_COUNT.load(Ordering::SeqCst);
    let start_dealloc = DEALLOC_COUNT.load(Ordering::SeqCst);
    let start_bytes = CURRENT_BYTES.load(Ordering::SeqCst);
    PEAK_BYTES.store(start_bytes, Ordering::SeqCst);

    let res = f();

    let end_alloc = ALLOC_COUNT.load(Ordering::SeqCst);
    let end_dealloc = DEALLOC_COUNT.load(Ordering::SeqCst);
    let end_bytes = CURRENT_BYTES.load(Ordering::SeqCst);
    let peak = PEAK_BYTES.load(Ordering::SeqCst);

    (
        res,
        Stats {
            alloc_ops: end_alloc - start_alloc,
            dealloc_ops: end_dealloc - start_dealloc,
            allocated_bytes: end_bytes.saturating_sub(start_bytes),
            peak_bytes: peak.saturating_sub(start_bytes),
        },
    )
}

fn main() {
    println!("\n=== Sveltui Memory & Allocation Benchmarks ===\n");

    // 1. Single VNode Tree creation
    let (_, stats) = measure_block(|| {
        view! {
            <FlexBox width=80 height=24 border=BorderStyle::Single>
                <TextSpan>"Baseline Widget Node"</TextSpan>
            </FlexBox>
        }
    });
    println!(
        "[Single VNode View] allocs: {}, deallocs: {}, net_bytes: {} B, peak: {} B",
        stats.alloc_ops, stats.dealloc_ops, stats.allocated_bytes, stats.peak_bytes
    );

    // 2. Large List (1,000 items)
    let (_, stats) = measure_block(|| {
        view! {
            <FlexBox width=80 height=500>
                {#each 0..1000 as i}
                    <TextSpan>{format!("Item {i}")}</TextSpan>
                {/each}
            </FlexBox>
        }
    });
    println!(
        "[1,000 Items #each] allocs: {}, deallocs: {}, net_bytes: {} B, peak: {} B",
        stats.alloc_ops, stats.dealloc_ops, stats.allocated_bytes, stats.peak_bytes
    );

    // 3. Layout Measure (1,000 items)
    let node = view! {
        <FlexBox width=80 height=500>
            {#each 0..1000 as i}
                <TextSpan>{format!("Item {i}")}</TextSpan>
            {/each}
        </FlexBox>
    };
    let (_, stats) = measure_block(|| {
        measure_node(&node, 80, 500);
    });
    println!(
        "[Layout 1,000 Items] allocs: {}, deallocs: {}, net_bytes: {} B, peak: {} B",
        stats.alloc_ops, stats.dealloc_ops, stats.allocated_bytes, stats.peak_bytes
    );

    // 4. Paint to Buffer (80x24)
    let small_node = view! {
        <FlexBox width=80 height=24 border=BorderStyle::Double>
            <TextSpan>"Render Target Test"</TextSpan>
        </FlexBox>
    };
    let mut buffer = Buffer::new(80, 24);
    let mut hit_map = HitMap::new();
    let (_, stats) = measure_block(|| {
        buffer.clear();
        hit_map.clear();
        render_node_interactive(
            &small_node,
            Rect::new(0, 0, 80, 24),
            &mut buffer,
            &mut hit_map,
        );
    });
    println!(
        "[Buffer Paint 80x24] allocs: {}, deallocs: {}, net_bytes: {} B, peak: {} B",
        stats.alloc_ops, stats.dealloc_ops, stats.allocated_bytes, stats.peak_bytes
    );

    // 5. Reactive Signal Writes (10,000 updates)
    let (_, stats) = measure_block(|| {
        let (read, write) = sveltui::reactive::create_signal(0usize);
        sveltui::reactive::batch(|| {
            for _ in 0..10_000 {
                write.update(|v| *v += 1);
            }
        });
        assert_eq!(read.get(), 10_000);
    });
    println!(
        "[10,000 Signal Writes] allocs: {}, deallocs: {}, net_bytes: {} B, peak: {} B",
        stats.alloc_ops, stats.dealloc_ops, stats.allocated_bytes, stats.peak_bytes
    );

    // 6. Component Mount, State, and Teardown (Leak Verification)
    let baseline_memory = CURRENT_BYTES.load(Ordering::SeqCst);
    {
        for _ in 0..500 {
            sveltui::reactive::ComponentScope::run_scoped("TestComp", || {
                let s = sveltui::reactive::state(12345);
                s.set(67890);
                sveltui::reactive::on_destroy(|| {});
            });
        }
    }
    let end_memory = CURRENT_BYTES.load(Ordering::SeqCst);
    println!(
        "[500 Scope Mount/Unmount] baseline: {} B, end: {} B, retained: {} B",
        baseline_memory,
        end_memory,
        end_memory.saturating_sub(baseline_memory)
    );

    println!("\nMemory benchmark completed successfully.\n");
}
