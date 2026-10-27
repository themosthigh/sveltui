use crossterm::style::Color;
use svtui_element::{BoxElement, IntoNode, TextElement, VNode};
use svtui_layout::{Align, BorderStyle, Justify, Padding, Size};

#[derive(Clone)]
pub struct Divider {
    title: Option<String>,
    color: Option<Color>,
    width: Size,
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
}

impl IntoNode for Divider {
    fn into_node(self) -> VNode {
        let text = match self.title {
            Some(t) => format!("─── {} ──────────────────────────", t),
            None => "──────────────────────────────────────".to_string(),
        };

        BoxElement::new()
            .border(BorderStyle::None)
            .padding(Padding::zero())
            .width(self.width)
            .align_items(Align::Center)
            .justify_content(Justify::Start)
            .child(
                TextElement::new(text)
                    .color(self.color.unwrap_or(Color::DarkGrey))
                    .into_node(),
            )
            .into_node()
    }
}
