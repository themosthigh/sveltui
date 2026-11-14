use crossterm::event::{KeyCode, KeyEvent};
use crossterm::style::Color;
use sveltui::prelude::*;

use super::hover_popup_types::HoverPopup;

/// Escape closes the popup while it's open, and is left for others
/// otherwise.
fn escape_closes(popup: HoverPopup) -> impl Fn(KeyEvent, &mut EventContext) -> EventResult {
    move |key, _| {
        if key.code == KeyCode::Esc && popup.open.get() {
            popup.dismiss();
            EventResult::Handled
        } else {
            EventResult::Ignored
        }
    }
}

/// What the popup is attached to. Hovering it, or focus anywhere inside it,
/// opens the popup.
pub(crate) fn trigger(
    popup: &HoverPopup,
    tab_stop: bool,
    semantics: Option<Semantics>,
    children: VNode,
) -> VNode {
    view! {
        <Pressable focus_id={popup.ids.trigger} tab_stop=tab_stop semantics=semantics
            on_hover={popup.on_trigger_hover()} on_key={escape_closes(popup.clone())}>
            {children}
        </Pressable>
    }
    .into_node()
}

/// The popup, next to the trigger. It never takes focus, and presses
/// outside it reach whatever is underneath.
pub(crate) fn content(
    popup: &HoverPopup,
    placement: Placement,
    semantics: Semantics,
    width: Dimension,
    padding: Padding,
    children: VNode,
) -> VNode {
    let on_close = {
        let popup = popup.clone();
        move || popup.dismiss()
    };
    view! {
        <Overlay id={popup.ids.layer} anchor={popup.ids.trigger} open={popup.open.get()}
            placement=placement layer={DismissLayer::tooltip()} backdrop=false
            on_close=on_close on_key={escape_closes(popup.clone())}>
            <Pressable focus_id={popup.ids.content} semantics=semantics
                on_hover={popup.on_content_hover()}>
                <View border=BorderStyle::Single border_color=Color::DarkGrey width=width
                    padding=padding>
                    {children}
                </View>
            </Pressable>
        </Overlay>
    }
    .into_node()
}
