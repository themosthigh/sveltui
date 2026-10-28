use crate::text_buffer::{TextBuffer, with_buffer, with_buffer_mut};
use crossterm::event::{KeyCode, KeyEvent};
use crossterm::style::Color;
use std::rc::Rc;
use svtui_element::{BoxElement, FocusId, IntoNode, TextElement, VNode, is_focused};
use svtui_element::{EventResult, Role, Semantics};
use svtui_layout::{Align, BorderStyle, Justify, Padding, Size};
use svtui_reactive::{ReadSignal, State, WriteSignal};

pub enum TextInputBinding {
    Read(ReadSignal<String>),
    State(State<String>),
}

impl From<ReadSignal<String>> for TextInputBinding {
    fn from(s: ReadSignal<String>) -> Self {
        Self::Read(s)
    }
}

impl From<State<String>> for TextInputBinding {
    fn from(s: State<String>) -> Self {
        Self::State(s)
    }
}

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

#[derive(Clone, Default)]
pub struct TextInput {
    value: Option<ReadSignal<String>>,
    set_value: Option<WriteSignal<String>>,
    buffer: Option<ReadSignal<TextBuffer>>,
    set_buffer: Option<WriteSignal<TextBuffer>>,
    placeholder: String,
    width: Size,
    title: Option<String>,
    id: Option<FocusId>,
    on_submit: Option<Rc<dyn Fn(String)>>,
}

impl TextInput {
    pub fn new(value: ReadSignal<String>, set_value: WriteSignal<String>) -> Self {
        Self {
            value: Some(value),
            set_value: Some(set_value),
            buffer: None,
            set_buffer: None,
            placeholder: String::from("Type here..."),
            width: Size::Fixed(24),
            title: None,
            id: None,
            on_submit: None,
        }
    }

    pub fn from_buffer(
        buffer: ReadSignal<TextBuffer>,
        set_buffer: WriteSignal<TextBuffer>,
    ) -> Self {
        Self {
            value: None,
            set_value: None,
            buffer: Some(buffer),
            set_buffer: Some(set_buffer),
            placeholder: String::from("Type here..."),
            width: Size::Fixed(24),
            title: None,
            id: None,
            on_submit: None,
        }
    }

    pub fn value(mut self, value: impl Into<TextInputBinding>) -> Self {
        match value.into() {
            TextInputBinding::Read(r) => {
                self.value = Some(r);
            }
            TextInputBinding::State(s) => {
                self.value = Some(s.read());
                self.set_value = Some(s.write());
            }
        }
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

    pub fn id(mut self, id: FocusId) -> Self {
        self.id = Some(id);
        self
    }

    pub fn on_submit<F>(mut self, handler: F) -> Self
    where
        F: Fn(String) + 'static,
    {
        self.on_submit = Some(Rc::new(handler));
        self
    }
}

impl IntoNode for TextInput {
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

        // Determine visible inner content width: width minus border (2) and padding (2)
        let visible_width = match self.width {
            Size::Fixed(w) => (w.saturating_sub(4)).max(1) as usize,
            _ => 20,
        };

        // Sync or retrieve TextBuffer for this FocusId
        let (display_text, text_color, cursor_col) = with_buffer_mut(id, |buf| {
            buf.set_multiline(false);

            if let Some(buf_sig) = self.buffer {
                *buf = buf_sig.get();
                buf.set_multiline(false);
            } else if let Some(val_sig) = self.value {
                let current_val = val_sig.get();
                if buf.content() != current_val {
                    buf.set_content(current_val);
                }
            }

            buf.update_viewport(visible_width, 1);
            let cur_col = (buf.cursor_col.saturating_sub(buf.scroll_col)) as u16;

            if buf.is_empty() {
                let ph = if self.placeholder.is_empty() {
                    String::from("Type here...")
                } else {
                    self.placeholder.clone()
                };
                (ph, Color::DarkGrey, 0u16)
            } else {
                let visible = buf.visible_text(visible_width);
                (visible, Color::White, cur_col)
            }
        });

        let set_value = self.set_value;
        let set_buffer = self.set_buffer;
        let on_submit = self.on_submit;

        let mut b = BoxElement::new()
            .border(border_style)
            .border_color(border_color)
            .padding(Padding::symmetric(1, 0))
            .width(self.width)
            .justify_content(Justify::Center)
            .align_items(Align::Start)
            .focus_id(id)
            .tab_stop(true)
            .semantics(Semantics::new(
                Role::Textbox,
                self.title
                    .clone()
                    .unwrap_or_else(|| self.placeholder.clone()),
            ))
            .on_click_at(move |rel_x: u16, _rel_y: u16| {
                // Outer box starts at target.rect.x.
                // Inner text starts after border(1) + padding(1) = 2.
                let click_col = rel_x.saturating_sub(2) as usize;
                with_buffer_mut(id, |buf| {
                    let new_cursor = buf.scroll_col + click_col;
                    buf.set_cursor(new_cursor);
                    if let Some(sb) = set_buffer {
                        sb.set(buf.clone());
                    }
                });
            })
            .on_key(move |key: KeyEvent, _ctx| {
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
            })
            .child(
                TextElement::new(display_text)
                    .color(text_color)
                    .bold(focused)
                    .into_node(),
            );

        if focused {
            b = b.cursor_offset(cursor_col, 0);
        }

        if let Some(ref title) = self.title {
            b = b.title(title.clone());
        }

        b.into_node()
    }
}
