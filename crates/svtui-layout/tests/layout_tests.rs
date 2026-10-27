use svtui_layout::{BorderStyle, Placement, Rect, Size, clamp_dimension, compute_popover_rect};

#[test]
fn test_rect_inner() {
    let r = Rect::new(5, 5, 20, 10);
    let inner = r.inner(1, 2, 3, 4);
    assert_eq!(inner.x, 9);
    assert_eq!(inner.y, 6);
    assert_eq!(inner.width, 14);
    assert_eq!(inner.height, 6);
}

#[test]
fn test_border_chars() {
    let rounded = BorderStyle::Rounded.chars().unwrap();
    assert_eq!(rounded.top_left, '╭');
    assert_eq!(rounded.top_right, '╮');
    assert_eq!(rounded.bottom_left, '╰');
    assert_eq!(rounded.bottom_right, '╯');
    assert_eq!(rounded.horizontal, '─');
    assert_eq!(rounded.vertical, '│');
}

#[test]
fn test_rect_intersect() {
    let a = Rect::new(0, 0, 10, 10);
    let b = Rect::new(5, 5, 10, 10);
    let inter = a.intersect(&b).unwrap();
    assert_eq!(inter, Rect::new(5, 5, 5, 5));

    let c = Rect::new(20, 20, 5, 5);
    assert_eq!(a.intersect(&c), None);
}

#[test]
fn test_clamp_dimension() {
    // Val=10, min=15, max=25 => 15
    assert_eq!(
        clamp_dimension(10, Some(Size::Fixed(15)), Some(Size::Fixed(25)), 100),
        15
    );
    // Val=30, min=15, max=25 => 25
    assert_eq!(
        clamp_dimension(30, Some(Size::Fixed(15)), Some(Size::Fixed(25)), 100),
        25
    );
    // Val=20, min=15, max=25 => 20
    assert_eq!(
        clamp_dimension(20, Some(Size::Fixed(15)), Some(Size::Fixed(25)), 100),
        20
    );
}

#[test]
fn test_compute_popover_rect_center() {
    let screen = Rect::new(0, 0, 80, 24);
    let rect = compute_popover_rect(Placement::Center, (20, 10), None, screen);
    assert_eq!(rect, Rect::new(30, 7, 20, 10));
}

#[test]
fn test_compute_popover_rect_anchored_bottom_start() {
    let screen = Rect::new(0, 0, 80, 24);
    let anchor = Rect::new(10, 5, 12, 3);
    let rect = compute_popover_rect(Placement::BottomStart, (20, 6), Some(anchor), screen);
    assert_eq!(rect, Rect::new(10, 8, 20, 6));
}

#[test]
fn test_compute_popover_rect_auto_flip() {
    let screen = Rect::new(0, 0, 80, 24);
    // Anchor placed near bottom edge: y=20, h=2 (ends at y=22)
    let anchor = Rect::new(10, 20, 12, 2);
    // With height=8, bottom would need y=22..30 which exceeds screen height 24.
    // Space below is 2, space above is 20, so it auto-flips to Top!
    let rect = compute_popover_rect(Placement::BottomStart, (20, 8), Some(anchor), screen);
    assert_eq!(rect, Rect::new(10, 12, 20, 8));
}

#[test]
fn test_compute_popover_rect_clamping() {
    let screen = Rect::new(0, 0, 80, 24);
    // Explicit anchor coordinate at edge
    let rect = compute_popover_rect(Placement::Anchor { x: 75, y: 22 }, (20, 10), None, screen);
    assert_eq!(rect, Rect::new(60, 14, 20, 10));
}
