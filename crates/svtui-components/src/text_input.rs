use crossterm::event::{KeyCode, KeyEvent};
use crossterm::style::Color;
use std::cell::RefCell;
use std::collections::HashMap;
use svtui_element::{BoxElement, FocusId, IntoNode, TextElement, VNode, is_focused};
use svtui_layout::{Align, BorderStyle, Justify, Padding, Size};
use svtui_reactive::{ReadSignal, WriteSignal};

thread_local! {
    static CURSOR_MAP: RefCell<HashMap<FocusId, usize>> = RefCell::new(HashMap::new());
}

fn get_cursor(id: FocusId, text_len: usize) -> usize {
    CURSOR_MAP.with(|m| {
        let mut map = m.borrow_mut();
        let cur = map.entry(id).or_insert(text_len);
        if *cur > text_len {
            *cur = text_len;
        }
        *cur
    })
}

fn set_cursor(id: FocusId, pos: usize) {
    CURSOR_MAP.with(|m| {
        m.borrow_mut().insert(id, pos);
    });
}

#[derive(Clone, Default)]
pub struct TextInput {
    value: Option<ReadSignal<String>>,
    set_value: Option<WriteSignal<String>>,
    placeholder: String,
    width: Size,
    title: Option<String>,
    id: Option<FocusId>,
}

impl TextInput {
    pub fn new(value: ReadSignal<String>, set_value: WriteSignal<String>) -> Self {
        Self {
            value: Some(value),
            set_value: Some(set_value),
            placeholder: String::from("Type here..."),
            width: Size::Fixed(24),
            title: None,
            id: None,
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
}

impl IntoNode for TextInput {
    fn into_node(self) -> VNode {
        let id = self.id.unwrap_or_default();
        let focused = is_focused(id);
        let current_text = self.value.map(|v| v.get()).unwrap_or_default();
        let text_len = current_text.chars().count();
        let cur_pos = get_cursor(id, text_len);

        let (display_text, text_color) = if focused {
            // Render text with inline cursor marker at current cursor position
            let mut chars: Vec<char> = current_text.chars().collect();
            let insert_idx = cur_pos.min(chars.len());
            chars.insert(insert_idx, '│');
            let with_cursor: String = chars.into_iter().collect();
            (with_cursor, Color::White)
        } else if current_text.is_empty() {
            (
                if self.placeholder.is_empty() {
                    String::from("Type here...")
                } else {
                    self.placeholder.clone()
                },
                Color::DarkGrey,
            )
        } else {
            (current_text.clone(), Color::White)
        };

        let border_color = if focused {
            Color::White
        } else {
            Color::DarkGrey
        };

        let border_style = if focused {
            BorderStyle::Double
        } else {
            BorderStyle::Single
        };

        let set_value = self.set_value;
        let text_clone = current_text.clone();

        let mut b = BoxElement::new()
            .border(border_style)
            .border_color(border_color)
            .padding(Padding::symmetric(1, 0))
            .width(self.width)
            .align_items(Align::Center)
            .justify_content(Justify::Start)
            .focus_id(id)
            .on_click(move || {
                // Focus is handled by app via focus_id
            })
            .on_key(move |key: KeyEvent| {
                let text = text_clone.clone();
                let chars_len = text.chars().count();
                let cur = get_cursor(id, chars_len);

                match key.code {
                    KeyCode::Char(c) => {
                        let mut chars: Vec<char> = text.chars().collect();
                        let insert_idx = cur.min(chars.len());
                        chars.insert(insert_idx, c);
                        let new_text: String = chars.into_iter().collect();
                        set_cursor(id, cur.saturating_add(1));
                        if let Some(set_v) = set_value {
                            set_v.set(new_text);
                        }
                    }
                    KeyCode::Backspace => {
                        if cur > 0 && chars_len > 0 {
                            let mut chars: Vec<char> = text.chars().collect();
                            let del_idx = cur - 1;
                            if del_idx < chars.len() {
                                chars.remove(del_idx);
                                let new_text: String = chars.into_iter().collect();
                                set_cursor(id, cur - 1);
                                if let Some(set_v) = set_value {
                                    set_v.set(new_text);
                                }
                            }
                        }
                    }
                    KeyCode::Delete => {
                        if cur < chars_len {
                            let mut chars: Vec<char> = text.chars().collect();
                            chars.remove(cur);
                            let new_text: String = chars.into_iter().collect();
                            if let Some(set_v) = set_value {
                                set_v.set(new_text);
                            }
                        }
                    }
                    KeyCode::Left => {
                        if cur > 0 {
                            set_cursor(id, cur - 1);
                        }
                    }
                    KeyCode::Right => {
                        if cur < chars_len {
                            set_cursor(id, cur + 1);
                        }
                    }
                    KeyCode::Home => {
                        set_cursor(id, 0);
                    }
                    KeyCode::End => {
                        set_cursor(id, chars_len);
                    }
                    _ => {}
                }
            })
            .child(
                TextElement::new(display_text)
                    .color(text_color)
                    .bold(focused)
                    .into_node(),
            );

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
