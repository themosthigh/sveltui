use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use svtui_reactive::{
    ComponentScope, Runtime, batch, create_signal, derived, effect, on_destroy, on_mount,
    provide_context, spawn_blocking, spawn_task, state, use_context,
};

#[test]
fn test_signal_read_write() {
    let (count, set_count) = create_signal(0);
    assert_eq!(count.get(), 0);
    set_count.set(42);
    assert_eq!(count.get(), 42);
    set_count.update(|c| *c += 1);
    assert_eq!(count.get(), 43);
}

#[test]
fn test_signal_copy() {
    let (name, set_name) = create_signal(String::from("Alice"));
    let n2 = name; // Copy!
    let s2 = set_name; // Copy!
    s2.set(String::from("Bob"));
    assert_eq!(n2.get(), "Bob");
}

#[test]
fn test_runtime_dirty_tracking() {
    Runtime::set_dirty(false);
    assert!(!Runtime::is_dirty());

    let (count, set_count) = create_signal("hello");
    let _ = count.get();
    set_count.set("world");
    assert!(Runtime::is_dirty());
}

#[test]
fn test_state_primitive() {
    let count = state(10);
    assert_eq!(count.get(), 10);
    count.set(20);
    assert_eq!(count.get(), 20);
    count.update(|v| *v += 5);
    assert_eq!(count.get(), 25);

    let copy = count;
    copy.set(30);
    assert_eq!(count.get(), 30);
}

#[test]
fn test_derived_state() {
    let quantity = state(2);
    let unit_price = state(25);
    let total = derived(move || quantity.get() * unit_price.get());

    assert_eq!(total.get(), 50);
    quantity.set(3);
    assert_eq!(total.get(), 75);
}

#[test]
fn test_effect_and_batch() {
    let a = state(1);
    let b = state(2);
    let runs = Arc::new(AtomicUsize::new(0));

    let runs_clone = runs.clone();
    effect(move || {
        let _sum = a.get() + b.get();
        runs_clone.fetch_add(1, Ordering::SeqCst);
    });

    assert_eq!(runs.load(Ordering::SeqCst), 1);

    a.set(10);
    assert_eq!(runs.load(Ordering::SeqCst), 2);

    batch(|| {
        a.set(20);
        b.set(30);
    });

    // Batched updates execute effect once after batch completion
    assert_eq!(runs.load(Ordering::SeqCst), 3);
}

#[test]
fn test_lifecycle_hooks() {
    let mounted = Arc::new(AtomicUsize::new(0));
    let cleaned_up = Arc::new(AtomicUsize::new(0));

    let m = mounted.clone();
    let c = cleaned_up.clone();

    let ((), scope) = ComponentScope::run(move || {
        on_mount(move || {
            m.fetch_add(1, Ordering::SeqCst);
            move || {
                c.fetch_add(1, Ordering::SeqCst);
            }
        });

        on_destroy(|| {
            // Additional destroy hook
        });
    });

    assert_eq!(mounted.load(Ordering::SeqCst), 1);
    assert_eq!(cleaned_up.load(Ordering::SeqCst), 0);

    scope.run_cleanups();
    assert_eq!(cleaned_up.load(Ordering::SeqCst), 1);
}

#[test]
fn test_context_api() {
    #[derive(Clone, PartialEq, Debug)]
    struct User(String);

    assert_eq!(use_context::<User>(), None);

    let result = provide_context(User("Ada".into()), || {
        let u1 = use_context::<User>().expect("Context should exist");
        assert_eq!(u1.0, "Ada");

        // Nested override
        provide_context(User("Grace".into()), || {
            let u2 = use_context::<User>().expect("Context should exist");
            assert_eq!(u2.0, "Grace");
        });

        // Restored
        let u3 = use_context::<User>().expect("Context should exist");
        assert_eq!(u3.0, "Ada");
        42
    });

    assert_eq!(result, 42);
    assert_eq!(use_context::<User>(), None);
}

#[test]
fn test_async_task_and_blocking() {
    let status = state(String::from("pending"));

    let task = spawn_task(async move {
        let res = spawn_blocking(|| 100 + 23).await;
        status.set(format!("result: {res}"));
    });

    std::thread::sleep(std::time::Duration::from_millis(50));
    assert_eq!(status.get(), "result: 123");
    task.abort();
    assert!(task.is_aborted());
}
