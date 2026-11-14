use crossterm::style::Color;
use std::rc::Rc;
use sveltui::prelude::*;

use super::checkbox_types::*;
use crate::controllable::use_controllable;

#[component]
pub fn Checkbox(props: CheckboxProps) -> impl IntoNode {
    let id = props.id.unwrap_or_default();
    let checked = use_controllable(
        props.checked.bound(),
        props.checked.value(),
        || props.default_checked,
        props.on_checked_change,
    );
    let is_checked = checked.get();
    let focused = is_focused(id);

    let box_glyph = if is_checked { "[x]" } else { "[ ]" };
    let box_prefix = if focused { "▶ " } else { "  " };
    let text_color = if props.disabled {
        Color::DarkGrey
    } else {
        Color::White
    };

    let label_str = props.label.unwrap_or_default();
    let disabled = props.disabled;

    let on_click: Option<Rc<dyn Fn()>> =
        (!disabled).then(|| Rc::new(move || checked.set(!checked.get())) as Rc<dyn Fn()>);

    let box_display = format!("{}{}", box_prefix, box_glyph);

    view! {
        <Pressable
            focus_id=id
            tab_stop=true
            disabled={props.disabled}
            activation=Activation::Toggle
            semantics={Semantics::new(Role::Checkbox, label_str.clone()).checked(is_checked)}
            on_click=on_click
        >
            <View
                border=BorderStyle::None
                padding={Padding::zero()}
                justify_content=JustifyContent::Center
                align_items=AlignItems::FlexStart
                flex_direction=FlexDirection::Row
            >
                <TextSpan foreground_color=text_color bold=focused>{box_display}</TextSpan>
                {#if props.children.is_some()}
                    {@render props.children}
                {:else if !label_str.is_empty()}
                    <TextSpan foreground_color=text_color bold=focused>{format!(" {}", label_str)}</TextSpan>
                {/if}
            </View>
        </Pressable>
    }
}
