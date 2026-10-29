use crossterm::event::{KeyCode, KeyModifiers};
use std::{
    cell::{Cell, RefCell},
    collections::HashSet,
    rc::Rc,
};
use sveltui::prelude::*;

fn identity(explicit: Option<FocusId>) -> FocusId {
    let stable = state_with(FocusId::new);
    explicit.unwrap_or_else(|| stable.get())
}

#[derive(Clone)]
struct MenuContext {
    id: FocusId,
    trigger: Rc<Cell<FocusId>>,
    triggered: Rc<Cell<bool>>,
    open: bool,
    change: Rc<dyn Fn(bool)>,
    kind: MenuTriggerKind,
}

#[derive(Clone)]
struct BarContext {
    values: Rc<RefCell<HashSet<String>>>,
    value: Option<String>,
    change: Rc<dyn Fn(Option<String>)>,
}

#[derive(Props)]
pub struct RootProps {
    #[prop(optional)]
    pub id: Option<FocusId>,
    #[prop(optional)]
    pub open: Option<bool>,
    #[prop(default = false)]
    pub default_open: bool,
    #[prop(optional)]
    pub on_open_change: Option<Rc<dyn Fn(bool)>>,
    #[snippet(optional)]
    pub children: Option<Snippet>,
}

fn root(props: RootProps, kind: MenuTriggerKind) -> VNode {
    let local = state(props.default_open);
    let open = props.open.unwrap_or_else(|| local.get());
    let change = Rc::new(move |next| {
        if next == open {
            return;
        }
        if props.open.is_none() {
            local.set(next);
        }
        if let Some(callback) = &props.on_open_change {
            callback(next);
        }
    });
    let context = MenuContext {
        id: identity(props.id),
        trigger: Rc::new(Cell::new(identity(None))),
        triggered: Rc::new(Cell::new(false)),
        open,
        change,
        kind,
    };
    provide_context(context, || props.children.render(()))
}

#[component]
pub fn Root(props: RootProps) -> VNode {
    let kind = MenuTriggerKind::Dropdown;
    root(props, kind)
}
#[component]
pub fn ContextRoot(props: RootProps) -> VNode {
    let kind = MenuTriggerKind::Context;
    root(props, kind)
}
#[component]
pub fn Sub(props: RootProps) -> VNode {
    let kind = MenuTriggerKind::Submenu;
    root(props, kind)
}
pub type SubProps = RootProps;

#[derive(Props)]
pub struct BarRootProps {
    #[prop(optional)]
    pub id: Option<FocusId>,
    #[prop(optional)]
    pub value: Option<Option<String>>,
    #[prop(optional)]
    pub default_value: Option<String>,
    #[prop(optional)]
    pub on_value_change: Option<Rc<dyn Fn(Option<String>)>>,
    #[snippet(optional)]
    pub children: Option<Snippet>,
}

#[component]
pub fn BarRoot(props: BarRootProps) -> VNode {
    let local = state(props.default_value);
    let current = props.value.clone().unwrap_or_else(|| local.get());
    let value = current.clone();
    let change = Rc::new(move |next: Option<String>| {
        if next == current {
            return;
        }
        if props.value.is_none() {
            local.set(next.clone());
        }
        if let Some(callback) = &props.on_value_change {
            callback(next);
        }
    });
    let children = provide_context(
        BarContext {
            value,
            change,
            values: Rc::new(RefCell::new(HashSet::new())),
        },
        || props.children.render(()),
    );
    let group = FocusGroup {
        orientation: Orientation::Horizontal,
        disabled_focusable: false,
        ..FocusGroup::menu()
    };
    FlexBoxElement::new()
        .focus_id(identity(props.id))
        .focus_group(group)
        .semantics(Semantics::new(Role::MenuBar, ""))
        .flex_direction(FlexDirection::Row)
        .child(children)
        .into_node()
}

#[derive(Props)]
pub struct BarMenuProps {
    pub value: String,
    #[prop(optional)]
    pub id: Option<FocusId>,
    #[snippet(optional)]
    pub children: Option<Snippet>,
}

#[component]
pub fn BarMenu(props: BarMenuProps) -> VNode {
    let bar = use_context::<BarContext>().expect("MenuBar::Menu requires MenuBar::Root");
    assert!(
        bar.values.borrow_mut().insert(props.value.clone()),
        "MenuBar::Menu values must be unique"
    );
    let open = bar.value.as_ref() == Some(&props.value);
    let change = Rc::new(move |next| {
        if next {
            (bar.change)(Some(props.value.clone()));
        } else if open {
            (bar.change)(None);
        }
    });
    let context = MenuContext {
        id: identity(props.id),
        trigger: Rc::new(Cell::new(identity(None))),
        triggered: Rc::new(Cell::new(false)),
        open,
        change,
        kind: MenuTriggerKind::MenuBar,
    };
    provide_context(context, || props.children.render(()))
}

fn style_text(node: &mut VNode, fg: Color, bold: bool, disabled: bool) {
    match node {
        VNode::Text(t) => {
            if disabled || t.fg.is_none() {
                t.fg = Some(fg);
            }
            t.bold = !disabled && (t.bold || bold);
        }
        VNode::FlexBox(b) => {
            for child in &mut b.children {
                style_text(child, fg, bold, disabled);
            }
        }
        VNode::Fragment(items) => {
            for child in items {
                style_text(child, fg, bold, disabled);
            }
        }
        _ => {}
    }
}

fn text(node: &VNode) -> String {
    match node {
        VNode::Text(t) => t.content.clone(),
        VNode::FlexBox(b) => b.children.iter().map(text).collect::<Vec<_>>().join(""),
        VNode::Fragment(items) => items.iter().map(text).collect::<Vec<_>>().join(""),
        _ => String::new(),
    }
}

#[derive(Props)]
pub struct TriggerProps {
    #[prop(optional)]
    pub id: Option<FocusId>,
    #[prop(default = false)]
    pub disabled: bool,
    #[prop(optional)]
    pub focus_bg: Option<Color>,
    #[prop(optional)]
    pub focus_fg: Option<Color>,
    #[snippet(optional)]
    pub children: Option<Snippet>,
}

#[component]
pub fn Trigger(props: TriggerProps) -> VNode {
    let menu = use_context::<MenuContext>().expect("Menu trigger requires a menu root");
    let id = props.id.unwrap_or_else(|| menu.trigger.get());
    menu.trigger.set(id);
    menu.triggered.set(true);
    let mut children = props.children.render(());
    let role = if matches!(
        menu.kind,
        MenuTriggerKind::Submenu | MenuTriggerKind::MenuBar
    ) {
        Role::MenuItem
    } else {
        Role::Button
    };
    let mut semantics = Semantics::new(role, text(&children));
    semantics.expanded = Some(menu.open);
    semantics.controls = Some(menu.id);
    semantics.has_popup = Some(Role::Menu);
    let interaction = Interaction {
        focusability: Focusability::TabStop,
        disabled: props.disabled,
        activation: Activation::None,
        semantics,
        menu: Some(MenuInteraction::Trigger {
            layer: menu.id,
            kind: menu.kind,
        }),
        ..Interaction::default()
    };
    let active = !props.disabled && (use_focus(id).is_active() || menu.open);
    let fg = if props.disabled {
        Color::DarkGrey
    } else if active {
        props.focus_fg.unwrap_or(Color::Black)
    } else {
        Color::White
    };
    style_text(&mut children, fg, active, props.disabled);
    let bg = active.then_some(props.focus_bg.unwrap_or(Color::White));
    let kind = menu.kind;
    let on_key: KeyHandler = Rc::new(move |key, ctx| {
        if key
            .modifiers
            .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT)
        {
            return EventResult::Ignored;
        }
        let initial = match (menu.kind, key.code) {
            (MenuTriggerKind::Context, KeyCode::F(10))
                if key.modifiers.contains(KeyModifiers::SHIFT) =>
            {
                Some(InitialFocus::First)
            }
            (MenuTriggerKind::Context, _) => return EventResult::Ignored,
            (MenuTriggerKind::Submenu, KeyCode::Right | KeyCode::Enter | KeyCode::Char(' ')) => {
                Some(InitialFocus::First)
            }
            (MenuTriggerKind::Submenu, _) => return EventResult::Ignored,
            (_, KeyCode::Up) => Some(InitialFocus::Last),
            (_, KeyCode::Down) => Some(InitialFocus::First),
            (_, KeyCode::Enter | KeyCode::Char(' ')) => {
                if menu.open {
                    ctx.dismiss(menu.id);
                    return EventResult::Handled;
                }
                Some(InitialFocus::First)
            }
            _ => None,
        };
        if let Some(initial) = initial {
            ctx.open_at(menu.id, initial);
            EventResult::Handled
        } else {
            EventResult::Ignored
        }
    });
    let mut row = FlexBoxElement::new()
        .focus_id(id)
        .semantics(interaction.semantics.clone())
        .fg(fg)
        .on_key(move |key, ctx| on_key(key, ctx))
        .padding(Padding::symmetric(1, 0))
        .child(children);
    row.interaction = Some(Box::new(interaction));
    row.bg = bg;
    if kind == MenuTriggerKind::Submenu {
        row.width = Size::Fill;
        row.flex_direction = FlexDirection::Row;
        row.padding = Padding::zero();
        let children = std::mem::take(&mut row.children);
        row.children.push(
            FlexBoxElement::new()
                .width(Size::Fill)
                .overflow(Overflow::Hidden)
                .padding(Padding::only(0, 0, 0, 1))
                .children(children)
                .into_node(),
        );
        row.children.push(TextElement::new("› ").fg(fg).into_node());
    }
    row.into_node()
}
pub use Trigger as SubTrigger;
pub type SubTriggerProps = TriggerProps;

#[derive(Props)]
pub struct ContentProps {
    #[prop(optional)]
    pub anchor: Option<FocusId>,
    #[prop(optional)]
    pub placement: Option<Placement>,
    #[prop(default = Size::Fixed(24))]
    pub width: Size,
    #[prop(default = BorderStyle::Single)]
    pub border: BorderStyle,
    #[prop(optional)]
    pub border_color: Option<Color>,
    #[prop(default = Color::Black)]
    pub bg: Color,
    #[prop(default = Padding::zero())]
    pub padding: Padding,
    #[prop(default = true)]
    pub loop_focus: bool,
    #[prop(optional)]
    pub on_key: Option<KeyHandler>,
    #[snippet(optional)]
    pub children: Option<Snippet>,
}

#[component]
pub fn Content(props: ContentProps) -> VNode {
    let scroll_id = identity(None);
    let menu = use_context::<MenuContext>().expect("Menu content requires a menu root");
    let placement = if menu.kind == MenuTriggerKind::Context {
        sveltui::element::layer_anchor(menu.id).map(|(x, y)| Placement::Anchor { x, y })
    } else {
        None
    }
    .or(props.placement)
    .unwrap_or(if menu.kind == MenuTriggerKind::Submenu {
        Placement::RightStart
    } else {
        Placement::BottomStart
    });
    let group = FocusGroup {
        wrap: props.loop_focus,
        disabled_focusable: false,
        ..FocusGroup::menu()
    };
    let layer = if menu.kind == MenuTriggerKind::Submenu {
        DismissLayer::submenu()
    } else {
        DismissLayer::menu()
    };
    let change = menu.change.clone();
    let mut overlay = OverlayElement::new()
        .id(menu.id)
        .opener(menu.trigger.get())
        .anchor(props.anchor.unwrap_or_else(|| menu.trigger.get()))
        .open(menu.open)
        .layer(layer)
        .focus_group(group)
        .placement(placement)
        .semantics(Semantics::new(Role::Menu, ""))
        .on_open(move || change(true))
        .on_close(move || (menu.change)(false));
    if !menu.triggered.get() {
        overlay.opener = None;
    }
    overlay.on_key = props.on_key;
    if menu.open {
        overlay = overlay.child(view! {
            <FlexBox focus_id=scroll_id width=props.width border=props.border border_color={props.border_color.unwrap_or(props.bg)}
                bg=props.bg padding=props.padding overflow=Overflow::Scroll>
                {@render props.children}
            </FlexBox>
        });
    }
    overlay.into_node()
}
pub use Content as SubContent;
pub type SubContentProps = ContentProps;

#[derive(Props)]
pub struct ItemProps {
    #[prop(optional)]
    pub id: Option<FocusId>,
    #[prop(default = false)]
    pub disabled: bool,
    #[prop(optional)]
    pub on_select: Option<Rc<dyn Fn()>>,
    #[prop(optional)]
    pub focus_bg: Option<Color>,
    #[prop(optional)]
    pub focus_fg: Option<Color>,
    #[snippet(optional)]
    pub children: Option<Snippet>,
}

#[component]
pub fn Item(props: ItemProps) -> VNode {
    let id = identity(props.id);
    let mut children = props.children.render(());
    let interaction = Interaction {
        focusability: Focusability::TabStop,
        disabled: props.disabled,
        activation: Activation::Press,
        semantics: Semantics::new(Role::MenuItem, text(&children)),
        menu: Some(MenuInteraction::Item),
        ..Interaction::default()
    };
    let active = !props.disabled && use_focus(id).is_focused();
    let fg = if props.disabled {
        Color::DarkGrey
    } else if active {
        props.focus_fg.unwrap_or(Color::Black)
    } else {
        Color::White
    };
    style_text(&mut children, fg, active, props.disabled);
    let on_key: KeyHandler = Rc::new(move |key, ctx| {
        if matches!(key.code, KeyCode::Enter | KeyCode::Char(' '))
            && !key
                .modifiers
                .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT)
        {
            if let Some(action) = &props.on_select {
                action();
            }
            ctx.dismiss_chain();
            EventResult::Handled
        } else {
            EventResult::Ignored
        }
    });
    let mut row = FlexBoxElement::new()
        .focus_id(id)
        .width(Size::Fill)
        .height(Size::Fixed(1))
        .fg(fg)
        .padding(Padding::symmetric(1, 0))
        .on_key(move |key, ctx| on_key(key, ctx))
        .child(children);
    row.interaction = Some(Box::new(interaction));
    row.bg = active.then_some(props.focus_bg.unwrap_or(Color::White));
    row.into_node()
}

#[derive(Props)]
pub struct GroupProps {
    #[snippet(optional)]
    pub children: Option<Snippet>,
}
#[component]
pub fn Group(props: GroupProps) -> VNode {
    fn heading(node: &VNode) -> Option<String> {
        match node {
            VNode::FlexBox(b)
                if b.interaction
                    .as_ref()
                    .is_some_and(|i| i.semantics.role == Role::Heading) =>
            {
                Some(text(node))
            }
            VNode::Fragment(items) => items.iter().find_map(heading),
            _ => None,
        }
    }
    let children = props.children.render(());
    let semantics = Semantics::new(Role::Group, heading(&children).unwrap_or_default());
    view! { <FlexBox focus_id={identity(None)} width=Size::Fill semantics=semantics>{children}</FlexBox> }
}
pub type GroupHeadingProps = GroupProps;
#[component]
pub fn GroupHeading(props: GroupHeadingProps) -> VNode {
    let mut children = props.children.render(());
    let semantics = Semantics::new(Role::Heading, text(&children));
    style_text(&mut children, Color::DarkGrey, false, false);
    view! { <FlexBox focus_id={identity(None)} semantics=semantics padding={Padding::symmetric(1, 0)}>{children}</FlexBox> }
}
#[derive(Props)]
pub struct SeparatorProps {}
#[component]
pub fn Separator(_props: SeparatorProps) -> VNode {
    view! { <FlexBox focus_id={identity(None)} semantics={Semantics::new(Role::Separator, "")}><Paragraph fg=Color::DarkGrey fill='─'>""</Paragraph></FlexBox> }
}
