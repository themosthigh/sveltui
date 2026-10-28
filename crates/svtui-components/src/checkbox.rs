use crossterm::style::Color;
use svtui_element::{Activation, Role, Semantics};
use svtui_element::{BoxElement, FocusId, IntoNode, TextElement, VNode, is_focused};
use svtui_layout::{Align, BorderStyle, Justify, Padding};
use svtui_reactive::{ReadSignal, State, WriteSignal};

pub enum CheckboxBinding {
    Read(ReadSignal<bool>),
    State(State<bool>),
}

impl From<ReadSignal<bool>> for CheckboxBinding {
    fn from(s: ReadSignal<bool>) -> Self {
        Self::Read(s)
    }
}

impl From<State<bool>> for CheckboxBinding {
    fn from(s: State<bool>) -> Self {
        Self::State(s)
    }
}

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

    pub fn checked(mut self, checked: impl Into<CheckboxBinding>) -> Self {
        match checked.into() {
            CheckboxBinding::Read(r) => {
                self.checked = Some(r);
            }
            CheckboxBinding::State(s) => {
                self.checked = Some(s.read());
                self.set_checked = Some(s.write());
            }
        }
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
            .justify_content(Justify::Center)
            .align_items(Align::Start)
            .focus_id(id)
            .tab_stop(true)
            .disabled(self.disabled)
            .activation(Activation::Toggle)
            .semantics(Semantics::new(Role::Checkbox, self.label.clone()).checked(is_checked))
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
