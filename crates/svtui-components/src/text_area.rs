use crate::text_input::TextInputBinding;
use crossterm::event::KeyEvent;
use crossterm::style::Color;
use std::rc::Rc;
use sveltui::prelude::*;

#[derive(Props)]
pub struct TextAreaProps {
    #[prop(optional)]
    pub id: Option<FocusId>,
    #[prop(optional)]
    pub value: Option<TextInputBinding>,
    #[prop(optional)]
    pub set_value: Option<WriteSignal<String>>,
    #[prop(optional)]
    pub buffer: Option<ReadSignal<TextBuffer>>,
    #[prop(optional)]
    pub set_buffer: Option<WriteSignal<TextBuffer>>,
    #[prop(default = String::from("Type text here..."))]
    pub placeholder: String,
    #[prop(default = Size::Fixed(40))]
    pub width: Size,
    #[prop(default = Size::Fixed(8))]
    pub height: Size,
    #[prop(optional)]
    pub title: Option<String>,
    #[prop(default = false)]
    pub show_line_numbers: bool,
}

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
        Size::Fixed(h) => (h.saturating_sub(2)).max(1) as usize,
        _ => 6,
    };

    let (val_sig, binding_set_val) = match props.value {
        Some(TextInputBinding::Read(r)) => (Some(r), None),
        Some(TextInputBinding::State(s)) => (Some(s.read()), Some(s.write())),
        None => (None, None),
    };

    let set_value = props.set_value.or(binding_set_val);

    let total_lines = with_buffer_mut(id, |buf| {
        buf.set_multiline(true);
        if let Some(buf_sig) = props.buffer {
            *buf = buf_sig.get();
            buf.set_multiline(true);
        } else if let Some(vs) = val_sig {
            let current_val = vs.get();
            if buf.content() != current_val {
                buf.set_content(current_val);
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
        Size::Fixed(w) => (w.saturating_sub(4 + gutter_width as u16)).max(1) as usize,
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

    let set_buffer = props.set_buffer;
    let show_line_numbers = props.show_line_numbers;

    let on_click_at: Rc<dyn Fn(u16, u16)> = Rc::new(move |rel_x: u16, rel_y: u16| {
        let click_col = rel_x.saturating_sub(2 + gutter_width as u16) as usize;
        let click_row = rel_y.saturating_sub(1) as usize;
        with_buffer_mut(id, |buf| {
            let target_row = buf.scroll_row + click_row;
            let target_col = buf.scroll_col + click_col;
            buf.set_cursor_pos(target_row, target_col);
            if let Some(sb) = set_buffer {
                sb.set(buf.clone());
            }
        });
    });

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
        with_buffer_mut(id, |buf| {
            let modified = buf.handle_key(&key);
            if modified {
                let text = buf.content();
                if let Some(sv) = set_value {
                    sv.set(text);
                }
                if let Some(sb) = set_buffer {
                    sb.set(buf.clone());
                }
            }
        });
        EventResult::Handled
    });

    let cursor_offset = if focused {
        Some((cursor_x, cursor_y))
    } else {
        None
    };
    let title_str = props
        .title
        .clone()
        .unwrap_or_else(|| props.placeholder.clone());

    view! {
        <FlexBox
            border=border_style
            border_color=border_color
            padding=Padding::symmetric(1, 0)
            width=props.width
            height=props.height
            align_items=Align::Start
            justify_content=Justify::Start
            focus_id=id
            tab_stop=true
            semantics=Semantics::new(Role::Textbox, title_str)
            on_click_at=on_click_at
            on_scroll=on_scroll
            on_key=on_key
            opt_title=props.title
            opt_cursor_offset=cursor_offset
        >
            {#if is_empty}
                {#if show_line_numbers}
                    <TextSpan fg=Color::DarkGrey>{format!("{:>width$}  {}", 1, props.placeholder, width = digits)}</TextSpan>
                {:else}
                    <TextSpan fg=Color::DarkGrey>{props.placeholder.clone()}</TextSpan>
                {/if}
            {:else}
                {#each lines.iter().enumerate() as (idx, line_text)}
                    {#if show_line_numbers}
                        <TextSpan fg=Color::White>{format!("{:>width$}  {}", scroll_row + idx + 1, line_text, width = digits)}</TextSpan>
                    {:else}
                        <TextSpan fg=Color::White>{line_text.clone()}</TextSpan>
                    {/if}
                {/each}
            {/if}
        </FlexBox>
    }
}
