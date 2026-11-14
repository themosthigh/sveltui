use sveltui::prelude::*;
use sveltui_components::*;

#[cfg(test)]
mod table_tests;

struct Package {
    name: &'static str,
    version: &'static str,
    description: &'static str,
    downloads: u32,
}

const PACKAGES: [Package; 5] = [
    Package {
        name: "serde",
        version: "1.0.219",
        description: "Serialization framework",
        downloads: 512_904,
    },
    Package {
        name: "tokio",
        version: "1.45.1",
        description: "Asynchronous runtime",
        downloads: 301_257,
    },
    Package {
        name: "taffy",
        version: "0.13.0",
        description: "Flexbox and grid layout",
        downloads: 8_812,
    },
    Package {
        name: "crossterm",
        version: "0.28.1",
        description: "Terminal manipulation",
        downloads: 97_003,
    },
    Package {
        name: "syn",
        version: "2.0.101",
        description: "Rust source parser",
        downloads: 610_448,
    },
];

const RULES: [(&str, TableRules); 5] = [
    ("Header", TableRules::Header),
    ("All", TableRules::All),
    ("Columns", TableRules::Columns),
    ("Rows", TableRules::Rows),
    ("None", TableRules::None),
];

const BORDERS: [(&str, BorderStyle); 4] = [
    ("Rounded", BorderStyle::Rounded),
    ("Double", BorderStyle::Double),
    ("Thick", BorderStyle::Thick),
    ("None", BorderStyle::None),
];

#[component]
fn TableDemo() -> impl IntoNode {
    let rules = state(1usize);
    let border = state(0usize);
    let (rules_name, table_rules) = RULES[rules.get()];
    let (border_name, table_border) = BORDERS[border.get()];

    view! {
        <View width={Dimension::Percent(1.0)} height={Dimension::Percent(1.0)} padding=1 gap=1>
            <View flex_direction=FlexDirection::Row gap=1>
                <Button on_click={move || rules.update(|r| *r = (*r + 1) % RULES.len())}>
                    {format!("Rules: {rules_name}")}
                </Button>
                <Button on_click={move || border.update(|b| *b = (*b + 1) % BORDERS.len())}>
                    {format!("Border: {border_name}")}
                </Button>
            </View>

            <Table::Root columns={[auto(), auto(), fr(1.0), auto()]} rules=table_rules
                border=table_border title="Popular crates">
                <Table::Header>
                    <Table::Row>
                        <Table::Cell>"Crate"</Table::Cell>
                        <Table::Cell>"Version"</Table::Cell>
                        <Table::Cell>"Description"</Table::Cell>
                        <Table::Cell align=CellAlign::End>"Downloads"</Table::Cell>
                    </Table::Row>
                </Table::Header>
                <Table::Body>
                    {PACKAGES.iter().map(|package| view! {
                        <Table::Row>
                            <Table::Cell>{package.name}</Table::Cell>
                            <Table::Cell>
                                <TextSpan foreground_color=Color::DarkGrey>{package.version}</TextSpan>
                            </Table::Cell>
                            <Table::Cell>{package.description}</Table::Cell>
                            <Table::Cell align=CellAlign::End>{package.downloads.to_string()}</Table::Cell>
                        </Table::Row>
                    }).collect::<Vec<_>>()}
                    <Table::Row>
                        <Table::Cell colspan=3 align=CellAlign::End>
                            <TextSpan foreground_color=Color::DarkGrey>"Total"</TextSpan>
                        </Table::Cell>
                        <Table::Cell align=CellAlign::End>
                            {PACKAGES.iter().map(|p| p.downloads).sum::<u32>().to_string()}
                        </Table::Cell>
                    </Table::Row>
                </Table::Body>
            </Table::Root>

            <TextSpan foreground_color=Color::DarkGrey>"Tab: focus · Enter: change · Esc: quit"</TextSpan>
        </View>
    }
}

fn main() -> std::io::Result<()> {
    App::new(|| view! { <TableDemo /> })
        .on_key(sveltui::app::exit_shortcuts)
        .run()
}
