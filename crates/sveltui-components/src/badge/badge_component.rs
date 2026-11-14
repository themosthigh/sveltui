use sveltui::prelude::*;

use super::badge_types::*;

#[component]
pub fn Badge(props: BadgeProps) -> impl IntoNode {
    view! {
        <View
            border=BorderStyle::Single
            border_color={props.fg}
            background_color={props.bg}
            foreground_color={props.fg}
            align_items=AlignItems::Center
            justify_content=JustifyContent::Center
        >
        {#if props.children.is_some()}
            {@render props.children}
        {:else if let Some(label) = &props.label}
            <TextSpan foreground_color={props.fg}>{label.clone()}</TextSpan>
        {/if}
        </View>
    }
}
