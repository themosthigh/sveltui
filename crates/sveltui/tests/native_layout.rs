use sveltui::{
    element::{
        ViewElement,
        taffy_bridge::{LayoutEngine, view_to_taffy_style},
    },
    layout::Rect,
    prelude::*,
};

#[test]
fn native_flex_defaults_and_explicit_zero_factors_are_preserved() {
    for direction in [FlexDirection::Row, FlexDirection::Column] {
        let (width, height) = direction.axes(Dimension::Percent(1.0), Dimension::Length(1.0));
        let default = ViewElement::new().width(width).height(height);
        let explicit = default.clone().flex_grow(0.0).flex_shrink(0.0);
        let default_style = view_to_taffy_style(&default);
        let explicit_style = view_to_taffy_style(&explicit);
        assert_eq!(default_style.flex_grow, 0.0);
        assert_eq!(default_style.flex_shrink, 1.0);
        assert_eq!(explicit_style.flex_grow, 0.0);
        assert_eq!(explicit_style.flex_shrink, 0.0);
    }
}

#[test]
fn disabling_native_growth_leaves_remaining_space_for_the_other_sibling() {
    let node = view! {
        <View width=100 height=1 flex_direction=FlexDirection::Row>
            <View min_width=10 flex_basis=0 flex_grow=0.0 />
            <View flex_basis=0 flex_grow=1.0 />
        </View>
    };
    let layout = LayoutEngine::new().compute_layout(&node, 100, 1);
    assert_eq!(layout.children[0].rect.width, 10);
    assert_eq!(layout.children[1].rect.width, 90);
}

#[test]
fn capped_percentages_respect_parent_alignment_in_all_directions() {
    for direction in [
        FlexDirection::Row,
        FlexDirection::RowReverse,
        FlexDirection::Column,
        FlexDirection::ColumnReverse,
    ] {
        for (alignment, cross) in [
            (AlignItems::FlexStart, 0),
            (AlignItems::Center, 30),
            (AlignItems::FlexEnd, 60),
            (AlignItems::Stretch, 0),
        ] {
            let (width, height) = direction.axes(12, 160);
            let (child_w, child_h) =
                direction.axes(Dimension::Length(4.0), Dimension::Percent(1.0));
            let (max_w, max_h) = direction.axes(12, 100);
            let node = view! {
                <View width=width height=height flex_direction=direction align_items=alignment>
                    <View width=child_w height=child_h max_width=max_w max_height=max_h />
                </View>
            };
            let layout = LayoutEngine::new().compute_layout(&node, width, height);
            let rect = layout.children[0].rect;
            assert_eq!(
                direction.axes(rect.x, rect.y).1,
                cross,
                "{direction:?}, {alignment:?}"
            );
            assert_eq!(direction.axes(rect.width, rect.height), (4, 100));
        }
    }
}

#[test]
fn explicit_alignment_does_not_change_native_sizing() {
    let node = view! {
        <View width=160 height=8 align_items=AlignItems::FlexStart>
            <View width={Dimension::Percent(1.0)} max_width=100 height=4 align_self=AlignSelf::Center />
        </View>
    };
    let layout = LayoutEngine::new().compute_layout(&node, 160, 8);
    assert_eq!(layout.children[0].rect, Rect::new(30, 0, 100, 4));
}

#[test]
fn native_auto_stretch_uses_space_inside_padding_borders_margins_and_scrollbars() {
    for cap in [8, 30] {
        let node = view! {
            <View width=20 height=10 border=BorderStyle::Single padding=1
                overflow_y=Overflow::Scroll align_items=AlignItems::Center>
                <View width=Dimension::Auto max_width=cap height=20 flex_shrink=0.0 margin=2 align_self=AlignSelf::Stretch />
            </View>
        };
        let layout = LayoutEngine::new().compute_layout(&node, 20, 10);
        let rect = layout.children[0].rect;
        // 20 - borders/padding (4) - vertical track (1) - margins (4).
        assert_eq!(rect.width, cap.min(11));
        assert_eq!(rect.x, 4);
    }
}

#[test]
fn capped_percentages_resize_and_keep_stretched_children_inside() {
    let node = view! {
        <View width={Dimension::Percent(1.0)} height=8 align_items=AlignItems::Center>
            <View width={Dimension::Percent(1.0)} max_width=100 height=6 border=BorderStyle::Single padding=1>
                <View height=1 />
            </View>
        </View>
    };
    for width in [160, 80, 200, 160] {
        let layout = LayoutEngine::new().compute_layout(&node, width, 8);
        let child = &layout.children[0];
        assert_eq!(child.rect.width, width.min(100));
        assert_eq!(child.rect.x, (width - child.rect.width) / 2);
        assert_eq!(child.children[0].rect.width, child.rect.width - 4);
    }
}
