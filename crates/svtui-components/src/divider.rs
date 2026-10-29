use crossterm::style::Color;
use svtui::prelude::*;

#[derive(Props)]
pub struct DividerProps {
    #[prop(optional)]
    pub title: Option<String>,
    #[prop(optional)]
    pub color: Option<Color>,
    #[prop(default = Size::Fill)]
    pub width: Size,
    #[prop(optional)]
    pub min_width: Option<Size>,
    #[prop(optional)]
    pub max_width: Option<Size>,
    #[snippet(optional)]
    pub children: Option<Snippet>,
}

#[component]
pub fn Divider(props: DividerProps) -> impl IntoNode {
    let color = props.color.unwrap_or(Color::DarkGrey);
    let title = props.title.unwrap_or_default();
    let text = if !title.is_empty() {
        format!("─── {title} ")
    } else {
        String::new()
    };

    view! {
        <FlexBox
            semantics=Semantics::new(Role::Separator, title)
            border=BorderStyle::None
            padding=Padding::zero()
            width=props.width
            height=1
            overflow=Overflow::Hidden
            align_items=Align::Stretch
            opt_min_width=props.min_width
            opt_max_width=props.max_width
        >
            {#if props.children.is_some()}
                {@render props.children}
            {:else}
                <Paragraph fill='─' fg=color>{text}</Paragraph>
            {/if}
        </FlexBox>
    }
}
