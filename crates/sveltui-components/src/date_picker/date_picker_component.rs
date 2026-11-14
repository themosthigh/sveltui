use crossterm::style::Color;
use std::rc::Rc;
use sveltui::prelude::*;

use super::date_picker_types::*;
use crate::calendar::{Calendar, CalendarProps};
use crate::controllable::use_controllable;
use crate::date::Date;
use crate::date_field::{DateField, DateFieldProps};
use crate::inline_button::{Press, inline_button_with};

/// A date field with a button that opens a calendar. Opening puts focus on
/// the picked date (else today); picking a day closes the calendar, and Esc
/// or a press outside closes it without a change. Either way focus goes
/// back to the button. Alt+Down in the field opens it too.
#[component]
pub fn DatePicker(props: DatePickerProps) -> VNode {
    let trigger = use_hook(FocusId::new);
    let grid = use_hook(FocusId::new);
    let popup = use_hook(FocusId::new);

    let value = use_controllable(
        props.value.bound(),
        props.value.value(),
        || props.default_value,
        props.on_value_change.clone(),
    );
    let open = use_controllable(
        props.open.bound(),
        props.open.value(),
        || props.default_open,
        props.on_open_change.clone(),
    );
    let is_open = open.get() && !props.disabled;

    // The field and the calendar both bind to one state. Changes they make
    // go on to `value`, which may be read-only, so the state is reset from
    // it every render.
    let shared = state_with(|| value.get());
    if shared.get() != value.get() {
        shared.set(value.get());
    }
    let changed: Rc<dyn Fn(Option<Date>)> = {
        let value = value.clone();
        Rc::new(move |next| value.set(next))
    };

    let set_open = {
        let open = open.clone();
        Rc::new(move |next: bool| open.set(next))
    };
    let toggle = {
        let set_open = set_open.clone();
        Rc::new(move || set_open(!is_open))
    };
    let press = Press {
        by_key: {
            let toggle = toggle.clone();
            Rc::new(move |_| toggle())
        },
        by_click: toggle,
    };
    let mut semantics = Semantics::new(Role::Button, "Choose date").expanded(is_open);
    semantics.has_popup = Some(Role::Dialog);
    let button = inline_button_with(trigger, semantics, "▾".to_owned(), None, press);

    let open_now: Rc<dyn Fn()> = {
        let set_open = set_open.clone();
        Rc::new(move || set_open(true))
    };
    let field = view! {
        <View flex_direction=FlexDirection::Row gap=1>
            <DateField id={props.id} bind:value={shared} on_value_change={changed.clone()}
                min={props.min} max={props.max} is_date_unavailable={props.is_date_unavailable.clone()}
                order={props.order} label={props.label.clone()} disabled={props.disabled}
                on_open=open_now />
            {button}
        </View>
    };

    let mut layer = DismissLayer::dialog();
    layer.scope.initial_focus = InitialFocus::Target(grid);
    let close: Rc<dyn Fn()> = {
        let set_open = set_open.clone();
        Rc::new(move || set_open(false))
    };
    let picked: Rc<dyn Fn(Date)> = Rc::new(move |_| set_open(false));
    let semantics = Semantics {
        modal: Some(true),
        ..Semantics::new(Role::Dialog, "Choose date")
    };
    view! {
        <Overlay id=popup open=is_open layer=layer modal=true opener=trigger
            placement=Placement::Bottom on_close=close semantics=semantics trigger=field>
            <View border=BorderStyle::Rounded border_color=Color::DarkGrey>
                <Calendar id=grid bind:value={shared} on_value_change=changed min={props.min}
                    max={props.max} is_date_unavailable={props.is_date_unavailable}
                    week_starts_on={props.week_starts_on} label={props.label}
                    on_select=picked />
            </View>
        </Overlay>
    }
    .into_node()
}
