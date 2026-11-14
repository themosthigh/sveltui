use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crossterm::style::Color;
use std::rc::Rc;
use sveltui::prelude::*;

use super::select_types::*;
use crate::controllable::use_controllable;
use crate::listbox::{Listbox, ListboxOption, Move, PopupLayout, option_row, popup, text_of};

/// Keeps track of the selected value and whether the popup is open, either from
/// the props you pass in or its own state, and shares them through context with
/// the other select parts.
#[component]
pub fn Root(props: RootProps) -> VNode {
    let listbox = Listbox::use_listbox(props.id, props.loop_focus);
    // An empty value means nothing's selected.
    let selection = use_controllable(
        props.value.bound(),
        props.value.value(),
        || props.default_value.clone().unwrap_or_default(),
        props.on_value_change.clone(),
    );
    let open_state = use_controllable(
        props.open.bound(),
        props.open.value(),
        || props.default_open,
        props.on_open_change.clone(),
    );
    listbox.begin_frame();

    let selected = Some(selection.get()).filter(|value| !value.is_empty());
    let open = !props.disabled && open_state.get();

    let set_open: Rc<dyn Fn(bool)> = Rc::new(move |next| {
        if next != open {
            open_state.set(next);
        }
    });
    let select: Rc<dyn Fn(String)> = Rc::new(move |next: String| selection.set(next));

    let context = SelectContext {
        listbox: listbox.clone(),
        selected,
        open,
        disabled: props.disabled,
        set_open,
        select,
    };
    let children = provide_context(context, || props.children.render(()));
    listbox.end_frame();
    children
}

/// The button that shows the current value. Click it or press Enter, Space or
/// an arrow key to open the list; while it's open, keys move the highlight and
/// focus stays here.
#[component]
pub fn Trigger(props: TriggerProps) -> VNode {
    let select = use_context::<SelectContext>().expect("Select::Trigger requires Select::Root");
    let listbox = select.listbox.clone();
    let id = listbox.trigger_id;
    let open = select.open;
    let active = !select.disabled && (open || use_focus(id).is_focused());

    let on_key: KeyHandler = {
        let select = select.clone();
        Rc::new(move |key: KeyEvent, _ctx: &mut EventContext| {
            if key
                .modifiers
                .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT)
            {
                return EventResult::Ignored;
            }
            handle_key(&select, key.code)
        })
    };
    let on_click: Rc<dyn Fn()> = {
        let select = select.clone();
        Rc::new(move || {
            if !select.open {
                select
                    .listbox
                    .highlight_or(select.selected.as_deref(), Move::First);
            }
            (select.set_open)(!select.open);
        })
    };

    let mut semantics = Semantics::new(Role::Combobox, "")
        .expanded(open)
        .controls(listbox.list_id)
        .active_descendant(open.then(|| listbox.active_descendant()).flatten());
    semantics.has_popup = Some(Role::Listbox);
    let (border, border_color) = if active {
        (BorderStyle::Double, Color::White)
    } else {
        (BorderStyle::Single, Color::DarkGrey)
    };
    let arrow = if open { "▴" } else { "▾" };

    view! {
        <Pressable focus_id=id tab_stop=true disabled={select.disabled} semantics=semantics
            on_click=on_click on_key=on_key>
            <View flex_direction=FlexDirection::Row width={props.width} gap=1
                border=border border_color=border_color padding={Padding::symmetric(1, 0)}>
                <View flex_grow=1.0f32>{@render props.children}</View>
                <TextSpan foreground_color=Color::DarkGrey>{arrow}</TextSpan>
            </View>
        </Pressable>
    }
}

fn handle_key(select: &SelectContext, code: KeyCode) -> EventResult {
    let listbox = &select.listbox;
    let selected = select.selected.as_deref();
    if !select.open {
        match code {
            KeyCode::Enter | KeyCode::Char(' ') | KeyCode::Down => {
                listbox.highlight_or(selected, Move::First);
            }
            KeyCode::Up => {
                listbox.highlight_or(selected, Move::Last);
            }
            KeyCode::Char(ch) => {
                listbox.highlight_or(selected, Move::First);
                listbox.typeahead(ch);
            }
            _ => return EventResult::Ignored,
        }
        (select.set_open)(true);
        return EventResult::Handled;
    }
    match code {
        KeyCode::Down => {
            listbox.step(Move::Next);
        }
        KeyCode::Up => {
            listbox.step(Move::Prev);
        }
        KeyCode::Home => {
            listbox.step(Move::First);
        }
        KeyCode::End => {
            listbox.step(Move::Last);
        }
        KeyCode::Enter | KeyCode::Char(' ') => {
            if let Some(value) = listbox.highlighted_choice() {
                (select.select)(value);
            }
            (select.set_open)(false);
        }
        KeyCode::Esc => (select.set_open)(false),
        KeyCode::Tab | KeyCode::BackTab => {
            (select.set_open)(false);
            return EventResult::Ignored;
        }
        KeyCode::Char(ch) => {
            listbox.typeahead(ch);
        }
        _ => return EventResult::Ignored,
    }
    EventResult::Handled
}

/// Shows the selected option's label, or the placeholder if nothing's selected.
#[component]
pub fn Value(props: ValueProps) -> VNode {
    let select = use_context::<SelectContext>().expect("Select::Value requires Select::Root");
    let (text, color) = match &select.selected {
        Some(value) => (
            select
                .listbox
                .label_of(value)
                .unwrap_or_else(|| value.clone()),
            Color::White,
        ),
        None => (props.placeholder, Color::DarkGrey),
    };
    view! { <TextSpan foreground_color=color>{text}</TextSpan> }
}

/// The popup list that holds the items.
#[component]
pub fn Content(props: ContentProps) -> VNode {
    let select = use_context::<SelectContext>().expect("Select::Content requires Select::Root");
    let items = props.children.render(());
    let set_open = select.set_open.clone();
    popup(
        &select.listbox,
        select.open,
        Rc::new(move || set_open(false)),
        PopupLayout {
            width: props.width,
            max_height: props.max_height,
            placement: props.placement,
        },
        items,
    )
}

/// An option in the list. It registers itself with the select every render.
#[component]
pub fn Item(props: ItemProps) -> VNode {
    let select = use_context::<SelectContext>().expect("Select::Item requires Select::Root");
    let has_children = props.children.is_some();
    let rendered = props.children.render(());
    let label = props.label.unwrap_or_else(|| text_of(&rendered));
    let display = if has_children {
        rendered
    } else {
        view! { <TextSpan>{label.clone()}</TextSpan> }
    };
    let option = ListboxOption::new(props.value, label).disabled(props.disabled);
    select.listbox.register(option.clone(), true);
    let selected = select.selected.as_deref() == Some(option.value.as_str());
    let choose: Rc<dyn Fn(String)> = {
        let select = select.clone();
        Rc::new(move |value: String| {
            (select.select)(value);
            (select.set_open)(false);
        })
    };
    option_row(&select.listbox, &option, display, selected, choose)
}

/// A ready-made select built from a list of options.
#[component]
pub fn Select(props: SelectProps) -> VNode {
    let options = props.options;
    let placeholder = props.placeholder;
    let width = props.width;
    Root(
        RootProps::builder()
            .id(props.id)
            .value(props.value.value())
            .bind_value(props.value.bound())
            .default_value(props.default_value)
            .on_value_change(props.on_value_change)
            .disabled(props.disabled)
            .children(Snippet::new(move |_| {
                let options = options.clone();
                view! {
                    <View>
                        <Trigger width=width>
                            <Value placeholder={placeholder.clone()} />
                        </Trigger>
                        <Content width=width>
                            {#each options as option}
                                <Item value={option.value} label={option.label.clone()}
                                    disabled={option.disabled}>{option.label}</Item>
                            {/each}
                        </Content>
                    </View>
                }
            }))
            .build(),
    )
}
