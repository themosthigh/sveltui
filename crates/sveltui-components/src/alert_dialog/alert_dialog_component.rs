use crossterm::event::{KeyCode, KeyEvent};
use crossterm::style::Color;
use std::cell::RefCell;
use std::rc::Rc;
use sveltui::Runtime;
use sveltui::prelude::*;

use super::alert_dialog_types::*;
use crate::controllable::use_controllable;
use crate::{Button, ButtonProps};

fn context(part: &str) -> AlertDialogContext {
    use_context::<AlertDialogContext>()
        .unwrap_or_else(|| panic!("AlertDialog::{part} requires AlertDialog::Root"))
}

/// Holds whether the dialog is open and shares it, with the ids the parts
/// point at each other with, through context.
#[component]
pub fn Root(props: RootProps) -> VNode {
    let open = use_controllable(
        props.open.bound(),
        props.open.value(),
        || props.default_open,
        props.on_open_change.clone(),
    );
    let ids = use_hook(|| Ids {
        trigger: FocusId::new(),
        layer: FocusId::new(),
        title: FocusId::new(),
        description: FocusId::new(),
        cancel: FocusId::new(),
    });
    let cancel = use_hook(|| Rc::new(RefCell::new(None)));
    // Cancel registers itself while rendering; start each frame without one.
    cancel.borrow_mut().take();
    let context = AlertDialogContext {
        open,
        ids,
        cancel,
        close_on_escape: props.close_on_escape,
        outside_press: props.outside_press,
    };
    provide_context(context, || props.children.render(()))
}

/// A button that opens the dialog. Focus comes back to it when the dialog
/// closes.
#[component]
pub fn Trigger(props: TriggerProps) -> VNode {
    let dialog = context("Trigger");
    let open = dialog.open.clone();
    let on_click: Rc<dyn Fn()> = Rc::new(move || open.set(true));
    button(
        Some(dialog.ids.trigger),
        props.label,
        props.children,
        on_click,
    )
}

/// The dialog itself: a modal layer over a backdrop. Focus starts on Cancel
/// and stays inside until the dialog closes.
#[component]
pub fn Content(props: ContentProps) -> VNode {
    let dialog = context("Content");
    // The parts register while rendering, so render them before reading
    // what Cancel registered.
    let body = props.children.render(());

    let mut layer = DismissLayer::dialog();
    layer.scope.initial_focus = InitialFocus::Target(dialog.ids.cancel);
    layer.dismiss_on_outside_press = dialog.outside_press == OutsidePress::Close;
    let semantics = Semantics {
        modal: Some(true),
        ..Semantics::new(Role::AlertDialog, "")
            .labelled_by(dialog.ids.title)
            .described_by(dialog.ids.description)
    };
    let on_outside_press: Option<Rc<dyn Fn()>> = (dialog.outside_press == OutsidePress::Bell)
        .then(|| -> Rc<dyn Fn()> { Rc::new(Runtime::bell) });
    let on_key = {
        let dialog = dialog.clone();
        move |key: KeyEvent, _: &mut EventContext| {
            if key.code != KeyCode::Esc {
                return EventResult::Ignored;
            }
            // Handled either way, so a dialog that can't be escaped doesn't
            // fall through to the layer's own Escape handling.
            if dialog.close_on_escape {
                dialog.cancel();
            }
            EventResult::Handled
        }
    };
    let on_close = {
        let dialog = dialog.clone();
        move || dialog.cancel()
    };
    view! {
        <Overlay id={dialog.ids.layer} open={dialog.open.get()} placement={props.placement}
            layer=layer backdrop={props.backdrop} modal=true opener={dialog.ids.trigger}
            on_close=on_close on_key=on_key on_outside_press=on_outside_press
            semantics=semantics>
            <View border=BorderStyle::Single border_color=Color::White width={props.width}
                padding={Padding::all(1)} gap=1>
                {body}
            </View>
        </Overlay>
    }
    .into_node()
}

/// The question the dialog asks. The dialog is labelled by it.
#[component]
pub fn Title(props: TitleProps) -> VNode {
    let dialog = context("Title");
    view! {
        <Pressable focus_id={dialog.ids.title} semantics={Semantics::new(Role::Heading, "")}>
            <View>
                <TextSpan bold=true>{props.children.render(())}</TextSpan>
            </View>
        </Pressable>
    }
    .into_node()
}

/// What answering means. The dialog is described by it.
#[component]
pub fn Description(props: DescriptionProps) -> VNode {
    let dialog = context("Description");
    view! {
        <Pressable focus_id={dialog.ids.description} semantics={Semantics::new(Role::Generic, "")}>
            <View foreground_color=Color::DarkGrey>
                {props.children.render(())}
            </View>
        </Pressable>
    }
    .into_node()
}

/// The safe answer. Focus starts here, and Escape (and a press outside with
/// `OutsidePress::Close`) does what it does.
#[component]
pub fn Cancel(props: CancelProps) -> VNode {
    let dialog = context("Cancel");
    let handler: Rc<dyn Fn()> = {
        let open = dialog.open.clone();
        let on_click = props.on_click.clone();
        Rc::new(move || {
            if let Some(on_click) = &on_click {
                on_click();
            }
            open.set(false);
        })
    };
    *dialog.cancel.borrow_mut() = Some(handler.clone());
    button(
        Some(dialog.ids.cancel),
        props.label,
        props.children,
        handler,
    )
}

/// The answer that does the thing. It runs `on_click`, then closes.
#[component]
pub fn Action(props: ActionProps) -> VNode {
    let dialog = context("Action");
    let open = dialog.open.clone();
    let on_click = props.on_click.clone();
    let handler: Rc<dyn Fn()> = Rc::new(move || {
        if let Some(on_click) = &on_click {
            on_click();
        }
        open.set(false);
    });
    button(None, props.label, props.children, handler)
}

/// A `Button` showing `children` when there are any and `label` otherwise.
/// An empty children snippet would hide the label, so it's only passed on
/// when it's there.
fn button(
    id: Option<FocusId>,
    label: Option<String>,
    children: Option<Snippet>,
    on_click: Rc<dyn Fn()>,
) -> VNode {
    let id = id.unwrap_or_else(|| use_hook(FocusId::new));
    match children {
        Some(children) => view! {
            <Button id=id on_click=on_click>{children.render(())}</Button>
        }
        .into_node(),
        None => view! { <Button id=id label=label on_click=on_click /> }.into_node(),
    }
}
