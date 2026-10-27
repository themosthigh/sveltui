use svtui_layout::{BorderStyle, Rect, Size, clamp_dimension};

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
