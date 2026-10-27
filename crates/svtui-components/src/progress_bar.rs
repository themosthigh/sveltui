use crossterm::style::Color;
use svtui_element::{BoxElement, IntoNode, TextElement, VNode};
use svtui_layout::{Align, BorderStyle, Justify, Padding};

#[derive(Clone)]
pub struct ProgressBar {
    ratio: f32,
    width: u16,
    show_percentage: bool,
    fill_color: Color,
    title: Option<String>,
}

impl Default for ProgressBar {
    fn default() -> Self {
        Self::new(0.0)
    }
}

impl ProgressBar {
    pub fn new(ratio: f32) -> Self {
        Self {
            ratio: ratio.clamp(0.0, 1.0),
            width: 20,
            show_percentage: true,
            fill_color: Color::White,
            title: None,
        }
    }

    pub fn ratio(mut self, ratio: f32) -> Self {
        self.ratio = ratio.clamp(0.0, 1.0);
        self
    }

    pub fn width(mut self, width: u16) -> Self {
        self.width = width;
        self
    }

    pub fn show_percentage(mut self, show: bool) -> Self {
        self.show_percentage = show;
        self
    }

    pub fn fill_color(mut self, color: Color) -> Self {
        self.fill_color = color;
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }
}

impl IntoNode for ProgressBar {
    fn into_node(self) -> VNode {
        let bar_width = self.width as usize;
        let filled_count = ((self.ratio * bar_width as f32).round() as usize).min(bar_width);
        let empty_count = bar_width.saturating_sub(filled_count);

        let filled: String = "█".repeat(filled_count);
        let empty: String = "░".repeat(empty_count);

        let percentage = (self.ratio * 100.0).round() as u32;
        let bar_text = if self.show_percentage {
            format!("[{}{}] {:>3}%", filled, empty, percentage)
        } else {
            format!("[{}{}]", filled, empty)
        };

        let mut b = BoxElement::new()
            .border(BorderStyle::None)
            .padding(Padding::zero())
            .justify_content(Justify::Center)
            .align_items(Align::Start)
            .child(
                TextElement::new(bar_text)
                    .color(self.fill_color)
                    .bold(true)
                    .into_node(),
            );

        if let Some(ref title) = self.title {
            b = b.title(title.clone());
        }

        b.into_node()
    }
}
