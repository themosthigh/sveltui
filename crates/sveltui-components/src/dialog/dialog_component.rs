use sveltui::prelude::*;

use super::dialog_types::*;
use crate::controllable::use_controllable;
use std::rc::Rc;

#[component]
pub fn Dialog(props: DialogProps) -> impl IntoNode {
    let id = props.id.unwrap_or_default();
    let open_state = use_controllable(
        props.open.bound(),
        props.open.value(),
        || props.default_open,
        props.on_open_change.clone(),
    );
    let open = open_state.get();
    let on_close: Rc<dyn Fn()> = {
        let open_state = open_state.clone();
        let on_close = props.on_close.clone();
        Rc::new(move || {
            open_state.set(false);
            if let Some(callback) = &on_close {
                callback();
            }
        })
    };
    let on_open: Rc<dyn Fn()> = {
        let on_open = props.on_open.clone();
        Rc::new(move || {
            open_state.set(true);
            if let Some(callback) = &on_open {
                callback();
            }
        })
    };
    let mut layer = DismissLayer::dialog();
    layer.scope.initial_focus = props.initial_focus;

    let title_str = props.title.unwrap_or_default();
    let semantics = Semantics {
        modal: Some(true),
        ..Semantics::new(Role::Dialog, title_str.clone())
    };

    view! {
        <Overlay
            id=id
            open=open
            placement={props.placement}
            layer=layer
            backdrop={props.backdrop}
            modal=true
            opener={props.opener}
            on_close=on_close
            on_open=on_open
            on_key={props.on_key}
            semantics=semantics
        >
            <View
                border={props.border}
                border_color={props.border_color}
                width={props.width}
                height={props.height}
                padding={props.padding}
                title={(!title_str.is_empty()).then_some(title_str)}
            >
                {#if props.children.is_some()}
                    {@render props.children}
                {/if}
            </View>
        </Overlay>
    }
}
