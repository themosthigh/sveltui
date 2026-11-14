use crossterm::event::{KeyCode, KeyEvent};
use crossterm::style::Color;
use std::rc::Rc;
use sveltui::prelude::*;

use super::text_input_types::*;
use crate::controllable::use_controllable;

pub fn get_cursor(id: FocusId, text_len: usize) -> usize {
    with_buffer_mut(id, |buf| {
        if buf.cursor_col > text_len {
            buf.cursor_col = text_len;
        }
        buf.cursor_col
    })
}

pub fn set_cursor(id: FocusId, pos: usize) {
    with_buffer_mut(id, |buf| {
        buf.cursor_col = pos;
    });
}

#[component]
pub fn TextInput(props: TextInputProps) -> impl IntoNode {
    let id = props.id.unwrap_or_default();
    let focused = is_focused(id);

    let border_style = if focused {
        BorderStyle::Double
    } else {
        BorderStyle::Single
    };

    let border_color = if focused {
        Color::White
    } else {
        Color::DarkGrey
    };

    let visible_width = match props.width {
        Dimension::Length(w) => ((w.round() as u16).saturating_sub(4)).max(1) as usize,
        _ => 20,
    };

    let owns_buffer = props.buffer.is_set();
    let text = use_controllable(
        props.value.bound(),
        props.value.value(),
        || props.default_value.clone().unwrap_or_default(),
        props.on_value_change,
    );
    let buffer = use_controllable(
        props.buffer.bound(),
        props.buffer.value(),
        || TextBuffer::new(""),
        props.on_buffer_change,
    );

    let (display_text, text_color, cursor_col) = with_buffer_mut(id, |buf| {
        buf.set_multiline(false);

        if owns_buffer {
            let current = buffer.get();
            if *buf != current {
                *buf = current;
                buf.set_multiline(false);
            }
        } else {
            let current = text.get();
            // The text changed from outside (or a controlled parent turned
            // an edit down), so the cursor goes to the end of the new text,
            // the way text fields usually behave.
            if buf.content() != current {
                buf.set_content(current);
                buf.move_end();
            }
        }

        buf.update_viewport(visible_width, 1);
        let cur_col = (buf.cursor_col.saturating_sub(buf.scroll_col)) as u16;

        if buf.is_empty() {
            let ph = if props.placeholder.is_empty() {
                String::from("Type here...")
            } else {
                props.placeholder.clone()
            };
            (ph, Color::DarkGrey, 0u16)
        } else {
            let visible = buf.visible_text(visible_width);
            (visible, Color::White, cur_col)
        }
    });

    let on_submit = props.on_submit;
    // Reports an edit or cursor move to whoever owns the text and buffer.
    let report: Rc<dyn Fn(Option<String>)> = {
        let text = text.clone();
        Rc::new(move |edited: Option<String>| {
            if owns_buffer {
                buffer.set(with_buffer(id, |b| b.clone()));
            }
            if let Some(content) = edited {
                text.set(content);
            }
        })
    };
    let intercept = props.on_key;

    let on_click_at: Rc<dyn Fn(u16, u16)> = {
        let report = report.clone();
        Rc::new(move |rel_x: u16, _rel_y: u16| {
            let click_col = rel_x.saturating_sub(2) as usize;
            with_buffer_mut(id, |buf| {
                let new_cursor = buf.scroll_col + click_col;
                buf.set_cursor(new_cursor);
            });
            report(None);
        })
    };

    let on_key: KeyHandler = Rc::new(move |key: KeyEvent, ctx: &mut EventContext| {
        if let Some(intercept) = &intercept
            && intercept(key, ctx) == EventResult::Handled
        {
            return EventResult::Handled;
        }
        if key.code == KeyCode::Enter {
            if let Some(ref submit) = on_submit {
                let text = with_buffer(id, |b| b.content());
                submit(text);
            }
            return if on_submit.is_some() {
                EventResult::Handled
            } else {
                EventResult::Ignored
            };
        }

        if !with_buffer(id, |buf| buf.accepts_key(&key)) {
            return EventResult::Ignored;
        }
        let modified = with_buffer_mut(id, |buf| buf.handle_key(&key));
        if modified {
            report(Some(with_buffer(id, |buf| buf.content())));
        }
        EventResult::Handled
    });

    let cursor_offset = if focused { Some((cursor_col, 0)) } else { None };
    let cursor_style = if focused {
        Some(props.cursor_style)
    } else {
        None
    };
    let title_str = props
        .title
        .clone()
        .unwrap_or_else(|| props.placeholder.clone());
    let semantics = props
        .semantics
        .unwrap_or_else(|| Semantics::new(Role::Textbox, title_str));

    view! {
        <Pressable
            focus_id=id
            tab_stop=true
            semantics=semantics
            on_click_at=on_click_at
            on_click={props.on_click}
            on_key=on_key
        >
            <View
                border=border_style
                border_color=border_color
                padding={Padding::symmetric(1, 0)}
                width={props.width}
                justify_content=JustifyContent::Center
                align_items=AlignItems::FlexStart
                title={props.title}
                cursor_offset=cursor_offset
                cursor_style=cursor_style
            >
                <TextSpan foreground_color=text_color bold=focused>{display_text}</TextSpan>
            </View>
        </Pressable>
    }
}
