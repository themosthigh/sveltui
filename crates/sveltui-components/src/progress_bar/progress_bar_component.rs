use sveltui::prelude::*;

use super::progress_bar_types::*;

#[component]
pub fn ProgressBar(props: ProgressBarProps) -> impl IntoNode {
    let ratio = props.ratio.clamp(0.0, 1.0);
    let bar_width = props.width as usize;
    let filled_count = ((ratio * bar_width as f32).round() as usize).min(bar_width);
    let empty_count = bar_width.saturating_sub(filled_count);

    let filled: String = "█".repeat(filled_count);
    let empty: String = "░".repeat(empty_count);

    let percentage = (ratio * 100.0).round() as u32;
    let bar_text = if props.show_percentage {
        format!("[{}{}] {:>3}%", filled, empty, percentage)
    } else {
        format!("[{}{}]", filled, empty)
    };

    let title_str = props.title.unwrap_or_default();
    let semantics = Semantics {
        value: Some(f64::from(ratio)),
        ..Semantics::new(Role::ProgressBar, title_str.clone())
    };

    view! {
        <Pressable semantics=semantics>
            <View
                border=BorderStyle::None
                padding={Padding::zero()}
                justify_content=JustifyContent::Center
                align_items=AlignItems::FlexStart
            >
                <TextSpan foreground_color={props.fill_color} bold=true>{bar_text}</TextSpan>
            </View>
        </Pressable>
    }
}
