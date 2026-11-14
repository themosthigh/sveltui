use std::collections::HashSet;
use std::rc::Rc;
use sveltui::prelude::*;
use sveltui_components::*;

#[cfg(test)]
mod data_grid_tests;

#[derive(Clone)]
struct Order {
    id: u32,
    customer: &'static str,
    city: &'static str,
    total: u32,
}

const CUSTOMERS: [&str; 8] = [
    "Ada", "Grace", "Alan", "Edsger", "Barbara", "Ken", "Margaret", "Dennis",
];
const CITIES: [&str; 6] = [
    "London",
    "Arlington",
    "Manchester",
    "Austin",
    "Boston",
    "Murray Hill",
];

/// A hundred thousand orders; the grid only ever builds the ones on screen.
fn orders() -> Rc<Vec<Order>> {
    Rc::new(
        (0..100_000u32)
            .map(|id| Order {
                id: 10_000 + id,
                customer: CUSTOMERS[(id * 7 % 8) as usize],
                city: CITIES[(id * 5 % 6) as usize],
                total: id * 7919 % 50_000,
            })
            .collect(),
    )
}

fn columns() -> Vec<GridColumn<Order>> {
    vec![
        GridColumn::new("Order", |o: &Order| format!("#{}", o.id))
            .width(length(9))
            .sort_by(|a, b| a.id.cmp(&b.id)),
        GridColumn::new("Customer", |o: &Order| o.customer.to_string())
            .width(fr(1.0))
            .sortable(),
        GridColumn::new("City", |o: &Order| o.city.to_string())
            .width(fr(1.0))
            .sortable(),
        GridColumn::new("Total", |o: &Order| {
            format!("{}.{:02}", o.total / 100, o.total % 100)
        })
        .width(length(10))
        .align(CellAlign::End)
        .sort_by(|a, b| a.total.cmp(&b.total))
        .render(|o: &Order, cell: CellState| {
            // Big orders stand out, unless the cell is highlighted anyway.
            let big = o.total >= 45_000 && !cell.active && !cell.selected;
            view! {
                <TextSpan bold=big>{format!("{}.{:02}", o.total / 100, o.total % 100)}</TextSpan>
            }
            .into_node()
        }),
    ]
}

#[component]
fn DataGridDemo() -> impl IntoNode {
    let rows = use_hook(orders);
    let selected = state(HashSet::<usize>::new());
    let sort = state(None::<SortState>);
    let opened = state(None::<u32>);

    let sort_text = match sort.get() {
        Some(s) => format!(
            "sorted by column {} {}",
            s.column + 1,
            if s.direction == SortDirection::Ascending {
                "▲"
            } else {
                "▼"
            }
        ),
        None => "unsorted".to_string(),
    };
    let opened_text = opened.get().map_or_else(
        || "Enter opens an order".to_string(),
        |id| format!("opened #{id}"),
    );
    let open = {
        let rows = rows.clone();
        move |cell: CellRef| opened.set(Some(rows[cell.row].id))
    };

    view! {
        <View width={Dimension::Percent(1.0)} height={Dimension::Percent(1.0)} padding=1 gap=1>
            // The grid fills whatever height the terminal leaves it.
            <View flex_grow=1.0 flex_basis=0>
                <DataGrid rows={GridRows::Shared(rows.clone())} columns={columns()}
                    height={Dimension::Percent(1.0)} title="Orders"
                    bind:selected={selected} bind:sort={sort} on_activate=open />
            </View>
            <TextSpan foreground_color=Color::DarkGrey>
                {format!("{} selected · {sort_text} · {opened_text}", selected.get().len())}
            </TextSpan>
            <TextSpan foreground_color=Color::DarkGrey>
                "Arrows/PgUp/PgDn move · Enter on a header sorts · Space/Shift select · Esc: quit"
            </TextSpan>
        </View>
    }
}

fn main() -> std::io::Result<()> {
    App::new(|| view! { <DataGridDemo /> })
        .on_key(sveltui::app::exit_shortcuts)
        .run()
}
