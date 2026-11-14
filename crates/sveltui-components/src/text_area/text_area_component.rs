use crossterm::event::KeyEvent;
use crossterm::style::Color;
use std::rc::Rc;
use sveltui::prelude::*;

use super::text_area_types::*;
use crate::controllable::use_controllable;

#[component]
pub fn TextArea(props: TextAreaProps) -> impl IntoNode {
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

    let visible_height = match props.height {
        Dimension::Length(h) => ((h.round() as u16).saturating_sub(2)).max(1) as usize,
        _ => 6,
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

    let total_lines = with_buffer_mut(id, |buf| {
        buf.set_multiline(true);
        if owns_buffer {
            let current = buffer.get();
            if *buf != current {
                *buf = current;
                buf.set_multiline(true);
            }
        } else {
            // The text changed from outside, or a controlled parent turned
            // an edit down, so the buffer takes the owner's text.
            let current = text.get();
            if buf.content() != current {
                buf.set_content(current);
            }
        }
        buf.line_count()
    });

    let digits = format!("{}", total_lines.max(1)).len().max(2);
    let gutter_width = if props.show_line_numbers {
        digits + 2
    } else {
        0
    };

    let visible_width = match props.width {
        Dimension::Length(w) => {
            ((w.round() as u16).saturating_sub(4 + gutter_width as u16)).max(1) as usize
        }
        _ => 30,
    };

    let (lines, cursor_x, cursor_y, is_empty, scroll_row) = with_buffer_mut(id, |buf| {
        buf.update_viewport(visible_width, visible_height);

        let rel_col = (buf.cursor_col.saturating_sub(buf.scroll_col)) as u16;
        let rel_row = (buf.cursor_row.saturating_sub(buf.scroll_row)) as u16;

        let cur_x = gutter_width as u16 + rel_col;
        let cur_y = rel_row;

        let lines = buf.visible_lines(visible_width, visible_height);
        let empty = buf.is_empty();

        (lines, cur_x, cur_y, empty, buf.scroll_row)
    });

    let show_line_numbers = props.show_line_numbers;
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

    let on_click_at: Rc<dyn Fn(u16, u16)> = {
        let report = report.clone();
        Rc::new(move |rel_x: u16, rel_y: u16| {
            let click_col = rel_x.saturating_sub(2 + gutter_width as u16) as usize;
            let click_row = rel_y.saturating_sub(1) as usize;
            with_buffer_mut(id, |buf| {
                let target_row = buf.scroll_row + click_row;
                let target_col = buf.scroll_col + click_col;
                buf.set_cursor_pos(target_row, target_col);
            });
            report(None);
        })
    };

    let on_scroll: Rc<dyn Fn(i16, i16)> = Rc::new(move |_dx: i16, dy: i16| {
        with_buffer_mut(id, |buf| {
            if dy > 0 {
                let max_row = buf.line_count().saturating_sub(1);
                buf.scroll_row = (buf.scroll_row + dy as usize).min(max_row);
            } else if dy < 0 {
                buf.scroll_row = buf.scroll_row.saturating_sub((-dy) as usize);
            }
        });
    });

    let on_key: KeyHandler = Rc::new(move |key: KeyEvent, _ctx: &mut EventContext| {
        if !with_buffer(id, |buf| buf.accepts_key(&key)) {
            return EventResult::Ignored;
        }
        let modified = with_buffer_mut(id, |buf| buf.handle_key(&key));
        if modified {
            report(Some(with_buffer(id, |buf| buf.content())));
        }
        EventResult::Handled
    });

    let cursor_offset = if focused {
        Some((cursor_x, cursor_y))
    } else {
        None
    };
    let cursor_style = if focused {
        Some(props.cursor_style)
    } else {
        None
    };
    let title_str = props
        .title
        .clone()
        .unwrap_or_else(|| props.placeholder.clone());

    view! {
        <Pressable
            focus_id=id
            tab_stop=true
            semantics={Semantics::new(Role::Textbox, title_str)}
            on_click_at=on_click_at
            on_key=on_key
        >
            <View
                border=border_style
                border_color=border_color
                padding={Padding::symmetric(1, 0)}
                width={props.width}
                height={props.height}
                align_items=AlignItems::FlexStart
                justify_content=JustifyContent::FlexStart
                on_scroll=on_scroll
                title={props.title}
                cursor_offset=cursor_offset
                cursor_style=cursor_style
            >
                {#if is_empty}
                    {#if show_line_numbers}
                        <TextSpan foreground_color=Color::DarkGrey>{format!("{:>width$}  {}", 1, props.placeholder, width = digits)}</TextSpan>
                    {:else}
                        <TextSpan foreground_color=Color::DarkGrey>{props.placeholder.clone()}</TextSpan>
                    {/if}
                {:else}
                    {#each lines.iter().enumerate() as (idx, line_text)}
                        {#if show_line_numbers}
                            <TextSpan foreground_color=Color::White>{format!("{:>width$}  {}", scroll_row + idx + 1, line_text, width = digits)}</TextSpan>
                        {:else}
                            <TextSpan foreground_color=Color::White>{line_text.clone()}</TextSpan>
                        {/if}
                    {/each}
                {/if}
            </View>
        </Pressable>
    }
}
