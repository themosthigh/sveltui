use crossterm::style::Color;
use std::rc::Rc;
use svtui_element::{Activation, Role, Semantics};
use svtui_element::{BoxElement, FocusId, IntoNode, TextElement, VNode, use_focus};
use svtui_layout::{Align, BorderStyle, Justify, Padding, Size};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ButtonVariant {
    #[default]
    Primary,
    Secondary,
    Outline,
    Danger,
    Success,
}

#[derive(Clone, Default)]
pub struct Button {
    label: String,
    on_click: Option<Rc<dyn Fn()>>,
    variant: ButtonVariant,
    disabled: bool,
    id: Option<FocusId>,
    width: Size,
}

impl Button {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            on_click: None,
            variant: ButtonVariant::Primary,
            disabled: false,
            id: None,
            width: Size::Auto,
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

    pub fn on_click(mut self, handler: impl Fn() + 'static) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }

    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
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

    pub fn width(mut self, width: impl Into<Size>) -> Self {
        self.width = width.into();
        self
    }
}

impl IntoNode for Button {
    fn into_node(self) -> VNode {
        let id = self.id.unwrap_or_default();
        let focus = use_focus(id);
        let is_active = focus.is_active();

        // Base-UI Black & White styling
        let (border_color, text_color, border_style) = if self.disabled {
            (Color::DarkGrey, Color::DarkGrey, BorderStyle::Single)
        } else if is_active {
            (Color::White, Color::White, BorderStyle::Double)
        } else {
            match self.variant {
                ButtonVariant::Primary => (Color::White, Color::White, BorderStyle::Single),
                ButtonVariant::Secondary => (Color::DarkGrey, Color::White, BorderStyle::Single),
                ButtonVariant::Outline => (Color::DarkGrey, Color::DarkGrey, BorderStyle::Single),
                ButtonVariant::Danger => (Color::White, Color::White, BorderStyle::Single),
                ButtonVariant::Success => (Color::White, Color::White, BorderStyle::Single),
            }
        };

        let displayed_text = if is_active {
            format!("▶ {} ◀", self.label)
        } else {
            format!("  {}  ", self.label)
        };

        let mut b = BoxElement::new()
            .border(border_style)
            .border_color(border_color)
            .padding(Padding::zero())
            .width(self.width)
            .align_items(Align::Center)
            .justify_content(Justify::Center)
            .focus_id(id)
            .tab_stop(true)
            .disabled(self.disabled)
            .activation(Activation::Press)
            .semantics(Semantics::new(Role::Button, self.label.clone()))
            .child(
                TextElement::new(displayed_text)
                    .color(text_color)
                    .bold(is_active)
                    .into_node(),
            );

        if let Some(on_click) = self.on_click.filter(|_| !self.disabled) {
            b = b.on_click(move || on_click());
        }

        b.into_node()
    }
}
