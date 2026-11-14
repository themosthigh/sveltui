use sveltui::prelude::*;

use super::popover_types::*;
use std::rc::Rc;

#[component]
pub fn Popover(props: PopoverProps) -> impl IntoNode {
    let id = props.id.unwrap_or_default();
    let managed = !props.open.is_set();
    let bound = props.open.bound();
    let open = bound
        .as_ref()
        .map(|state| state.get())
        .or(props.open.value())
        .unwrap_or_else(|| is_layer_open(id));
    let on_close: Rc<dyn Fn()> = {
        let on_close = props.on_close.clone();
        let bound = bound.clone();
        Rc::new(move || {
            if let Some(state) = &bound {
                state.set(false);
            }
            if let Some(callback) = &on_close {
                callback();
            }
        })
    };
    let on_open: Rc<dyn Fn()> = {
        let on_open = props.on_open.clone();
        let bound = bound.clone();
        Rc::new(move || {
            if let Some(state) = &bound {
                state.set(true);
            }
            if let Some(callback) = &on_open {
                callback();
            }
        })
    };

    view! {
        <Overlay
            id=id
            open=open
            managed=managed
            placement={props.placement}
            backdrop={props.backdrop}
            modal=false
            on_close=on_close
            on_open=on_open
            on_key={props.on_key}
            slot_trigger={props.trigger}
        >
            {#if props.children.is_some()}
                {@render props.children}
            {/if}
        </Overlay>
    }
}
