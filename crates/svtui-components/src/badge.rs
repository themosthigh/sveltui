use crossterm::style::Color;
use svtui_element::{BoxElement, IntoNode, TextElement, VNode};
use svtui_layout::{Align, BorderStyle, Justify, Padding};

#[derive(Clone)]
pub struct Badge {
    label: String,
    color: Color,
    bg: Option<Color>,
}

impl Default for Badge {
    fn default() -> Self {
        Self::new("")
    }
}

impl Badge {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            color: Color::White,
            bg: None,
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    pub fn child(mut self, child: impl IntoNode) -> Self {
        if let VNode::Text(t) = child.into_node() {
            self.label = t.content;
        }
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn bg(mut self, bg: Color) -> Self {
        self.bg = Some(bg);
        self
    }
}

impl IntoNode for Badge {
    fn into_node(self) -> VNode {
        let text = format!(" {} ", self.label);

        let mut b = BoxElement::new()
            .border(BorderStyle::Single)
            .border_color(self.color)
            .padding(Padding::zero())
            .align_items(Align::Center)
            .justify_content(Justify::Center)
            .child(
                TextElement::new(text)
                    .color(self.color)
                    .bold(true)
                    .into_node(),
            );

        if let Some(bg) = self.bg {
            b = b.bg(bg);
        }

        b.into_node()
    }
}
