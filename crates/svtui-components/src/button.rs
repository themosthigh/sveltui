use crossterm::style::Color;
use std::rc::Rc;
use svtui::prelude::*;

#[derive(Props)]
pub struct ButtonProps {
    #[prop(optional)]
    pub id: Option<FocusId>,
    #[prop(default = false)]
    pub disabled: bool,
    #[prop(optional)]
    pub on_click: Option<Rc<dyn Fn()>>,
    #[prop(default)]
    pub width: Size,
    #[prop(default)]
    pub height: Size,
    #[prop(optional)]
    pub border: Option<BorderStyle>,
    #[prop(optional)]
    pub border_color: Option<Color>,
    #[prop(default)]
    pub padding: Padding,
    #[prop(optional)]
    pub label: Option<String>,
    #[snippet(optional)]
    pub children: Option<Snippet>,
}

#[component]
pub fn Button(props: ButtonProps) -> impl IntoNode {
    let id = props.id.unwrap_or_default();
    let focus = use_focus(id);
    let is_active = focus.is_active();

    let (default_border_color, default_text_color, default_border_style) = if props.disabled {
        (Color::DarkGrey, Color::DarkGrey, BorderStyle::Single)
    } else if is_active {
        (Color::White, Color::White, BorderStyle::Double)
    } else {
        (Color::DarkGrey, Color::White, BorderStyle::Single)
    };

    let border_style = props.border.unwrap_or(default_border_style);
    let border_color = props.border_color.or(Some(default_border_color));
    let text_color = default_text_color;
    let on_click = props.on_click.filter(|_| !props.disabled);

    view! {
        <Box
            border=border_style
            opt_border_color=border_color
            padding=props.padding
            width=props.width
            height=props.height
            align_items=Align::Center
            justify_content=Justify::Center
            focus_id=id
            tab_stop=true
            disabled=props.disabled
            activation=Activation::Press
            semantics=Semantics::new(Role::Button, "")
            opt_on_click=on_click
        >
            {#if props.children.is_some()}
                {@render props.children}
            {:else if let Some(label) = &props.label}
                <Text fg=text_color bold=is_active>{label.clone()}</Text>
            {/if}
        </Box>
    }
}
