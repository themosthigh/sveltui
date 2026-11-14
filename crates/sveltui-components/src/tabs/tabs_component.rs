use crossterm::style::Color;
use std::{cell::RefCell, collections::HashMap, rc::Rc};
use sveltui::prelude::*;

use super::tabs_types::*;
use crate::controllable::use_controllable;

/// Keeps track of the selected tab, either the `value` you pass in or its own
/// state, and shares it through context with `List`, `Trigger` and `Content`,
/// along with stable tab and panel ids. Put `List` before the `Content` parts;
/// otherwise the panels render before a default tab has been picked.
#[component]
pub fn Root(props: RootProps) -> VNode {
    // An empty value means nothing's selected yet, so the first enabled tab
    // gets picked while rendering.
    let selection = use_controllable(
        props.value.bound(),
        props.value.value(),
        || props.default_value.clone().unwrap_or_default(),
        props.on_value_change.clone(),
    );
    let ids = use_hook(|| Rc::new(RefCell::new(HashMap::new())));
    let resolved = use_hook(|| Rc::new(RefCell::new(None)));
    *resolved.borrow_mut() = Some(selection.get()).filter(|value| !value.is_empty());

    let current = resolved.clone();
    let select: Rc<dyn Fn(String)> = Rc::new(move |next: String| {
        // The tab picked by default isn't stored anywhere, so compare with
        // what's on screen rather than the stored value.
        if current.borrow().as_deref() != Some(next.as_str()) {
            selection.set(next);
        }
    });

    let context = TabsContext {
        resolved,
        ids,
        select,
        orientation: props.orientation,
        activation: props.activation_mode,
        loop_focus: props.loop_focus,
    };
    let children = provide_context(context, || props.children.render(()));
    match props.id {
        Some(id) => PressableElement::new()
            .focus_id(id)
            .child(children)
            .into_node(),
        None => children,
    }
}

/// A row or column of tabs, depending on the root's orientation. The whole list
/// is one tab stop, and the arrow keys move between tabs.
#[component]
pub fn List(props: ListProps) -> VNode {
    let tabs = use_context::<TabsContext>().expect("Tabs::List requires Tabs::Root");
    let group = FocusGroup {
        orientation: tabs.orientation,
        wrap: tabs.loop_focus,
        typeahead: false,
        disabled_focusable: false,
    };
    let direction = match tabs.orientation {
        Orientation::Horizontal => FlexDirection::Row,
        Orientation::Vertical => FlexDirection::Column,
    };
    let mut list = PressableElement::new()
        .focus_group(group)
        .semantics(Semantics::new(Role::TabList, ""));
    if let Some(id) = props.id {
        list = list.focus_id(id);
    }
    list.child(
        ViewElement::new()
            .flex_direction(direction)
            .gap(1)
            .child(props.children.render(())),
    )
    .into_node()
}

#[component]
pub fn Trigger(props: TriggerProps) -> VNode {
    let tabs = use_context::<TabsContext>().expect("Tabs::Trigger requires Tabs::Root");
    let (default_id, panel_id) = tabs.ids_for(&props.value);
    let id = props.id.unwrap_or(default_id);
    if props.id.is_some() {
        tabs.ids
            .borrow_mut()
            .insert(props.value.clone(), (id, panel_id));
    }

    // Nothing selected yet? The first enabled tab to render claims it.
    if !props.disabled {
        let mut resolved = tabs.resolved.borrow_mut();
        if resolved.is_none() {
            *resolved = Some(props.value.clone());
        }
    }
    let selected = tabs.is_selected(&props.value);
    let focused = !props.disabled && use_focus(id).is_focused();

    let select = tabs.select.clone();
    let value = props.value.clone();
    let on_click: Option<Rc<dyn Fn()>> =
        (!props.disabled).then(|| -> Rc<dyn Fn()> { Rc::new(move || select(value.clone())) });
    let on_focus = on_click
        .clone()
        .filter(|_| tabs.activation == TabsActivation::Automatic);

    let (fg, bg) = if props.disabled {
        (Color::DarkGrey, None)
    } else if focused {
        (Color::Black, Some(Color::White))
    } else if selected {
        (Color::White, None)
    } else {
        (Color::DarkGrey, None)
    };
    let label = props.children.render(());
    let semantics = Semantics::new(Role::Tab, "")
        .selected(selected)
        .controls(panel_id);

    view! {
        <Pressable
            focus_id=id
            tab_stop=true
            disabled={props.disabled}
            activation=Activation::Press
            semantics=semantics
            on_click=on_click
            on_focus=on_focus
        >
            <View flex_direction=FlexDirection::Row background_color=bg>
                <TextSpan>" "</TextSpan>
                <View foreground_color=fg>
                    {styled(label, fg, selected)}
                </View>
                <TextSpan>" "</TextSpan>
            </View>
        </Pressable>
    }
}

/// A tab panel. It only renders while its tab is selected, and it's labelled by
/// that tab using the ids the root keeps.
#[component]
pub fn Content(props: ContentProps) -> VNode {
    let tabs = use_context::<TabsContext>().expect("Tabs::Content requires Tabs::Root");
    if !tabs.is_selected(&props.value) {
        return VNode::Empty;
    }
    let (tab_id, id) = tabs.ids_for(&props.value);
    let semantics = Semantics::new(Role::TabPanel, "").labelled_by(tab_id);
    let children = props.children.render(());
    // The panel only takes a tab stop when there's nothing focusable inside it;
    // otherwise Tab goes straight to its content.
    let tab_stop = !has_tab_stop(&children);
    view! {
        <Pressable focus_id=id tab_stop=tab_stop semantics=semantics>
            <View>{children}</View>
        </Pressable>
    }
}

fn has_tab_stop(node: &VNode) -> bool {
    match node {
        VNode::View(v) => {
            v.pressables.iter().any(|p| p.tab_stop) || v.children.iter().any(has_tab_stop)
        }
        VNode::Fragment(items) => items.iter().any(has_tab_stop),
        VNode::Overlay(o) => o.trigger.as_deref().is_some_and(has_tab_stop),
        _ => false,
    }
}

/// Colors any text in `node` that doesn't have its own color, and makes it bold
/// and underlined when `selected`.
fn styled(mut node: VNode, fg: Color, selected: bool) -> VNode {
    fn walk(node: &mut VNode, fg: Color, selected: bool) {
        match node {
            VNode::Text(text) => {
                if text.fg.is_none() {
                    text.fg = Some(fg);
                }
                text.bold |= selected;
                text.underline |= selected;
            }
            VNode::View(view) => view.children.iter_mut().for_each(|c| walk(c, fg, selected)),
            VNode::Fragment(items) => items.iter_mut().for_each(|c| walk(c, fg, selected)),
            _ => {}
        }
    }
    walk(&mut node, fg, selected);
    node
}
