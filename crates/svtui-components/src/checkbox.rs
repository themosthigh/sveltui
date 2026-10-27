use crossterm::style::Color;
use svtui_element::{BoxElement, FocusId, IntoNode, TextElement, VNode, is_focused};
use svtui_layout::{Align, BorderStyle, Justify, Padding};
use svtui_reactive::{ReadSignal, WriteSignal};

#[derive(Clone, Default)]
pub struct Checkbox {
    checked: Option<ReadSignal<bool>>,
    set_checked: Option<WriteSignal<bool>>,
    label: String,
    disabled: bool,
    id: Option<FocusId>,
}

impl Checkbox {
    pub fn new(
        checked: ReadSignal<bool>,
        set_checked: WriteSignal<bool>,
        label: impl Into<String>,
    ) -> Self {
        Self {
            checked: Some(checked),
            set_checked: Some(set_checked),
            label: label.into(),
            disabled: false,
            id: None,
        }
    }

    pub fn checked(mut self, checked: ReadSignal<bool>) -> Self {
        self.checked = Some(checked);
        self
    }

    pub fn set_checked(mut self, set_checked: WriteSignal<bool>) -> Self {
        self.set_checked = Some(set_checked);
        self
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

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn id(mut self, id: FocusId) -> Self {
        self.id = Some(id);
        self
    }
}

impl IntoNode for Checkbox {
    fn into_node(self) -> VNode {
        let id = self.id.unwrap_or_default();
        let is_checked = self.checked.map(|c| c.get()).unwrap_or(false);
        let focused = is_focused(id);

        let box_glyph = if is_checked { "[x]" } else { "[ ]" };
        let box_prefix = if focused { "▶ " } else { "  " };

        let text_color = if self.disabled {
            Color::DarkGrey
        } else {
            Color::White
        };

        let box_display = format!("{}{}", box_prefix, box_glyph);

        let mut b = BoxElement::new()
            .border(BorderStyle::None)
            .padding(Padding::zero())
            .align_items(Align::Center)
            .justify_content(Justify::Start)
            .focus_id(id)
            .child(
                TextElement::new(box_display)
                    .color(text_color)
                    .bold(focused)
                    .into_node(),
            );

        if !self.label.is_empty() {
            b = b.child(
                TextElement::new(format!(" {}", self.label))
                    .color(text_color)
                    .bold(focused)
                    .into_node(),
            );
        }

        if let Some(set_checked) = self.set_checked.filter(|_| !self.disabled) {
            b = b.on_click(move || {
                set_checked.update(|c| *c = !*c);
            });
        }

        b.into_node()
    }
}
