use crossterm::style::Color;
use svtui_element::{BoxElement, IntoNode, TextElement, VNode};
use svtui_layout::{Align, BorderStyle, Overflow, Padding, Size};

/// A one-line rule that fills its allocated width, including after resizing.
/// Width defaults to Fill; fixed/percentage sizes and min/max bounds are supported.
#[derive(Clone)]
pub struct Divider {
    title: Option<String>,
    color: Option<Color>,
    width: Size,
    min_width: Option<Size>,
    max_width: Option<Size>,
}

impl Default for Divider {
    fn default() -> Self {
        Self::new()
    }
}

impl Divider {
    pub fn new() -> Self {
        Self {
            title: None,
            color: Some(Color::DarkGrey),
            width: Size::Fill,
            min_width: None,
            max_width: None,
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn child(mut self, child: impl IntoNode) -> Self {
        if let VNode::Text(t) = child.into_node() {
            self.title = Some(t.content);
        }
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    pub fn width(mut self, width: impl Into<Size>) -> Self {
        self.width = width.into();
        self
    }

    pub fn min_width(mut self, width: impl Into<Size>) -> Self {
        self.min_width = Some(width.into());
        self
    }

    pub fn max_width(mut self, width: impl Into<Size>) -> Self {
        self.max_width = Some(width.into());
        self
    }
}

impl IntoNode for Divider {
    fn into_node(self) -> VNode {
        let text = self
            .title
            .filter(|title| !title.is_empty())
            .map_or_else(String::new, |title| format!("─── {title} "));

        let mut element = BoxElement::new()
            .border(BorderStyle::None)
            .padding(Padding::zero())
            .width(self.width)
            .height(1)
            .overflow(Overflow::Hidden)
            .align_items(Align::Stretch)
            .child(
                TextElement::new(text)
                    .fill('─')
                    .color(self.color.unwrap_or(Color::DarkGrey))
                    .into_node(),
            );
        if let Some(min) = self.min_width {
            element = element.min_width(min);
        }
        if let Some(max) = self.max_width {
            element = element.max_width(max);
        }
        element.into_node()
    }
}
