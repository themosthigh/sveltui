use crossterm::style::Color;
use sveltui::prelude::*;

use super::button_types::*;

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
    let border_color = props.border_color.unwrap_or(default_border_color);
    let text_color = default_text_color;
    let on_click = props.on_click.filter(|_| !props.disabled);

    view! {
        <Pressable
            focus_id=id
            tab_stop=true
            disabled={props.disabled}
            activation=Activation::Press
            semantics={Semantics::new(Role::Button, "")}
            on_click=on_click
        >
            <View
                border=border_style
                border_color=border_color
                padding={props.padding}
                width={props.width}
                height={props.height}
                align_items=AlignItems::Center
                justify_content=JustifyContent::Center
            >
                {#if props.children.is_some()}
                    {@render props.children}
                {:else if let Some(label) = &props.label}
                    <TextSpan foreground_color=text_color bold=is_active>{label.clone()}</TextSpan>
                {/if}
            </View>
        </Pressable>
    }
}
