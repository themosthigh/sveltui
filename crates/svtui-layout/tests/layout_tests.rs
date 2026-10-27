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

#[test]
fn fill_allocations_conserve_space_across_minimums_and_caps() {
    use svtui_layout::{FlexItem, resolve_flex_sizes};
    for available in 0..=24 {
        for min_a in [0, 3, 9] {
            for min_b in [0, 4, 12] {
                for max_a in [2, 8, 20] {
                    for max_b in [1, 7, 18] {
                        let mut items = [
                            FlexItem {
                                size: 5,
                                min: 0,
                                max: u16::MAX,
                                fill: false,
                            },
                            FlexItem {
                                size: 0,
                                min: min_a,
                                max: max_a,
                                fill: true,
                            },
                            FlexItem {
                                size: 0,
                                min: min_b,
                                max: max_b,
                                fill: true,
                            },
                        ];
                        resolve_flex_sizes(&mut items, available, 1);
                        let budget = available.saturating_sub(7);
                        let a = items[1].size;
                        let b = items[2].size;
                        assert_eq!(items[0].size, 5);
                        assert!((min_a..=max_a.max(min_a)).contains(&a));
                        assert!((min_b..=max_b.max(min_b)).contains(&b));
                        if min_a + min_b > budget {
                            assert_eq!(a + b, min_a + min_b);
                        } else {
                            assert!(a + b <= budget);
                            if a + b < budget {
                                assert_eq!(a, max_a.max(min_a));
                                assert_eq!(
                                    b,
                                    max_b.max(min_b),
                                    "available={available}, min={min_a},{min_b}, max={max_a},{max_b}, result={a},{b}"
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}
