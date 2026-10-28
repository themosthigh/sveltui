use crate::text_buffer::with_buffer;
use crate::text_buffer::{TextBuffer, with_buffer_mut};
use crossterm::event::KeyEvent;
use crossterm::style::Color;
use svtui_element::{BoxElement, FocusId, IntoNode, TextElement, VNode, is_focused};
use svtui_element::{EventResult, Role, Semantics};
use svtui_layout::{Align, BorderStyle, Justify, Padding, Size};
use svtui_reactive::{ReadSignal, WriteSignal};

#[derive(Clone)]
pub struct TextArea {
    value: Option<ReadSignal<String>>,
    set_value: Option<WriteSignal<String>>,
    buffer: Option<ReadSignal<TextBuffer>>,
    set_buffer: Option<WriteSignal<TextBuffer>>,
    placeholder: String,
    width: Size,
    height: Size,
    title: Option<String>,
    id: Option<FocusId>,
    show_line_numbers: bool,
}

impl Default for TextArea {
    fn default() -> Self {
        Self {
            value: None,
            set_value: None,
            buffer: None,
            set_buffer: None,
            placeholder: String::from("Type text here..."),
            width: Size::Fixed(40),
            height: Size::Fixed(8),
            title: None,
            id: None,
            show_line_numbers: false,
        }
    }
}

impl TextArea {
    pub fn new(value: ReadSignal<String>, set_value: WriteSignal<String>) -> Self {
        Self {
            value: Some(value),
            set_value: Some(set_value),
            ..Default::default()
        }
    }

    pub fn from_buffer(
        buffer: ReadSignal<TextBuffer>,
        set_buffer: WriteSignal<TextBuffer>,
    ) -> Self {
        Self {
            buffer: Some(buffer),
            set_buffer: Some(set_buffer),
            ..Default::default()
        }
    }

    pub fn value(mut self, value: ReadSignal<String>) -> Self {
        self.value = Some(value);
        self
    }

    pub fn set_value(mut self, set_value: WriteSignal<String>) -> Self {
        self.set_value = Some(set_value);
        self
    }

    pub fn buffer(mut self, buffer: ReadSignal<TextBuffer>) -> Self {
        self.buffer = Some(buffer);
        self
    }

    pub fn set_buffer(mut self, set_buffer: WriteSignal<TextBuffer>) -> Self {
        self.set_buffer = Some(set_buffer);
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn width(mut self, width: impl Into<Size>) -> Self {
        self.width = width.into();
        self
    }

    pub fn height(mut self, height: impl Into<Size>) -> Self {
        self.height = height.into();
        self
    }

    pub fn id(mut self, id: FocusId) -> Self {
        self.id = Some(id);
        self
    }

    pub fn show_line_numbers(mut self, show: bool) -> Self {
        self.show_line_numbers = show;
        self
    }
}

impl IntoNode for TextArea {
    fn into_node(self) -> VNode {
        let id = self.id.unwrap_or_default();
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

        let visible_height = match self.height {
            Size::Fixed(h) => (h.saturating_sub(2)).max(1) as usize,
            _ => 6,
        };

        // Determine total lines and gutter size
        let total_lines = with_buffer_mut(id, |buf| {
            buf.set_multiline(true);
            if let Some(buf_sig) = self.buffer {
                *buf = buf_sig.get();
                buf.set_multiline(true);
            } else if let Some(val_sig) = self.value {
                let current_val = val_sig.get();
                if buf.content() != current_val {
                    buf.set_content(current_val);
                }
            }
            buf.line_count()
        });

        let digits = format!("{}", total_lines.max(1)).len().max(2);
        let gutter_width = if self.show_line_numbers {
            digits + 2
        } else {
            0
        };

        let visible_width = match self.width {
            Size::Fixed(w) => (w.saturating_sub(4 + gutter_width as u16)).max(1) as usize,
            _ => 30,
        };

        // Update viewport and extract visible lines & cursor
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

        let set_value = self.set_value;
        let set_buffer = self.set_buffer;
        let show_line_numbers = self.show_line_numbers;

        let mut b = BoxElement::new()
            .border(border_style)
            .border_color(border_color)
            .padding(Padding::symmetric(1, 0))
            .width(self.width)
            .height(self.height)
            .align_items(Align::Start)
            .justify_content(Justify::Start)
            .focus_id(id)
            .tab_stop(true)
            .semantics(Semantics::new(
                Role::Textbox,
                self.title
                    .clone()
                    .unwrap_or_else(|| self.placeholder.clone()),
            ))
            .on_click_at(move |rel_x: u16, rel_y: u16| {
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
            })
            .on_scroll(move |_dx, dy| {
                with_buffer_mut(id, |buf| {
                    if dy > 0 {
                        let max_row = buf.line_count().saturating_sub(1);
                        buf.scroll_row = (buf.scroll_row + dy as usize).min(max_row);
                    } else if dy < 0 {
                        buf.scroll_row = buf.scroll_row.saturating_sub((-dy) as usize);
                    }
                });
            })
            .on_key(move |key: KeyEvent, _ctx| {
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

        if focused {
            b = b.cursor_offset(cursor_x, cursor_y);
        }

        if let Some(ref title) = self.title {
            b = b.title(title.clone());
        }

        // Render line children
        if is_empty {
            let placeholder_line = if show_line_numbers {
                format!("{:>width$}  {}", 1, self.placeholder, width = digits)
            } else {
                self.placeholder.clone()
            };
            b = b.child(
                TextElement::new(placeholder_line)
                    .color(Color::DarkGrey)
                    .into_node(),
            );
        } else {
            for (idx, line_text) in lines.into_iter().enumerate() {
                let line_num = scroll_row + idx + 1;
                let full_line = if show_line_numbers {
                    format!("{:>width$}  {}", line_num, line_text, width = digits)
                } else {
                    line_text
                };
                b = b.child(TextElement::new(full_line).color(Color::White).into_node());
            }
        }

        b.into_node()
    }
}
