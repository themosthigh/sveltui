use std::cell::Cell as Counter;
use std::rc::Rc;
use sveltui::prelude::*;

use crate::listbox::text_of;

use super::table_types::*;

/// How many columns a template makes. A counted repeat counts every
/// repetition; an auto-fill one can't be known before layout, so it counts once.
fn column_count(columns: &[Track]) -> usize {
    columns
        .iter()
        .map(|track| match track {
            Track::Repeat(Repeat::Count(n), inner) => usize::from(*n) * inner.len(),
            Track::Repeat(_, inner) => inner.len(),
            _ => 1,
        })
        .sum()
}

fn table() -> Rc<TableState> {
    use_context::<TableContext>()
        .expect("Table parts must be inside Table::Root")
        .0
}

/// A table laid out as one grid. Every cell is placed straight into it, so
/// columns line up across all rows, whatever their content.
#[component]
pub fn Root(props: RootProps) -> VNode {
    let rules = props.rules;
    let state = Rc::new(TableState {
        columns: column_count(&props.columns).max(1),
        rules,
        cell_padding: u16::from(rules.columns()),
        in_header: Counter::new(false),
        row: Counter::new(0),
        column: Counter::new(0),
    });
    let cells = provide_context(TableContext(state.clone()), || props.children.render(()));

    let axes = match rules {
        TableRules::All => RuleAxes::Both,
        TableRules::Rows => RuleAxes::Rows,
        TableRules::Columns => RuleAxes::Columns,
        TableRules::Header | TableRules::None => RuleAxes::None,
    };
    let rule_style = if rules == TableRules::None {
        BorderStyle::None
    } else {
        props.rule_style
    };
    // A column line needs a one-cell gap; without one, columns get room to
    // breathe instead.
    let column_gap = if rules.columns() { 1 } else { props.column_gap };
    let label = props
        .label
        .or_else(|| props.title.clone())
        .unwrap_or_default();

    view! {
        <Pressable semantics={Semantics::new(Role::Table, label)}>
            <View display=Display::Grid grid_template_columns={props.columns}
                border={props.border} border_color={props.border_color} title={props.title}
                grid_rules=rule_style grid_rule_axes=axes column_gap=column_gap>
                {cells}
            </View>
        </Pressable>
    }
}

/// The header rows. Cells inside are column headers, drawn bold, and a line
/// goes under them when the table's rules call for one.
#[component]
pub fn Header(props: HeaderProps) -> VNode {
    let table = table();
    table.in_header.set(true);
    let rows = props.children.render(());
    table.in_header.set(false);
    let line = table.rules.header_line().then(|| {
        view! { <View separator=true grid_column={lines(1, -1)} height=1 /> }
    });
    VNode::Fragment(vec![rows, line.into_node()])
}

/// The body rows.
#[component]
pub fn Body(props: BodyProps) -> VNode {
    props.children.render(())
}

/// One row. A row with fewer cells than the table has columns is filled out
/// with empty ones, so the next row still starts in the first column.
#[component]
pub fn Row(props: RowProps) -> VNode {
    let table = table();
    table.column.set(0);
    let cells = props.children.render(());
    let filled = table.column.get();
    table.row.set(table.row.get() + 1);
    let padding = (filled..table.columns).map(|_| view! { <View /> }.into_node());
    VNode::Fragment(std::iter::once(cells).chain(padding).collect())
}

/// A cell, at the next free column of its row.
fn cell(props: CellProps, header: bool) -> VNode {
    let table = table();
    let column = table.column.get();
    let colspan = props.colspan.max(1);
    table.column.set(column + usize::from(colspan));
    let header = header || table.in_header.get();

    let mut content = props.children.map(|c| c.render(())).unwrap_or(VNode::Empty);
    if header {
        embolden(&mut content);
    }
    let mut semantics = Semantics::new(
        if header {
            Role::ColumnHeader
        } else {
            Role::Cell
        },
        text_of(&content),
    );
    semantics.row = Some(table.row.get());
    semantics.column = Some(column);

    let align = match props.align {
        CellAlign::Start => AlignItems::Start,
        CellAlign::Center => AlignItems::Center,
        CellAlign::End => AlignItems::End,
    };
    view! {
        <Pressable semantics=semantics>
            <View grid_column={span(colspan)} align_items=align overflow_x=Overflow::Hidden
                padding={(table.cell_padding, 0)}>
                {content}
            </View>
        </Pressable>
    }
}

/// Header text is bold.
fn embolden(node: &mut VNode) {
    match node {
        VNode::Text(text) => text.bold = true,
        VNode::View(view) => view.children.iter_mut().for_each(embolden),
        VNode::Fragment(items) => items.iter_mut().for_each(embolden),
        _ => {}
    }
}

#[component]
pub fn Cell(props: CellProps) -> VNode {
    cell(props, false)
}

/// A column header cell. Cells inside `Table::Header` are headers anyway;
/// this one is for a header somewhere else, such as a row's first cell.
#[component]
pub fn HeaderCell(props: CellProps) -> VNode {
    cell(props, true)
}
