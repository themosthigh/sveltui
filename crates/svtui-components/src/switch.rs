use crossterm::style::Color;
use std::rc::Rc;
use sveltui::prelude::*;

#[derive(Clone, Copy)]
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

impl SwitchBinding {
    pub fn get(&self) -> bool {
        match self {
            Self::Read(r) => r.get(),
            Self::State(s) => s.get(),
        }
    }
}

impl IntoPropOption<SwitchBinding> for SwitchBinding {
    fn into_prop_option(self) -> Option<SwitchBinding> {
        Some(self)
    }
}

impl IntoPropOption<SwitchBinding> for ReadSignal<bool> {
    fn into_prop_option(self) -> Option<SwitchBinding> {
        Some(SwitchBinding::Read(self))
    }
}

impl IntoPropOption<SwitchBinding> for State<bool> {
    fn into_prop_option(self) -> Option<SwitchBinding> {
        Some(SwitchBinding::State(self))
    }
}

#[derive(Props)]
pub struct SwitchProps {
    #[prop(optional)]
    pub id: Option<FocusId>,
    #[prop(optional)]
    pub checked: Option<SwitchBinding>,
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
pub fn Switch(props: SwitchProps) -> impl IntoNode {
    let id = props.id.unwrap_or_default();
    let is_on = props.checked.map(|c| c.get()).unwrap_or(false);
    let focused = is_focused(id);

    let toggle_glyph = if is_on { "[●]" } else { "[ ]" };
    let switch_prefix = if focused { "▶ " } else { "  " };
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
    } else if let Some(SwitchBinding::State(s)) = binding {
        Some(Rc::new(move || s.update(|c| *c = !*c)))
    } else {
        None
    };

    let switch_display = format!("{}{}", switch_prefix, toggle_glyph);

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
            semantics=Semantics::new(Role::Switch, label_str.clone()).checked(is_on)
            opt_on_click=on_click
            flex_direction=FlexDirection::Row
        >
            <TextSpan fg=text_color bold=focused>{switch_display}</TextSpan>
            {#if props.children.is_some()}
                {@render props.children}
            {:else if !label_str.is_empty()}
                <TextSpan fg=text_color bold=focused>{format!(" {}", label_str)}</TextSpan>
            {/if}
        </FlexBox>
    }
}
