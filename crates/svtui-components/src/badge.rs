use crossterm::style::Color;
use svtui::prelude::*;

#[derive(Props)]
pub struct BadgeProps {
    #[prop(default = Color::Grey)]
    pub bg: Color,
    #[prop(default = Color::Black)]
    pub fg: Color,
    #[prop(optional)]
    pub label: Option<String>,
    #[prop(optional)]
    pub children: Option<Snippet>,
}

#[component]
pub fn Badge(props: BadgeProps) -> impl IntoNode {
    view! {
        <Box
            border=BorderStyle::Single
            border_color=props.fg
            bg=props.bg
            fg=props.fg
            align_items=Align::Center
            justify_content=Justify::Center
        >
        {#if props.children.is_some()}
            {@render props.children}
        {:else if let Some(label) = &props.label}
            <Text color=props.fg>{label.clone()}</Text>
        {/if}
        </Box>
    }
}
