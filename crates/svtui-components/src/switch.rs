use crossterm::style::Color;
use svtui_element::{Activation, Role, Semantics};
use svtui_element::{BoxElement, FocusId, IntoNode, TextElement, VNode, is_focused};
use svtui_layout::{Align, BorderStyle, Justify, Padding};
use svtui_reactive::{ReadSignal, State, WriteSignal};

pub enum SwitchBinding {
    Read(ReadSignal<bool>),
    State(State<bool>),
}

impl From<ReadSignal<bool>> for SwitchBinding {
    fn from(s: ReadSignal<bool>) -> Self {
        Self::Read(s)
    }
}

impl From<State<bool>> for SwitchBinding {
    fn from(s: State<bool>) -> Self {
        Self::State(s)
    }
}

#[derive(Clone, Default)]
pub struct Switch {
    checked: Option<ReadSignal<bool>>,
    set_checked: Option<WriteSignal<bool>>,
    label: Option<String>,
    disabled: bool,
    id: Option<FocusId>,
}

impl Switch {
    pub fn new(checked: ReadSignal<bool>, set_checked: WriteSignal<bool>) -> Self {
        Self {
            checked: Some(checked),
            set_checked: Some(set_checked),
            label: None,
            disabled: false,
            id: None,
        }
    }

    pub fn checked(mut self, checked: impl Into<SwitchBinding>) -> Self {
        match checked.into() {
            SwitchBinding::Read(r) => {
                self.checked = Some(r);
            }
            SwitchBinding::State(s) => {
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
        self.label = Some(label.into());
        self
    }

    pub fn child(mut self, child: impl IntoNode) -> Self {
        if let VNode::Text(t) = child.into_node() {
            self.label = Some(t.content);
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

impl IntoNode for Switch {
    fn into_node(self) -> VNode {
        let id = self.id.unwrap_or_default();
        let is_on = self.checked.map(|c| c.get()).unwrap_or(false);
        let focused = is_focused(id);

        let toggle_glyph = if is_on { "[●]" } else { "[ ]" };

        // Base-UI Black & White styling
        let switch_prefix = if focused { "▶ " } else { "  " };
        let switch_color = if self.disabled {
            Color::DarkGrey
        } else {
            Color::White
        };

        let label_color = if self.disabled {
            Color::DarkGrey
        } else {
            Color::White
        };

        let label_text = self.label.as_deref().unwrap_or("");
        let switch_display = format!("{}{}", switch_prefix, toggle_glyph);

        let mut b = BoxElement::new()
            .border(BorderStyle::None)
            .padding(Padding::zero())
            .justify_content(Justify::Center)
            .align_items(Align::Start)
            .focus_id(id)
            .tab_stop(true)
            .disabled(self.disabled)
            .activation(Activation::Toggle)
            .semantics(Semantics::new(Role::Switch, label_text).checked(is_on))
            .child(
                TextElement::new(switch_display)
                    .color(switch_color)
                    .bold(focused)
                    .into_node(),
            );

        if !label_text.is_empty() {
            b = b.child(
                TextElement::new(format!(" {}", label_text))
                    .color(label_color)
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
