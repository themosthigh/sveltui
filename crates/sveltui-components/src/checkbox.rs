use crossterm::style::Color;
use std::rc::Rc;
use sveltui::prelude::*;

#[derive(Clone, Copy)]
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

impl CheckboxBinding {
    pub fn get(&self) -> bool {
        match self {
            Self::Read(r) => r.get(),
            Self::State(s) => s.get(),
        }
    }
}

impl IntoPropOption<CheckboxBinding> for CheckboxBinding {
    fn into_prop_option(self) -> Option<CheckboxBinding> {
        Some(self)
    }
}

impl IntoPropOption<CheckboxBinding> for ReadSignal<bool> {
    fn into_prop_option(self) -> Option<CheckboxBinding> {
        Some(CheckboxBinding::Read(self))
    }
}

impl IntoPropOption<CheckboxBinding> for State<bool> {
    fn into_prop_option(self) -> Option<CheckboxBinding> {
        Some(CheckboxBinding::State(self))
    }
}

#[derive(Props)]
pub struct CheckboxProps {
    #[prop(optional)]
    pub id: Option<FocusId>,
    #[prop(optional)]
    pub checked: Option<CheckboxBinding>,
    #[prop(optional)]
    pub set_checked: Option<WriteSignal<bool>>,
    #[prop(optional)]
    pub label: Option<String>,
    #[prop(default = false)]
    pub disabled: bool,
    #[snippet(optional)]
    pub children: Option<Snippet>,
}

#[component]
pub fn Checkbox(props: CheckboxProps) -> impl IntoNode {
    let id = props.id.unwrap_or_default();
    let is_checked = props.checked.map(|c| c.get()).unwrap_or(false);
    let focused = is_focused(id);

    let box_glyph = if is_checked { "[x]" } else { "[ ]" };
    let box_prefix = if focused { "▶ " } else { "  " };
    let text_color = if props.disabled {
        Color::DarkGrey
    } else {
        Color::White
    };

    let label_str = props.label.unwrap_or_default();
    let binding = props.checked;
    let set_checked = props.set_checked;
    let disabled = props.disabled;

    let on_click: Option<Rc<dyn Fn()>> = if disabled {
        None
    } else if let Some(sc) = set_checked {
        Some(Rc::new(move || sc.update(|c| *c = !*c)))
    } else if let Some(CheckboxBinding::State(s)) = binding {
        Some(Rc::new(move || s.update(|c| *c = !*c)))
    } else {
        None
    };

    let box_display = format!("{}{}", box_prefix, box_glyph);

    view! {
        <FlexBox
            border=BorderStyle::None
            padding=Padding::zero()
            justify_content=Justify::Center
            align_items=Align::Start
            focus_id=id
            tab_stop=true
            disabled=props.disabled
            activation=Activation::Toggle
            semantics=Semantics::new(Role::Checkbox, label_str.clone()).checked(is_checked)
            opt_on_click=on_click
            flex_direction=FlexDirection::Row
        >
            <TextSpan fg=text_color bold=focused>{box_display}</TextSpan>
            {#if props.children.is_some()}
                {@render props.children}
            {:else if !label_str.is_empty()}
                <TextSpan fg=text_color bold=focused>{format!(" {}", label_str)}</TextSpan>
            {/if}
        </FlexBox>
    }
}
