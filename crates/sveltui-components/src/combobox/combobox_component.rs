use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crossterm::style::Color;
use std::{cell::RefCell, rc::Rc};
use sveltui::prelude::*;

use super::combobox_types::*;
use crate::controllable::use_controllable;
use crate::listbox::{
    Listbox, ListboxOption, Move, PopupLayout, option_row, popup, style_label, text_of,
};
use crate::{TextInput, TextInputProps};

/// Keeps track of the selected value, the input's text and whether the
/// popup is open, either from the props you pass in or its own state, and
/// shares them through context with the other combobox parts. Whenever the
/// selection changes, we put its label in the input.
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
    let text = state(String::new());
    let text_on_open = use_hook(|| Rc::new(RefCell::new(String::new())));
    let synced = use_hook(|| Rc::new(RefCell::new(None::<(String, String)>)));
    listbox.begin_frame();

    let selected = Some(selection.get()).filter(|value| !value.is_empty());
    let open = !props.disabled && open_state.get();
    let selected_label = selected
        .as_ref()
        .map(|value| listbox.label_of(value).unwrap_or_else(|| value.clone()));

    // Only sync when the selection or its label changes, so we never stomp
    // on text the user is typing.
    let pair = selected.clone().zip(selected_label.clone());
    if *synced.borrow() != pair {
        if let Some((_, label)) = &pair
            && text.get() != *label
        {
            text.set(label.clone());
        }
        *synced.borrow_mut() = pair;
    }
    let current = text.get();
    let query = if Some(&current) == selected_label.as_ref() {
        String::new()
    } else {
        current
    };

    let set_open: Rc<dyn Fn(bool)> = {
        let text_on_open = text_on_open.clone();
        Rc::new(move |next| {
            if next == open {
                return;
            }
            if next {
                *text_on_open.borrow_mut() = text.get();
            }
            open_state.set(next);
        })
    };
    let choose: Rc<dyn Fn(String)> = {
        let listbox = listbox.clone();
        let set_open = set_open.clone();
        Rc::new(move |next: String| {
            text.set(listbox.label_of(&next).unwrap_or_else(|| next.clone()));
            selection.set(next);
            set_open(false);
        })
    };
    let revert: Rc<dyn Fn()> = {
        let set_open = set_open.clone();
        Rc::new(move || {
            text.set(text_on_open.borrow().clone());
            set_open(false);
        })
    };

    let context = ComboboxContext {
        listbox: listbox.clone(),
        selected,
        open,
        disabled: props.disabled,
        text,
        query,
        filter: props.filter,
        set_open,
        choose,
        revert,
    };
    let children = provide_context(context, || props.children.render(()));
    listbox.end_frame();
    children
}

/// The text input. Typing filters the list and opens it; the arrow keys move
/// the highlight while focus stays in the input, Enter picks the highlighted
/// option and Escape puts the earlier text back.
#[component]
pub fn Input(props: InputProps) -> VNode {
    let combobox =
        use_context::<ComboboxContext>().expect("Combobox::Input requires Combobox::Root");
    let listbox = combobox.listbox.clone();
    let open = combobox.open;

    let mut semantics = Semantics::new(Role::Combobox, props.placeholder.clone())
        .expanded(open)
        .controls(listbox.list_id)
        .active_descendant(open.then(|| listbox.active_descendant()).flatten());
    semantics.has_popup = Some(Role::Listbox);

    let on_key: KeyHandler = {
        let combobox = combobox.clone();
        Rc::new(move |key: KeyEvent, _ctx: &mut EventContext| handle_key(&combobox, key))
    };
    let on_click: Rc<dyn Fn()> = {
        let combobox = combobox.clone();
        Rc::new(move || {
            if !combobox.disabled && !combobox.open {
                (combobox.set_open)(true);
            }
        })
    };

    TextInput(
        TextInputProps::builder()
            .id(listbox.trigger_id)
            .bind_value(combobox.text)
            .placeholder(props.placeholder)
            .width(props.width)
            .semantics(semantics)
            .on_key(on_key)
            .on_click(on_click)
            .build(),
    )
    .into_node()
}

fn handle_key(combobox: &ComboboxContext, key: KeyEvent) -> EventResult {
    let listbox = &combobox.listbox;
    let open = combobox.open;
    if matches!(key.code, KeyCode::Tab | KeyCode::BackTab) {
        if open {
            (combobox.set_open)(false);
        }
        return EventResult::Ignored;
    }
    if combobox.disabled {
        return EventResult::Handled;
    }
    if key.modifiers.contains(KeyModifiers::ALT) {
        match key.code {
            KeyCode::Down if !open => (combobox.set_open)(true),
            KeyCode::Up if open => (combobox.set_open)(false),
            _ => return EventResult::Ignored,
        }
        return EventResult::Handled;
    }
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        return EventResult::Ignored;
    }
    match key.code {
        KeyCode::Down | KeyCode::Up => {
            let (closed_start, step) = if key.code == KeyCode::Down {
                (Move::First, Move::Next)
            } else {
                (Move::Last, Move::Prev)
            };
            if open {
                listbox.step(step);
            } else {
                listbox.highlight(None);
                listbox.step(closed_start);
                (combobox.set_open)(true);
            }
            EventResult::Handled
        }
        KeyCode::Enter if open => {
            match listbox.highlighted_choice() {
                Some(value) => (combobox.choose)(value),
                None => (combobox.set_open)(false),
            }
            EventResult::Handled
        }
        KeyCode::Esc if open => {
            (combobox.revert)();
            EventResult::Handled
        }
        // Editing keys reach the input. We clear the highlight because the
        // filtered list is about to change, and open the list if needed.
        KeyCode::Char(_) | KeyCode::Backspace | KeyCode::Delete => {
            listbox.highlight(None);
            if !open {
                (combobox.set_open)(true);
            }
            EventResult::Ignored
        }
        _ => EventResult::Ignored,
    }
}

/// A small button that opens and closes the list. It isn't a tab stop, so
/// keyboard users stay in the input.
#[component]
pub fn Trigger(_props: TriggerProps) -> VNode {
    let combobox =
        use_context::<ComboboxContext>().expect("Combobox::Trigger requires Combobox::Root");
    let open = combobox.open;
    let on_click: Rc<dyn Fn()> = {
        let combobox = combobox.clone();
        Rc::new(move || {
            if !combobox.disabled {
                (combobox.set_open)(!combobox.open);
            }
        })
    };
    let arrow = if open { " ▴ " } else { " ▾ " };
    let semantics = Semantics::new(Role::Button, "Show options").expanded(open);
    view! {
        <Pressable on_click=on_click semantics=semantics disabled={combobox.disabled}>
            <TextSpan foreground_color=Color::DarkGrey>{arrow}</TextSpan>
        </Pressable>
    }
}

/// The popup list that holds the items.
#[component]
pub fn Content(props: ContentProps) -> VNode {
    let combobox =
        use_context::<ComboboxContext>().expect("Combobox::Content requires Combobox::Root");
    let items = props.children.render(());
    let set_open = combobox.set_open.clone();
    popup(
        &combobox.listbox,
        combobox.open,
        Rc::new(move || set_open(false)),
        PopupLayout {
            width: props.width,
            max_height: props.max_height,
            placement: props.placement,
        },
        items,
    )
}

/// An option in the list. It registers itself every render and hides when
/// its label doesn't contain the typed text.
#[component]
pub fn Item(props: ItemProps) -> VNode {
    let combobox =
        use_context::<ComboboxContext>().expect("Combobox::Item requires Combobox::Root");
    let has_children = props.children.is_some();
    let rendered = props.children.render(());
    let label = props.label.unwrap_or_else(|| text_of(&rendered));
    let visible = !combobox.filter
        || combobox.query.is_empty()
        || label
            .to_lowercase()
            .contains(&combobox.query.to_lowercase());
    let option = ListboxOption::new(props.value, label).disabled(props.disabled);
    combobox.listbox.register(option.clone(), visible);
    if !visible {
        return VNode::Empty;
    }
    let display = if has_children {
        rendered
    } else {
        view! { <TextSpan>{option.label.clone()}</TextSpan> }
    };
    let selected = combobox.selected.as_deref() == Some(option.value.as_str());
    option_row(
        &combobox.listbox,
        &option,
        display,
        selected,
        combobox.choose.clone(),
    )
}

/// Shown when no option matches the typed text. Put it after the items;
/// it counts the options that have registered before it.
#[component]
pub fn Empty(props: EmptyProps) -> VNode {
    let combobox =
        use_context::<ComboboxContext>().expect("Combobox::Empty requires Combobox::Root");
    if combobox.listbox.visible_count() > 0 {
        return VNode::Empty;
    }
    let message = props.children.render(());
    view! {
        <View padding={Padding::symmetric(1, 0)}>
            {style_label(message, Color::DarkGrey, false)}
        </View>
    }
}

/// A ready-made combobox built from a list of options.
#[component]
pub fn Combobox(props: ComboboxProps) -> VNode {
    let options = props.options;
    let placeholder = props.placeholder;
    let width = props.width;
    let empty = props.empty;
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
                        <View flex_direction=FlexDirection::Row align_items=AlignItems::Center>
                            <Input placeholder={placeholder.clone()} width=width />
                            <Trigger />
                        </View>
                        <Content width=width>
                            {#each options as option}
                                <Item value={option.value} label={option.label.clone()}
                                    disabled={option.disabled}>{option.label}</Item>
                            {/each}
                            <Empty>{empty.clone()}</Empty>
                        </Content>
                    </View>
                }
            }))
            .build(),
    )
}
