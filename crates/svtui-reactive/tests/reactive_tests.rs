use svtui_reactive::{Runtime, create_signal};

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
