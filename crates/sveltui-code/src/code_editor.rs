use crate::buffer::{with_code_buffer, with_code_buffer_mut};
use crate::highlight::highlight_text;
use crossterm::event::KeyEvent;
use crossterm::style::Color;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use sveltui::element::scroll::{get_scroll, get_viewport, set_scroll};
use sveltui::layout::Dimension;
use sveltui::prelude::*;

thread_local! {
    static LAST_SYNCED: RefCell<HashMap<FocusId, String>> = RefCell::new(HashMap::new());
}

#[derive(Props)]
pub struct CodeEditorProps {
    #[prop(optional)]
    pub id: Option<FocusId>,
    /// The code. Pass it to control the editor yourself: edits then only
    /// call `on_value_change`, and the code resets unless you pass the new
    /// value back.
    /// Code shared with the parent, updated on every edit. Pass it as
    /// `bind:value={state}`.
    #[prop(bindable)]
    pub value: Bindable<String>,
    /// The code it starts with when nothing else decides.
    #[prop(optional)]
    pub default_value: Option<String>,
    #[prop(optional)]
    pub on_value_change: Option<Rc<dyn Fn(String)>>,
    #[prop(default = String::from("// Type code here..."))]
    pub placeholder: String,
    #[prop(optional)]
    pub language: Option<String>,
    #[prop(optional)]
    pub theme: Option<String>,
    #[prop(default = true)]
    pub show_line_numbers: bool,
    #[prop(default = Dimension::Auto)]
    pub width: Dimension,
    #[prop(default = Dimension::Auto)]
    pub height: Dimension,
    #[prop(optional)]
    pub title: Option<String>,
    #[prop(default = 4)]
    pub tab_size: usize,
    #[prop(default = true)]
    pub autofocus: bool,
    #[prop(default = CursorStyle::Line)]
    pub cursor_style: CursorStyle,
}

#[component]
pub fn CodeEditor(props: CodeEditorProps) -> impl IntoNode {
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

    let bound = props.value.bound();
    let on_value_change = props.on_value_change.clone();
    // The parent's code, if it owns it. Without an owner, the buffer keeps
    // the code itself after starting from `default_value`.
    let owned = bound
        .as_ref()
        .map(|state| state.get())
        .or_else(|| props.value.value());

    let total_lines = with_code_buffer_mut(id, |buf| {
        buf.tab_size = props.tab_size;
        let last = LAST_SYNCED.with(|m| m.borrow().get(&id).cloned());
        // Sync whenever the owner's code differs from what we last synced.
        // That covers outside changes and edits a controlled parent turned
        // down.
        let first = last.is_none();
        let incoming = match (owned.clone(), last) {
            (Some(current), Some(prev)) if prev != current => Some(current),
            (Some(current), None) => Some(current),
            (None, None) => Some(props.default_value.clone().unwrap_or_default()),
            _ => None,
        };
        if let Some(code) = incoming {
            // On the first render an empty value leaves the fresh buffer alone.
            if !(first && code.is_empty()) {
                buf.set_content(&code);
            }
            LAST_SYNCED.with(|m| m.borrow_mut().insert(id, code));
        }
        buf.line_count()
    });

    let digits = format!("{}", total_lines.max(1)).len().max(2);
    let gutter_width = if props.show_line_numbers {
        digits + 3
    } else {
        0
    };

    let (_rendered_vw, rendered_vh) = get_viewport(id).unwrap_or_else(|| {
        let (term_w, term_h) = crossterm::terminal::size().unwrap_or((80, 24));
        let default_h = match props.height {
            Dimension::Length(h) => (h.round() as u16).saturating_sub(2),
            _ => term_h.saturating_sub(2),
        };
        let default_w = match props.width {
            Dimension::Length(w) => (w.round() as u16).saturating_sub(4 + gutter_width as u16),
            _ => term_w.saturating_sub(4 + gutter_width as u16),
        };
        (default_w.max(1), default_h.max(1))
    });

    let visible_height = (rendered_vh as usize).max(1);

    let (is_empty, full_content) = with_code_buffer(id, |buf| (buf.is_empty(), buf.content()));

    let all_line_spans = highlight_text(
        &full_content,
        props.language.as_deref(),
        props.theme.as_deref(),
    );

    let show_line_numbers = props.show_line_numbers;

    let rendered_line_nodes: Vec<VNode> = if is_empty {
        let mut row = ViewProps::builder()
            .flex_direction(FlexDirection::Row)
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::FlexStart);
        if show_line_numbers {
            row = row.child(TextSpan(
                TextSpanProps::builder()
                    .foreground_color(Color::DarkGrey)
                    .content(format!("{:>width$} │ ", 1, width = digits))
                    .build(),
            ));
        }
        row = row.child(TextSpan(
            TextSpanProps::builder()
                .foreground_color(Color::DarkGrey)
                .content(props.placeholder.clone())
                .build(),
        ));
        vec![View(row.build()).into_node()]
    } else {
        all_line_spans
            .into_iter()
            .enumerate()
            .map(|(offset, spans)| {
                let line_num = offset + 1;
                let mut row = ViewProps::builder()
                    .flex_direction(FlexDirection::Row)
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::FlexStart);
                if show_line_numbers {
                    row = row.child(TextSpan(
                        TextSpanProps::builder()
                            .foreground_color(Color::DarkGrey)
                            .content(format!("{:>width$} │ ", line_num, width = digits))
                            .build(),
                    ));
                }
                if spans.is_empty() {
                    row = row.child(TextSpan(
                        TextSpanProps::builder()
                            .foreground_color(Color::DarkGrey)
                            .content(" ")
                            .build(),
                    ));
                } else {
                    for span in spans {
                        row = row.child(TextSpan(
                            TextSpanProps::builder()
                                .foreground_color(span.fg)
                                .bold(span.bold)
                                .italic(span.italic)
                                .underline(span.underline)
                                .content(span.text)
                                .build(),
                        ));
                    }
                }
                View(row.build()).into_node()
            })
            .collect()
    };

    let on_click_at: Rc<dyn Fn(u16, u16)> = Rc::new(move |rel_x: u16, rel_y: u16| {
        let (scroll_x, scroll_y) = get_scroll(id);
        let click_col =
            (rel_x as i32 + scroll_x as i32 - (2 + gutter_width as i32)).max(0) as usize;
        let click_row = (rel_y as i32 + scroll_y as i32 - 1).max(0) as usize;
        with_code_buffer_mut(id, |buf| {
            buf.set_cursor_pos(click_row, click_col);
        });
    });

    let on_key: KeyHandler = Rc::new(move |key: KeyEvent, _ctx: &mut EventContext| {
        if !with_code_buffer(id, |buf| buf.accepts_key(&key)) {
            return EventResult::Ignored;
        }

        let (vw, vh) = get_viewport(id).unwrap_or_else(|| {
            let (term_w, term_h) = crossterm::terminal::size().unwrap_or((80, 24));
            (
                term_w.saturating_sub(4 + gutter_width as u16).max(1),
                term_h.saturating_sub(2).max(1),
            )
        });
        let cur_visible_w = ((vw as usize).saturating_sub(gutter_width)).max(1);
        let cur_visible_h = (vh as usize).max(1);

        let edited = with_code_buffer_mut(id, |buf| {
            let modified = buf.handle_key(&key);
            let (mut sx, mut sy) = get_scroll(id);

            if (buf.cursor_row as u16) < sy {
                sy = buf.cursor_row as u16;
            } else if buf.cursor_row >= (sy as usize) + cur_visible_h {
                sy = (buf.cursor_row.saturating_sub(cur_visible_h) + 1) as u16;
            }

            if (buf.cursor_col as u16) < sx {
                sx = buf.cursor_col as u16;
            } else if buf.cursor_col >= (sx as usize) + cur_visible_w {
                sx = (buf.cursor_col.saturating_sub(cur_visible_w) + 1) as u16;
            }

            set_scroll(id, (sx, sy));

            if modified {
                let text = buf.content();
                LAST_SYNCED.with(|m| m.borrow_mut().insert(id, text.clone()));
                Some(text)
            } else {
                None
            }
        });
        // Report after the buffer is released, so callbacks can read it.
        if let Some(text) = edited {
            if let Some(state) = &bound {
                state.set(text.clone());
            }
            if let Some(callback) = &on_value_change {
                callback(text);
            }
        }
        EventResult::Handled
    });

    let (scroll_x, scroll_y) = get_scroll(id);
    let (cur_row, cur_col) = with_code_buffer(id, |buf| (buf.cursor_row, buf.cursor_col));
    let rel_row = (cur_row as i32) - (scroll_y as i32);
    let rel_col = (gutter_width as i32 + cur_col as i32) - (scroll_x as i32);

    let cursor_offset =
        if focused && rel_row >= 0 && rel_col >= 0 && (rel_row as usize) < visible_height {
            Some((rel_col as u16, rel_row as u16))
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

    let focus_scope = if props.autofocus {
        Some(FocusScope {
            contain_tab: false,
            initial_focus: InitialFocus::Container,
        })
    } else {
        None
    };

    view! {
        <Pressable
            focus_id=id
            tab_stop=true
            focus_scope=focus_scope
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
                flex_direction=FlexDirection::Column
                align_items=AlignItems::FlexStart
                justify_content=JustifyContent::FlexStart
                overflow_x=Overflow::Scroll
                overflow_y=Overflow::Scroll
                scroll_id=id
                title={props.title}
                cursor_offset=cursor_offset
                cursor_style=cursor_style
            >
                {#each rendered_line_nodes as line_node}
                    {line_node}
                {/each}
            </View>
        </Pressable>
    }
}
