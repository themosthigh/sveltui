//! `grid_rules` draws lines in a grid's gaps that join each other and the
//! grid's border.

use sveltui::element::render_node;
use sveltui::layout::Rect;
use sveltui::prelude::*;
use sveltui::terminal::Buffer;
use sveltui::view;

fn screen(node: impl IntoNode, width: u16, height: u16) -> Vec<String> {
    let mut buffer = Buffer::new(width, height);
    render_node(
        &node.into_node(),
        Rect::new(0, 0, width, height),
        &mut buffer,
    );
    (0..height)
        .map(|y| (0..width).map(|x| buffer.get(x, y).unwrap().ch).collect())
        .collect()
}

#[test]
fn rules_join_each_other_and_the_border() {
    let rows = screen(
        view! {
            <View display=Display::Grid border=BorderStyle::Single grid_rules=BorderStyle::Single
                grid_template_columns={[length(3), length(3)]}>
                <TextSpan>"a"</TextSpan><TextSpan>"b"</TextSpan>
                <TextSpan>"c"</TextSpan><TextSpan>"d"</TextSpan>
            </View>
        },
        9,
        5,
    );
    assert_eq!(
        rows,
        [
            "┌───┬───┐",
            "│a  │b  │",
            "├───┼───┤",
            "│c  │d  │",
            "└───┴───┘",
        ]
    );
}

#[test]
fn a_spanning_cell_breaks_the_line_under_it() {
    let rows = screen(
        view! {
            <View display=Display::Grid border=BorderStyle::Single grid_rules=BorderStyle::Single
                grid_template_columns={[length(3), length(3)]}>
                <View grid_column={span(2)}><TextSpan>"title"</TextSpan></View>
                <TextSpan>"c"</TextSpan><TextSpan>"d"</TextSpan>
            </View>
        },
        9,
        5,
    );
    assert_eq!(
        rows,
        [
            "┌───────┐",
            "│title  │",
            "├───┬───┤",
            "│c  │d  │",
            "└───┴───┘",
        ]
    );
}

#[test]
fn rows_only_and_mixed_weights() {
    let rows = screen(
        view! {
            <View display=Display::Grid border=BorderStyle::Double grid_rules=BorderStyle::Single
                grid_rule_axes=RuleAxes::Rows column_gap=1
                grid_template_columns={[length(3), length(3)]}>
                <TextSpan>"a"</TextSpan><TextSpan>"b"</TextSpan>
                <TextSpan>"c"</TextSpan><TextSpan>"d"</TextSpan>
            </View>
        },
        9,
        5,
    );
    assert_eq!(
        rows,
        [
            "╔═══════╗",
            "║a   b  ║",
            "╟───────╢",
            "║c   d  ║",
            "╚═══════╝",
        ]
    );
}

#[test]
fn without_a_border_the_lines_stop_at_the_edge() {
    let rows = screen(
        view! {
            <View display=Display::Grid grid_rules=BorderStyle::Single
                grid_template_columns={[length(1), length(1)]}>
                <TextSpan>"a"</TextSpan><TextSpan>"b"</TextSpan>
                <TextSpan>"c"</TextSpan><TextSpan>"d"</TextSpan>
            </View>
        },
        3,
        3,
    );
    assert_eq!(rows, ["a│b", "─┼─", "c│d"]);
}

#[test]
fn rules_make_room_for_themselves() {
    let rows = screen(
        view! {
            <View display=Display::Grid grid_rules=BorderStyle::Ascii row_gap=0 column_gap=0
                grid_template_columns={[length(1), length(1)]}>
                <TextSpan>"a"</TextSpan><TextSpan>"b"</TextSpan>
            </View>
        },
        3,
        1,
    );
    assert_eq!(rows, ["a|b"], "a zero gap still gets one cell for the line");
}

#[test]
fn a_separator_draws_one_line_joined_to_the_border() {
    let rows = screen(
        view! {
            <View display=Display::Grid border=BorderStyle::Single grid_rules=BorderStyle::Single
                grid_rule_axes=RuleAxes::None column_gap=1
                grid_template_columns={[length(3), length(3)]}>
                <TextSpan>"a"</TextSpan><TextSpan>"b"</TextSpan>
                <View separator=true grid_column={lines(1, -1)} height=1 />
                <TextSpan>"c"</TextSpan><TextSpan>"d"</TextSpan>
                <TextSpan>"e"</TextSpan><TextSpan>"f"</TextSpan>
            </View>
        },
        9,
        6,
    );
    assert_eq!(
        rows,
        [
            "┌───────┐",
            "│a   b  │",
            "├───────┤",
            "│c   d  │",
            "│e   f  │",
            "└───────┘",
        ]
    );
}

#[test]
fn a_separator_crosses_column_lines() {
    let rows = screen(
        view! {
            <View display=Display::Grid border=BorderStyle::Single grid_rules=BorderStyle::Single
                grid_rule_axes=RuleAxes::Columns grid_template_columns={[length(3), length(3)]}>
                <TextSpan>"a"</TextSpan><TextSpan>"b"</TextSpan>
                <View separator=true grid_column={lines(1, -1)} height=1 />
                <TextSpan>"c"</TextSpan><TextSpan>"d"</TextSpan>
            </View>
        },
        9,
        5,
    );
    assert_eq!(
        rows,
        [
            "┌───┬───┐",
            "│a  │b  │",
            "├───┼───┤",
            "│c  │d  │",
            "└───┴───┘",
        ]
    );
}

#[test]
fn tees_leave_the_title_alone() {
    let top_row = |first: u16, width: u16| {
        screen(
            view! {
                <View display=Display::Grid border=BorderStyle::Single grid_rules=BorderStyle::Single
                    title="Title" grid_template_columns={[length(first), length(3)]}>
                    <TextSpan>"a"</TextSpan><TextSpan>"b"</TextSpan>
                </View>
            },
            width,
            3,
        )[0]
            .clone()
    };
    // The title is cut to fit; the line at x=4 is under it either way.
    assert_eq!(
        top_row(3, 9),
        "┌─ Titl─┐",
        "the line under the title gets no tee"
    );
    assert_eq!(top_row(9, 15), "┌─ Title ─┬───┐", "past the title it does");
}
