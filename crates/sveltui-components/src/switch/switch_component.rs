use crossterm::style::Color;
use std::rc::Rc;
use sveltui::prelude::*;

use super::switch_types::*;
use crate::controllable::use_controllable;

#[component]
pub fn Switch(props: SwitchProps) -> impl IntoNode {
    let id = props.id.unwrap_or_default();
    let checked = use_controllable(
        props.checked.bound(),
        props.checked.value(),
        || props.default_checked,
        props.on_checked_change,
    );
    let is_on = checked.get();
    let focused = is_focused(id);

    let toggle_glyph = if is_on { "[●]" } else { "[ ]" };
    let switch_prefix = if focused { "▶ " } else { "  " };
    let text_color = if props.disabled {
        Color::DarkGrey
    } else {
        Color::White
    };

    let label_str = props.label.unwrap_or_default();
    let disabled = props.disabled;

    let on_click: Option<Rc<dyn Fn()>> =
        (!disabled).then(|| Rc::new(move || checked.set(!checked.get())) as Rc<dyn Fn()>);

    let switch_display = format!("{}{}", switch_prefix, toggle_glyph);

    view! {
        <Pressable
            focus_id=id
            tab_stop=true
            disabled={props.disabled}
            activation=Activation::Toggle
            semantics={Semantics::new(Role::Switch, label_str.clone()).checked(is_on)}
            on_click=on_click
        >
            <View
                border=BorderStyle::None
                padding={Padding::zero()}
                justify_content=JustifyContent::Center
                align_items=AlignItems::FlexStart
                flex_direction=FlexDirection::Row
            >
                <TextSpan foreground_color=text_color bold=focused>{switch_display}</TextSpan>
                {#if props.children.is_some()}
                    {@render props.children}
                {:else if !label_str.is_empty()}
                    <TextSpan foreground_color=text_color bold=focused>{format!(" {}", label_str)}</TextSpan>
                {/if}
            </View>
        </Pressable>
    }
}
