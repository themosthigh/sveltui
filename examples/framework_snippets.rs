use sveltui::prelude::*;
use sveltui_components::*;

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
        <FlexBox border=BorderStyle::Single border_color=Color::DarkGrey padding=Padding::all(1) gap=1>
            <TextSpan color=Color::White bold=true>{props.title.clone()}</TextSpan>
            <Divider />
            <FlexBox flex_direction=FlexDirection::Row gap=2>
                {#each props.items as item}
                    {@render props.card(item)}
                {/each}
            </FlexBox>
        </FlexBox>
    }
}

#[component]
fn SnippetsDemo() -> impl IntoNode {
    let projects = vec![
        ProjectItem {
            name: String::from("sveltui"),
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
        <FlexBox width=Size::Fill height=Size::Fill align_items=Align::Center justify_content=Justify::Center>
            <FlexBox width=74 gap=1>
                <CardGrid title="Featured Projects" items=projects>
                    {#snippet card(project: ProjectItem)}
                        <FlexBox
                            width=22
                            border=BorderStyle::Double
                            border_color=Color::DarkGrey
                            padding=Padding::all(1)
                            gap=1
                        >
                            <TextSpan color=Color::White bold=true>{project.name.clone()}</TextSpan>
                            <TextSpan color=Color::DarkGrey>{format!("Stars: *{}", project.stars)}</TextSpan>
                            <Badge bg=Color::Green fg=Color::DarkGreen>{project.language.clone()}</Badge>
                        </FlexBox>
                    {/snippet}
                </CardGrid>
            </FlexBox>
        </FlexBox>
    }
}

fn main() -> std::io::Result<()> {
    App::new(|| view! { <SnippetsDemo /> })
        .on_key(sveltui::app::exit_shortcuts)
        .run()
}
