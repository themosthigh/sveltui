use crossterm::style::Color;
use sveltui::prelude::*;

#[derive(Props)]
pub struct ProgressBarProps {
    #[prop(default = 0.0)]
    pub ratio: f32,
    #[prop(default = 20)]
    pub width: u16,
    #[prop(default = true)]
    pub show_percentage: bool,
    #[prop(default = Color::White)]
    pub fill_color: Color,
    #[prop(optional)]
    pub title: Option<String>,
}

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
        <FlexBox
            semantics=semantics
            border=BorderStyle::None
            padding=Padding::zero()
            justify_content=Justify::Center
            align_items=Align::Start
        >
            <TextSpan fg=props.fill_color bold=true>{bar_text}</TextSpan>
        </FlexBox>
    }
}
