use crate::text_buffer::TextBuffer;
use crossterm::event::{KeyCode, KeyEvent};
use crossterm::style::Color;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use svtui_element::{BoxElement, FocusId, IntoNode, TextElement, VNode, is_focused};
use svtui_layout::{Align, BorderStyle, Justify, Padding, Size};
use svtui_reactive::{ReadSignal, WriteSignal};

thread_local! {
    static BUFFER_MAP: RefCell<HashMap<FocusId, TextBuffer>> = RefCell::new(HashMap::new());
}

#[allow(dead_code)]
fn get_cursor(id: FocusId, text_len: usize) -> usize {
    BUFFER_MAP.with(|m| {
        let mut map = m.borrow_mut();
        let buf = map.entry(id).or_default();
        if buf.cursor > text_len {
            buf.cursor = text_len;
        }
        buf.cursor
    })
}

#[allow(dead_code)]
fn set_cursor(id: FocusId, pos: usize) {
    BUFFER_MAP.with(|m| {
        let mut map = m.borrow_mut();
        let buf = map.entry(id).or_default();
        buf.cursor = pos;
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
        let (display_text, text_color, cursor_col) = BUFFER_MAP.with(|m| {
            let mut map = m.borrow_mut();
            let buf = map.entry(id).or_default();

            if let Some(buf_sig) = self.buffer {
                *buf = buf_sig.get();
            } else if let Some(val_sig) = self.value {
                let current_val = val_sig.get();
                if buf.content() != current_val {
                    buf.set_content(current_val);
                }
            }

            buf.update_viewport(visible_width);
            let cur_col = (buf.cursor().saturating_sub(buf.scroll_offset())) as u16;

            if buf.is_empty() {
                let ph = if self.placeholder.is_empty() {
                    String::from("Type here...")
                } else {
                    self.placeholder.clone()
                };
                (ph, Color::DarkGrey, 0u16)
            } else {
                let visible = buf.visible_text(visible_width).to_string();
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
            .align_items(Align::Center)
            .justify_content(Justify::Start)
            .focus_id(id)
            .on_click_at(move |rel_x: u16, _rel_y: u16| {
                // Outer box starts at target.rect.x.
                // Inner text starts after border(1) + padding(1) = 2.
                let click_col = rel_x.saturating_sub(2) as usize;
                BUFFER_MAP.with(|m| {
                    let mut map = m.borrow_mut();
                    if let Some(buf) = map.get_mut(&id) {
                        let new_cursor = buf.scroll_offset() + click_col;
                        buf.set_cursor(new_cursor);
                        if let Some(sb) = set_buffer {
                            sb.set(buf.clone());
                        }
                    }
                });
            })
            .on_key(move |key: KeyEvent| {
                if key.code == KeyCode::Enter {
                    if let Some(ref submit) = on_submit {
                        let text = BUFFER_MAP.with(|m| {
                            m.borrow()
                                .get(&id)
                                .map(|b| b.content().to_string())
                                .unwrap_or_default()
                        });
                        submit(text);
                    }
                    return;
                }

                BUFFER_MAP.with(|m| {
                    let mut map = m.borrow_mut();
                    if let Some(buf) = map.get_mut(&id) {
                        let modified = buf.handle_key(&key);
                        if modified {
                            let text = buf.content().to_string();
                            if let Some(sv) = set_value {
                                sv.set(text);
                            }
                            if let Some(sb) = set_buffer {
                                sb.set(buf.clone());
                            }
                        }
                    }
                });
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

#[cfg(test)]
mod tests {
    use super::*;
    use svtui_reactive::create_signal;

    #[test]
    fn test_text_input_cursor_forward_typing() {
        let (val, set_val) = create_signal(String::new());
        let id = FocusId(999);

        // Initially cursor should be at 0
        assert_eq!(get_cursor(id, 0), 0);

        // Type 'a'
        let mut text: String = val.get();
        let cur = get_cursor(id, text.len());
        text.insert(cur, 'a');
        set_cursor(id, cur + 1);
        set_val.set(text);

        assert_eq!(val.get(), "a");
        assert_eq!(get_cursor(id, 1), 1);

        // Type 'b'
        let mut text: String = val.get();
        let cur = get_cursor(id, text.len());
        text.insert(cur, 'b');
        set_cursor(id, cur + 1);
        set_val.set(text);

        // Value must be "ab", NOT "ba"!
        assert_eq!(val.get(), "ab");
        assert_eq!(get_cursor(id, 2), 2);

        // Type 'c'
        let mut text: String = val.get();
        let cur = get_cursor(id, text.len());
        text.insert(cur, 'c');
        set_cursor(id, cur + 1);
        set_val.set(text);

        // Value must be "abc"!
        assert_eq!(val.get(), "abc");
        assert_eq!(get_cursor(id, 3), 3);
    }
}
