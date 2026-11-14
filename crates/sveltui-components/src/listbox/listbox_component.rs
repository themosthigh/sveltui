use crossterm::style::Color;
use std::rc::Rc;
use sveltui::element::{get_scroll, set_scroll};
use sveltui::prelude::*;

use super::listbox_types::*;
use crate::{Divider, DividerProps};

/// How big the listbox popup is and where it goes.
pub(crate) struct PopupLayout {
    pub(crate) width: Dimension,
    /// Height including the border. Longer lists scroll.
    pub(crate) max_height: u16,
    pub(crate) placement: Placement,
}

/// The dropdown list, anchored to the trigger. It only shows `items` while
/// `open`, but render them every time anyway; that's how the options register
/// while it's closed.
pub(crate) fn popup(
    listbox: &Listbox,
    open: bool,
    close: Rc<dyn Fn()>,
    layout: PopupLayout,
    items: VNode,
) -> VNode {
    let mut overlay = OverlayElement::new()
        .id(listbox.list_id)
        .opener(listbox.trigger_id)
        .anchor(listbox.trigger_id)
        .open(open)
        .layer(listbox_layer())
        .placement(layout.placement)
        .semantics(Semantics::new(Role::Listbox, ""))
        .on_close(move || close());
    if open {
        let rows = layout.max_height.saturating_sub(2).max(1);
        keep_highlight_visible(listbox, rows);
        overlay = overlay.child(view! {
            <View width={layout.width} max_height={layout.max_height}
                border=BorderStyle::Single border_color=Color::DarkGrey
                overflow_y=Overflow::Scroll scroll_id={listbox.scroll_id}>
                {items}
            </View>
        });
    }
    overlay.into_node()
}

/// Scrolls the list so the highlighted option is within `rows`. We count one
/// row per visible option, so headings and separators can throw it off by a row
/// or two.
fn keep_highlight_visible(listbox: &Listbox, rows: u16) {
    let Some(value) = listbox.highlighted() else {
        return;
    };
    let Some(index) = listbox
        .entries()
        .iter()
        .filter(|e| e.visible)
        .position(|e| e.option.value == value)
    else {
        return;
    };
    let index = index as u16;
    let (x, top) = get_scroll(listbox.scroll_id);
    let top = if index < top {
        index
    } else if index >= top + rows {
        index + 1 - rows
    } else {
        top
    };
    set_scroll(listbox.scroll_id, (x, top));
}

/// One option row. Clicking it calls `choose`, and hovering highlights it.
pub(crate) fn option_row(
    listbox: &Listbox,
    option: &ListboxOption,
    label: VNode,
    selected: bool,
    choose: Rc<dyn Fn(String)>,
) -> VNode {
    let id = listbox.option_id(&option.value);
    let highlighted = listbox.highlighted().as_deref() == Some(option.value.as_str());
    let (fg, bg) = if option.disabled {
        (Color::DarkGrey, None)
    } else if highlighted {
        (Color::Black, Some(Color::White))
    } else {
        (Color::White, None)
    };
    let marker = if selected { "✓ " } else { "  " };
    let on_click = (!option.disabled).then(|| {
        let value = option.value.clone();
        Rc::new(move || choose(value.clone())) as Rc<dyn Fn()>
    });
    let on_enter = (!option.disabled).then(|| {
        let value = option.value.clone();
        let listbox = listbox.clone();
        Rc::new(move || listbox.highlight(Some(value.clone()))) as Rc<dyn Fn()>
    });
    let semantics = Semantics::new(Role::Option, option.label.clone()).selected(selected);
    view! {
        <Pressable focus_id=id disabled={option.disabled} semantics=semantics
            on_click=on_click on_pointer_enter=on_enter>
            <View flex_direction=FlexDirection::Row width={Dimension::Percent(1.0)}
                background_color=bg padding={Padding::symmetric(1, 0)}>
                <TextSpan foreground_color=fg>{marker}</TextSpan>
                {style_label(label, fg, highlighted)}
            </View>
        </Pressable>
    }
}

/// Colors any text in `node` that doesn't have its own color.
pub(crate) fn style_label(mut node: VNode, fg: Color, bold: bool) -> VNode {
    fn walk(node: &mut VNode, fg: Color, bold: bool) {
        match node {
            VNode::Text(text) => {
                if text.fg.is_none() {
                    text.fg = Some(fg);
                }
                text.bold |= bold;
            }
            VNode::View(view) => view.children.iter_mut().for_each(|c| walk(c, fg, bold)),
            VNode::Fragment(items) => items.iter_mut().for_each(|c| walk(c, fg, bold)),
            _ => {}
        }
    }
    walk(&mut node, fg, bold);
    node
}

#[derive(Props)]
pub struct GroupProps {
    #[snippet(optional)]
    pub children: Option<Snippet>,
}

/// Groups related options. Start it with a `GroupHeading` to label them.
#[component]
pub fn Group(props: GroupProps) -> VNode {
    let semantics = Semantics::new(Role::Group, "");
    view! {
        <Pressable semantics=semantics>
            <View>{@render props.children}</View>
        </Pressable>
    }
}

#[derive(Props)]
pub struct GroupHeadingProps {
    #[snippet(optional)]
    pub children: Option<Snippet>,
}

#[component]
pub fn GroupHeading(props: GroupHeadingProps) -> VNode {
    let label = props.children.render(());
    view! {
        <View padding={Padding::symmetric(1, 0)}>
            {style_label(label, Color::DarkGrey, true)}
        </View>
    }
}

#[derive(Props)]
pub struct SeparatorProps {}

#[component]
pub fn Separator(_props: SeparatorProps) -> VNode {
    view! { <Divider /> }
}
