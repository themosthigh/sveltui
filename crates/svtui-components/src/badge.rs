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
    #[snippet(optional)]
    pub children: Option<Snippet>,
}

#[component]
pub fn Badge(props: BadgeProps) -> impl IntoNode {
    view! {
        <FlexBox
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
            <TextSpan fg=props.fg>{label.clone()}</TextSpan>
        {/if}
        </FlexBox>
    }
}
