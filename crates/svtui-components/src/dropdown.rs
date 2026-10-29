use crossterm::event::{KeyCode, KeyEvent};
use crossterm::style::Color;
use std::rc::Rc;
use svtui::prelude::*;

#[derive(Props)]
pub struct DropdownItemProps {
    #[prop(optional)]
    pub label: Option<String>,
    #[prop(default = false)]
    pub disabled: bool,
    #[prop(optional)]
    pub id: Option<FocusId>,
    #[prop(optional)]
    pub on_select: Option<Rc<dyn Fn()>>,
    #[prop(optional)]
    pub on_click: Option<Rc<dyn Fn()>>,
    #[prop(optional)]
    pub focus_bg: Option<Color>,
    #[prop(optional)]
    pub focus_fg: Option<Color>,
    #[snippet(optional)]
    pub children: Option<Snippet>,
}

#[component]
pub fn DropdownItem(props: DropdownItemProps) -> impl IntoNode {
    let id = props.id.unwrap_or_default();
    let active = use_focus(id).is_active();
    let label_fg = if props.disabled {
        Color::DarkGrey
    } else if active {
        props.focus_fg.unwrap_or(Color::Black)
    } else {
        Color::White
    };
    let row_bg = active.then_some(props.focus_bg.unwrap_or(Color::White));
    let label_str = props.label.unwrap_or_default();
    let label_text = format!(" {}", label_str);
    let semantics = Semantics::new(Role::MenuItem, label_str);
    let select = props.on_select.or(props.on_click);

    let on_key: KeyHandler = Rc::new(move |key: KeyEvent, context: &mut EventContext| {
        if matches!(key.code, KeyCode::Enter | KeyCode::Char(' ')) {
            if let Some(select) = &select {
                select();
            }
            context.dismiss_chain();
            EventResult::Handled
        } else {
            EventResult::Ignored
        }
    });

    view! {
        <FlexBox
            width=Size::Fill
            padding=Padding::zero()
            flex_direction=FlexDirection::Row
            overflow=Overflow::Hidden
            focus_id=id
            tab_stop=true
            disabled=props.disabled
            activation=Activation::Press
            semantics=semantics
            opt_bg=row_bg
            on_key=on_key
        >
            {#if props.children.is_some()}
                {@render props.children}
            {:else}
                <TextSpan fg=label_fg bold=active>{label_text}</TextSpan>
            {/if}
        </FlexBox>
    }
}

#[derive(Props)]
pub struct DropdownSubmenuProps {
    #[prop(optional)]
    pub label: Option<String>,
    #[prop(default = false)]
    pub disabled: bool,
    #[prop(optional)]
    pub id: Option<FocusId>,
    #[prop(default = Size::Fixed(24))]
    pub width: Size,
    #[prop(default = Placement::RightStart)]
    pub placement: Placement,
    #[prop(optional)]
    pub open: Option<bool>,
    #[prop(optional)]
    pub layer_id: Option<FocusId>,
    #[prop(optional)]
    pub bg: Option<Color>,
    #[prop(optional)]
    pub on_close: Option<Rc<dyn Fn()>>,
    #[prop(optional)]
    pub on_open: Option<Rc<dyn Fn()>>,
    #[prop(optional)]
    pub focus_bg: Option<Color>,
    #[prop(optional)]
    pub focus_fg: Option<Color>,
    #[snippet(optional)]
    pub children: Option<Snippet>,
}

#[component]
pub fn DropdownSubmenu(props: DropdownSubmenuProps) -> impl IntoNode {
    let id = props.id.unwrap_or_default();
    let layer_id = props.layer_id.unwrap_or_default();
    let managed = props.open.is_none();
    let is_open = props.open.unwrap_or_else(|| is_layer_open(layer_id));
    let active = use_focus(id).is_active() || is_open;
    let label_fg = if props.disabled {
        Color::DarkGrey
    } else if active {
        props.focus_fg.unwrap_or(Color::Black)
    } else {
        Color::White
    };
    let arrow_fg = if active && !props.disabled {
        label_fg
    } else {
        Color::DarkGrey
    };
    let row_bg = active.then_some(props.focus_bg.unwrap_or(Color::White));
    let label_str = props.label.unwrap_or_default();
    let label_text = format!(" {}", label_str);
    let semantics = Semantics::new(Role::MenuItem, label_str.clone())
        .expanded(is_open)
        .controls(layer_id);
    let layer_semantics = Semantics::new(Role::Menu, label_str);
    let menu_bg = props.bg.unwrap_or(Color::Black);

    let on_key: KeyHandler = Rc::new(move |key: KeyEvent, context: &mut EventContext| {
        if matches!(
            key.code,
            KeyCode::Right | KeyCode::Enter | KeyCode::Char(' ')
        ) {
            if is_open && key.code != KeyCode::Right {
                context.dismiss(layer_id);
            } else {
                context.open(layer_id);
            }
            EventResult::Handled
        } else {
            EventResult::Ignored
        }
    });

    view! {
        <Overlay
            id=layer_id
            opener=id
            open=is_open
            managed=managed
            layer=DismissLayer::submenu()
            focus_group=FocusGroup::menu()
            placement=props.placement
            semantics=layer_semantics
            opt_on_close=props.on_close
            opt_on_open=props.on_open
            trigger={view! {
                <FlexBox
                    width=Size::Fill
                    padding=Padding::zero()
                    flex_direction=FlexDirection::Row
                    overflow=Overflow::Hidden
                    focus_id=id
                    tab_stop=true
                    disabled=props.disabled
                    activation=Activation::Press
                    semantics=semantics
                    opt_bg=row_bg
                    on_key=on_key
                >
                    <FlexBox width=Size::Fill overflow=Overflow::Hidden>
                        <TextSpan fg=label_fg bold=active>{label_text}</TextSpan>
                    </FlexBox>
                    <TextSpan fg=arrow_fg bold=active>"▶ "</TextSpan>
                </FlexBox>
            }}
        >
            <FlexBox
                border=BorderStyle::Single
                border_color=Color::White
                bg=menu_bg
                width=props.width
                padding=Padding::zero()
            >
                {#if props.children.is_some()}
                    {@render props.children}
                {/if}
            </FlexBox>
        </Overlay>
    }
}

#[derive(Props)]
pub struct DropdownMenuProps {
    #[prop(optional)]
    pub id: Option<FocusId>,
    #[prop(optional)]
    pub open: Option<bool>,
    #[prop(default = Placement::BottomStart)]
    pub placement: Placement,
    #[prop(default = Size::Fixed(24))]
    pub width: Size,
    #[prop(optional)]
    pub bg: Option<Color>,
    #[prop(optional)]
    pub on_close: Option<Rc<dyn Fn()>>,
    #[prop(optional)]
    pub on_open: Option<Rc<dyn Fn()>>,
    #[prop(optional)]
    pub on_key: Option<KeyHandler>,
    #[snippet(optional)]
    pub trigger: Option<Snippet>,
    #[snippet(optional)]
    pub children: Option<Snippet>,
}

#[component]
pub fn DropdownMenu(props: DropdownMenuProps) -> impl IntoNode {
    let id = props.id.unwrap_or_default();
    let menu_bg = props.bg.unwrap_or(Color::Black);
    let managed = props.open.is_none();
    let open = props.open.unwrap_or_else(|| is_layer_open(id));

    view! {
        <Overlay
            id=id
            open=open
            managed=managed
            layer=DismissLayer::menu()
            focus_group=FocusGroup::menu()
            placement=props.placement
            semantics=Semantics::new(Role::Menu, "Menu")
            opt_on_close=props.on_close
            opt_on_open=props.on_open
            opt_on_key=props.on_key
            opt_slot_trigger=props.trigger
        >
            <FlexBox
                border=BorderStyle::Single
                border_color=Color::White
                bg=menu_bg
                width=props.width
                padding=Padding::zero()
            >
                {#if props.children.is_some()}
                    {@render props.children}
                {/if}
            </FlexBox>
        </Overlay>
    }
}
