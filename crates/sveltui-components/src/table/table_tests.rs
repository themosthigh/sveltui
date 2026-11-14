use crate::Table;
use crate::table::{CellAlign, TableRules};
use sveltui::element::{InteractionTree, render_node};
use sveltui::layout::Rect;
use sveltui::prelude::*;
use sveltui::terminal::Buffer;

fn screen(node: VNode, width: u16, height: u16) -> Vec<String> {
    let mut buffer = Buffer::new(width, height);
    render_node(&node, Rect::new(0, 0, width, height), &mut buffer);
    (0..height)
        .map(|y| {
            (0..width)
                .map(|x| buffer.get(x, y).unwrap().ch)
                .collect::<String>()
                .trim_end()
                .to_owned()
        })
        .collect()
}

fn people(rules: TableRules, border: BorderStyle) -> VNode {
    view! {
        <Table::Root columns={[auto(), auto()]} rules=rules border=border>
            <Table::Header>
                <Table::Row>
                    <Table::Cell>"Name"</Table::Cell>
                    <Table::Cell align=CellAlign::End>"Age"</Table::Cell>
                </Table::Row>
            </Table::Header>
            <Table::Body>
                <Table::Row>
                    <Table::Cell>"Ada"</Table::Cell>
                    <Table::Cell align=CellAlign::End>"36"</Table::Cell>
                </Table::Row>
                <Table::Row>
                    <Table::Cell>"Grace"</Table::Cell>
                    <Table::Cell align=CellAlign::End>"85"</Table::Cell>
                </Table::Row>
            </Table::Body>
        </Table::Root>
    }
}

#[test]
fn columns_line_up_with_a_line_under_the_header() {
    let rows = screen(people(TableRules::Header, BorderStyle::None), 20, 4);
    assert_eq!(
        rows,
        ["Name   Age", "──────────", "Ada     36", "Grace   85"]
    );
}

#[test]
fn all_rules_draw_a_full_grid_inside_the_border() {
    let rows = screen(people(TableRules::All, BorderStyle::Rounded), 20, 7);
    assert_eq!(
        rows,
        [
            "╭───────┬─────╮",
            "│ Name  │ Age │",
            "├───────┼─────┤",
            "│ Ada   │  36 │",
            "├───────┼─────┤",
            "│ Grace │  85 │",
            "╰───────┴─────╯",
        ]
    );
}

#[test]
fn column_rules_keep_a_header_line() {
    let rows = screen(people(TableRules::Columns, BorderStyle::Single), 20, 6);
    assert_eq!(
        rows,
        [
            "┌───────┬─────┐",
            "│ Name  │ Age │",
            "├───────┼─────┤",
            "│ Ada   │  36 │",
            "│ Grace │  85 │",
            "└───────┴─────┘",
        ]
    );
}

#[test]
fn no_rules_just_spaces_the_columns() {
    let rows = screen(people(TableRules::None, BorderStyle::None), 20, 3);
    assert_eq!(rows, ["Name   Age", "Ada     36", "Grace   85"]);
}

#[test]
fn short_rows_are_filled_out_and_spans_cover_columns() {
    let node = view! {
        <Table::Root columns={[length(4), length(4), length(4)]} rules=TableRules::None column_gap=1>
            <Table::Body>
                <Table::Row><Table::Cell colspan=2>"wide"</Table::Cell><Table::Cell>"c"</Table::Cell></Table::Row>
                <Table::Row><Table::Cell>"a"</Table::Cell></Table::Row>
                <Table::Row><Table::Cell>"x"</Table::Cell><Table::Cell>"y"</Table::Cell><Table::Cell>"z"</Table::Cell></Table::Row>
            </Table::Body>
        </Table::Root>
    };
    let rows = screen(node, 14, 3);
    assert_eq!(rows, ["wide      c", "a", "x    y    z"]);
}

#[test]
fn cells_say_where_they_are_and_headers_are_bold() {
    let node = people(TableRules::Header, BorderStyle::None);
    let tree = InteractionTree::from_node(&node);
    let roles: Vec<_> = tree
        .nodes()
        .iter()
        .map(|n| {
            let s = &n.interaction.semantics;
            (s.role, s.label.clone(), s.row, s.column)
        })
        .collect();
    assert_eq!(roles[0].0, Role::Table);
    assert_eq!(
        roles[1],
        (Role::ColumnHeader, "Name".into(), Some(0), Some(0))
    );
    assert_eq!(roles[4], (Role::Cell, "36".into(), Some(1), Some(1)));

    let mut buffer = Buffer::new(20, 4);
    render_node(&node, Rect::new(0, 0, 20, 4), &mut buffer);
    assert!(buffer.get(0, 0).unwrap().bold, "header text is bold");
    assert!(!buffer.get(0, 2).unwrap().bold);
}
