use crossterm::event::{KeyCode, KeyEvent};
use crossterm::style::Color;
use std::cell::Cell;
use std::rc::Rc;
use sveltui::Runtime;
use sveltui::prelude::*;

use super::toast_manager::ToastManager;
use super::toast_types::*;
use crate::inline_button::{Press, inline_button};

/// What the provider shares with the viewport and the toasts.
#[derive(Clone)]
struct ToastContext {
    manager: ToastManager,
    /// Whether the pointer is over the viewport.
    hovered: Rc<Cell<bool>>,
}

/// The provider's manager, for adding toasts from event handlers.
///
/// # Panics
///
/// Outside a `Toast::Provider`.
pub fn use_toast_manager() -> ToastManager {
    use_context::<ToastContext>()
        .expect("use_toast_manager requires Toast::Provider")
        .manager
}

/// Holds the toasts and runs their countdowns. Wrap the whole app in it, and
/// put a `Toast::Viewport` inside. The hotkey (F8 by default) moves focus to
/// the newest toast from anywhere inside.
#[component]
pub fn Provider(props: ProviderProps) -> VNode {
    let own = use_hook(ToastManager::new);
    let manager = props.manager.clone().unwrap_or(own);
    let hovered = use_hook(|| Rc::new(Cell::new(false)));
    let id = use_hook(FocusId::new);
    manager.configure(props.timeout, props.limit);
    manager.tick();

    let hotkey = props.hotkey;
    let on_key = {
        let manager = manager.clone();
        move |key: KeyEvent, ctx: &mut EventContext| {
            if key.code != hotkey {
                return EventResult::Ignored;
            }
            match manager.list().first() {
                Some(newest) => {
                    manager.set_return_focus(ctx.focused());
                    ctx.focus(newest.focus_id);
                    EventResult::Handled
                }
                None => EventResult::Ignored,
            }
        }
    };
    let context = ToastContext { manager, hovered };
    let children = provide_context(context, || props.children.render(()));
    view! {
        <Pressable focus_id=id on_key=on_key>
            <View width={Dimension::Percent(1.0)} height={Dimension::Percent(1.0)}>
                {children}
            </View>
        </Pressable>
    }
    .into_node()
}

/// Where the toasts show: pinned to a corner of the provider, on top of
/// everything else in it. Hovering it, or moving focus into it, pauses
/// every countdown until the pointer and focus leave.
#[component]
pub fn Viewport(props: ViewportProps) -> VNode {
    let toasts = use_context::<ToastContext>().expect("Toast::Viewport requires Toast::Provider");
    let id = use_hook(FocusId::new);
    let focus_within = use_focus(id).has_focus_within();
    toasts
        .manager
        .set_paused(toasts.hovered.get() || focus_within);

    let hovered = toasts.hovered.clone();
    let on_hover = move |inside: bool| {
        hovered.set(inside);
        Runtime::set_dirty(true);
    };
    let bottom = matches!(
        props.placement,
        Placement::Bottom | Placement::BottomStart | Placement::BottomEnd
    );
    let start = matches!(
        props.placement,
        Placement::TopStart | Placement::BottomStart
    );
    let inset = match (bottom, start) {
        (true, true) => Inset::new().bottom(0).left(0),
        (true, false) => Inset::new().bottom(0).right(0),
        (false, true) => Inset::new().top(0).left(0),
        (false, false) => Inset::new().top(0).right(0),
    };
    // The list is newest first, so at the bottom it runs upwards to keep the
    // newest toast next to the edge.
    let direction = if bottom {
        FlexDirection::ColumnReverse
    } else {
        FlexDirection::Column
    };
    view! {
        <Pressable focus_id=id on_hover=on_hover
            semantics={Semantics::new(Role::Region, props.label.clone())}>
            <View position=Position::Absolute inset=inset width={props.width}
                flex_direction=direction>
                {props.children.render(())}
            </View>
        </Pressable>
    }
    .into_node()
}

/// One toast. With no children it shows the title, description, action and
/// a close button; pass children to lay those parts out yourself. It's a
/// tab stop, and Escape closes it.
#[component]
pub fn Root(props: RootProps) -> VNode {
    let toasts = use_context::<ToastContext>().expect("Toast::Root requires Toast::Provider");
    let toast = props.toast;
    let focus = use_focus(toast.focus_id);
    let active = focus.is_active();

    let (role, live) = match toast.priority() {
        ToastPriority::Low => (Role::Status, Live::Polite),
        ToastPriority::High => (Role::Alert, Live::Assertive),
    };
    let label = match toast.description() {
        Some(description) => format!("{}. {description}", toast.title()),
        None => toast.title().to_owned(),
    };
    let semantics = Semantics::new(role, label).live(live).atomic(true);

    let on_key = {
        let manager = toasts.manager.clone();
        let id = toast.id;
        move |key: KeyEvent, ctx: &mut EventContext| {
            if key.code == KeyCode::Esc {
                manager.close_from_key(id, ctx);
                EventResult::Handled
            } else {
                EventResult::Ignored
            }
        }
    };
    let context = RootContext {
        toast: toast.clone(),
        manager: toasts.manager,
    };
    let body = provide_context(context, || {
        if props.children.is_some() {
            props.children.render(())
        } else {
            view! {
                <View flex_direction=FlexDirection::Row gap=1>
                    <View flex_grow=1.0>
                        <Title />
                        <Description />
                    </View>
                    <Action />
                    <Close />
                </View>
            }
            .into_node()
        }
    });
    let (border, color) = if active {
        (BorderStyle::Double, Color::White)
    } else {
        (BorderStyle::Single, Color::DarkGrey)
    };
    view! {
        <Pressable focus_id={toast.focus_id} tab_stop=true semantics=semantics on_key=on_key>
            <View border=border border_color=color padding={Padding::only(0, 1, 0, 1)}>
                {body}
            </View>
        </Pressable>
    }
    .into_node()
}

fn root_context(part: &str) -> RootContext {
    use_context::<RootContext>().unwrap_or_else(|| panic!("Toast::{part} requires Toast::Root"))
}

/// The toast's title.
#[component]
pub fn Title(props: TitleProps) -> VNode {
    let toast = root_context("Title").toast;
    if props.children.is_some() {
        return props.children.render(());
    }
    view! {
        <TextSpan bold=true>{toast.title().to_owned()}</TextSpan>
    }
    .into_node()
}

/// The toast's description, if it has one.
#[component]
pub fn Description(props: DescriptionProps) -> VNode {
    let toast = root_context("Description").toast;
    if props.children.is_some() {
        return props.children.render(());
    }
    match toast.description() {
        Some(text) => view! {
            <TextSpan foreground_color=Color::DarkGrey>{text.to_owned()}</TextSpan>
        }
        .into_node(),
        None => VNode::Empty,
    }
}

/// The toast's action button, if it has one. Pressing it runs the action,
/// then closes the toast.
#[component]
pub fn Action(props: ActionProps) -> VNode {
    let RootContext { toast, manager } = root_context("Action");
    let id = use_hook(FocusId::new);
    let Some(action) = toast.action().cloned() else {
        return VNode::Empty;
    };
    let run = action.on_press.clone();
    let key_manager = manager.clone();
    let press = Press {
        by_key: Rc::new(move |ctx| {
            run();
            key_manager.close_from_key(toast.id, ctx);
        }),
        by_click: Rc::new(move || {
            (action.on_press)();
            manager.close(toast.id);
        }),
    };
    let content = props.children.is_some().then(|| props.children.render(()));
    inline_button(id, action.label.clone(), action.label, content, press)
}

/// Closes the toast.
#[component]
pub fn Close(props: CloseProps) -> VNode {
    let RootContext { toast, manager } = root_context("Close");
    let id = use_hook(FocusId::new);
    let key_manager = manager.clone();
    let press = Press {
        by_key: Rc::new(move |ctx| key_manager.close_from_key(toast.id, ctx)),
        by_click: Rc::new(move || manager.close(toast.id)),
    };
    let content = props.children.is_some().then(|| props.children.render(()));
    inline_button(id, props.label, "×".to_owned(), content, press)
}
