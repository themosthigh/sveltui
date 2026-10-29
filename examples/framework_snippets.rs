use svtui::prelude::*;
use svtui_components::*;

#[derive(Clone)]
struct ProjectItem {
    name: String,
    stars: u32,
    language: String,
}

#[derive(Props)]
struct CardGridProps {
    title: String,
    items: Vec<ProjectItem>,
    #[snippet]
    card: Snippet<ProjectItem>,
}

#[component]
fn CardGrid(props: CardGridProps) -> impl IntoNode {
    view! {
        <Box border=BorderStyle::Single border_color=Color::DarkGrey padding=Padding::all(1) gap=1>
            <Text color=Color::White bold=true>{props.title.clone()}</Text>
            <Divider />
            <Box flex_direction=FlexDirection::Row gap=2>
                {#each props.items as item}
                    {@render props.card(item)}
                {/each}
            </Box>
        </Box>
    }
}

#[component]
fn SnippetsDemo() -> impl IntoNode {
    let projects = vec![
        ProjectItem {
            name: String::from("svtui"),
            stars: 1240,
            language: String::from("Rust"),
        },
        ProjectItem {
            name: String::from("crossterm"),
            stars: 3800,
            language: String::from("Rust"),
        },
        ProjectItem {
            name: String::from("ratatui"),
            stars: 6200,
            language: String::from("Rust"),
        },
    ];

    view! {
        <Box width=Size::Fill height=Size::Fill align_items=Align::Center justify_content=Justify::Center>
            <Box width=74 gap=1>
                <CardGrid title="Featured Projects" items=projects>
                    {#snippet card(project: ProjectItem)}
                        <Box
                            width=22
                            border=BorderStyle::Double
                            border_color=Color::DarkGrey
                            padding=Padding::all(1)
                            gap=1
                        >
                            <Text color=Color::White bold=true>{project.name.clone()}</Text>
                            <Text color=Color::DarkGrey>{format!("Stars: *{}", project.stars)}</Text>
                            <Badge bg=Color::Green fg=Color::DarkGreen>{project.language.clone()}</Badge>
                        </Box>
                    {/snippet}
                </CardGrid>
            </Box>
        </Box>
    }
}

fn main() -> std::io::Result<()> {
    App::new(|| view! { <SnippetsDemo /> })
        .on_key(svtui::app::exit_shortcuts)
        .run()
}
