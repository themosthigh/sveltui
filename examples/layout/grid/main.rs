use sveltui::prelude::*;

#[cfg(test)]
mod grid_tests;

#[derive(Props)]
struct PanelProps {
    title: String,
    #[snippet]
    children: Snippet,
}

/// A titled, bordered area of the dashboard.
#[component]
fn Panel(props: PanelProps) -> impl IntoNode {
    view! {
        <View border=BorderStyle::Single border_color=Color::DarkGrey title={props.title} padding={(1, 0)}>
            {@render props.children}
        </View>
    }
}

const SERVICES: [(&str, &str); 6] = [
    ("api", "up"),
    ("worker", "up"),
    ("scheduler", "up"),
    ("search", "degraded"),
    ("mailer", "up"),
    ("billing", "down"),
];

#[component]
fn GridDemo() -> impl IntoNode {
    view! {
        // A header and footer row around a sidebar and a main area. The
        // header and footer span both columns.
        <View display=Display::Grid width={Dimension::Percent(1.0)} height={Dimension::Percent(1.0)}
            grid_template_columns={[length(22), fr(1.0)]}
            grid_template_rows={[length(3), fr(1.0), length(1)]}
            column_gap=1>

            <View grid_column={span(2)} border=BorderStyle::Single border_color=Color::DarkGrey
                padding={(1, 0)} justify_content=JustifyContent::Center>
                <TextSpan bold=true>"Status board"</TextSpan>
            </View>

            <Panel title="Details">
                // Labels in a content-sized column, values in the rest.
                <View display=Display::Grid grid_template_columns={[max_content(), fr(1.0)]} column_gap=2>
                    <TextSpan foreground_color=Color::DarkGrey>"Region"</TextSpan>
                    <TextSpan>"eu-west"</TextSpan>
                    <TextSpan foreground_color=Color::DarkGrey>"Uptime"</TextSpan>
                    <TextSpan>"99.94%"</TextSpan>
                    <TextSpan foreground_color=Color::DarkGrey>"Deploys"</TextSpan>
                    <TextSpan>"12"</TextSpan>
                </View>
            </Panel>

            <Panel title="Services">
                // As many 18-cell columns as fit; cards wrap onto new rows.
                <View display=Display::Grid row_gap=1 column_gap=1
                    grid_template_columns={[repeat(Repeat::AutoFill, [minmax(length(18), fr(1.0))])]}>
                    {SERVICES.iter().map(|(name, status)| view! {
                        <View border=BorderStyle::Single border_color=Color::DarkGrey padding={(1, 0)}>
                            <TextSpan>{*name}</TextSpan>
                            <TextSpan foreground_color=Color::DarkGrey>{*status}</TextSpan>
                        </View>
                    }).collect::<Vec<_>>()}
                </View>
            </Panel>

            <View grid_column={lines(1, -1)} padding={(1, 0)}>
                <TextSpan foreground_color=Color::DarkGrey>"Resize the terminal to reflow the services · Esc: quit"</TextSpan>
            </View>
        </View>
    }
}

fn main() -> std::io::Result<()> {
    App::new(|| view! { <GridDemo /> })
        .on_key(sveltui::app::exit_shortcuts)
        .run()
}
