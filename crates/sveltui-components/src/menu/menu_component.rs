use crossterm::event::{KeyCode, KeyModifiers};
use std::{
    cell::{Cell, RefCell},
    collections::HashSet,
    rc::Rc,
};
use sveltui::prelude::*;

use super::menu_types::*;
use crate::controllable::use_controllable;

fn identity(explicit: Option<FocusId>) -> FocusId {
    let stable = state_with(FocusId::new);
    explicit.unwrap_or_else(|| stable.get())
}

fn root(props: RootProps, kind: MenuTriggerKind) -> VNode {
    let open_state = use_controllable(
        props.open.bound(),
        props.open.value(),
        || props.default_open,
        props.on_open_change.clone(),
    );
    let open = open_state.get();
    let change = Rc::new(move |next| open_state.set(next));
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

#[component]
pub fn BarRoot(props: BarRootProps) -> VNode {
    let selection = use_controllable(
        props.value.bound(),
        props.value.value(),
        || props.default_value.clone(),
        props.on_value_change.clone(),
    );
    let value = selection.get();
    let change = Rc::new(move |next: Option<String>| selection.set(next));
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
    PressableElement::new()
        .focus_id(identity(props.id))
        .focus_group(group)
        .semantics(Semantics::new(Role::MenuBar, ""))
        .child(
            ViewElement::new()
                .flex_direction(FlexDirection::Row)
                .child(children),
        )
        .into_node()
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
        VNode::View(v) => {
            for child in &mut v.children {
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
        VNode::View(v) => v.children.iter().map(text).collect::<Vec<_>>().join(""),
        VNode::Fragment(items) => items.iter().map(text).collect::<Vec<_>>().join(""),
        _ => String::new(),
    }
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
    let inner_view = if kind == MenuTriggerKind::Submenu {
        let sub = ViewElement::new()
            .flex_grow(1.0)
            .flex_basis(Dimension::Length(0.0))
            .overflow_x(Overflow::Hidden)
            .padding(Padding::only(0, 0, 0, 1))
            .child(children);
        ViewElement::new()
            .flex_direction(FlexDirection::Row)
            .background_color(bg)
            .foreground_color(fg)
            .child(sub)
            .child(TextElement::new("› ").fg(fg))
    } else {
        ViewElement::new()
            .background_color(bg)
            .foreground_color(fg)
            .padding(Padding::symmetric(1, 0))
            .child(children)
    };

    PressableElement::new()
        .focus_id(id)
        .interaction(interaction)
        .on_key(move |key, ctx: &mut EventContext| on_key(key, ctx))
        .child(inner_view)
        .into_node()
}

pub use Trigger as SubTrigger;

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
            <View width={props.width} border={props.border} border_color={props.border_color.unwrap_or(props.bg)}
                background_color={props.bg} padding={props.padding} overflow=Overflow::Scroll scroll_id=scroll_id>
                {@render props.children}
            </View>
        });
    }
    overlay.into_node()
}

pub use Content as SubContent;

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
    let v = ViewElement::new()
        .height(1)
        .foreground_color(fg)
        .background_color(active.then_some(props.focus_bg.unwrap_or(Color::White)))
        .padding(Padding::symmetric(1, 0))
        .child(children);

    PressableElement::new()
        .focus_id(id)
        .interaction(interaction)
        .on_key(on_key)
        .child(v)
        .into_node()
}

#[component]
pub fn Group(props: GroupProps) -> VNode {
    fn heading(node: &VNode) -> Option<String> {
        match node {
            VNode::View(v)
                if v.pressables.iter().any(|p| {
                    p.semantics
                        .as_ref()
                        .is_some_and(|s| s.role == Role::Heading)
                }) =>
            {
                Some(text(node))
            }
            VNode::Fragment(items) => items.iter().find_map(heading),
            _ => None,
        }
    }
    let children = props.children.render(());
    let semantics = Semantics::new(Role::Group, heading(&children).unwrap_or_default());
    view! { <Pressable focus_id={identity(None)} semantics=semantics><View>{children}</View></Pressable> }
}

#[component]
pub fn GroupHeading(props: GroupHeadingProps) -> VNode {
    let mut children = props.children.render(());
    let semantics = Semantics::new(Role::Heading, text(&children));
    style_text(&mut children, Color::DarkGrey, false, false);
    view! { <Pressable focus_id={identity(None)} semantics=semantics><View padding={Padding::symmetric(1, 0)}>{children}</View></Pressable> }
}

#[component]
pub fn Separator(_props: SeparatorProps) -> VNode {
    view! { <Pressable focus_id={identity(None)} semantics={Semantics::new(Role::Separator, "")}><Paragraph foreground_color=Color::DarkGrey fill='─'>""</Paragraph></Pressable> }
}
