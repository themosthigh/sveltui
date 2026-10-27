use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct TextBuffer {
    content: String,
    pub cursor: usize,
    pub scroll_offset: usize,
}

fn char_to_byte_index(s: &str, char_idx: usize) -> usize {
    s.char_indices()
        .nth(char_idx)
        .map(|(b, _)| b)
        .unwrap_or(s.len())
}

impl TextBuffer {
    pub fn new(content: impl Into<String>) -> Self {
        let content = content.into();
        let cursor = content.chars().count();
        Self {
            content,
            cursor,
            scroll_offset: 0,
        }
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn as_str(&self) -> &str {
        &self.content
    }

    pub fn set_content(&mut self, text: impl Into<String>) {
        self.content = text.into();
        let len = self.len();
        if self.cursor > len {
            self.cursor = len;
        }
    }

    pub fn len(&self) -> usize {
        self.content.chars().count()
    }

    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn set_cursor(&mut self, pos: usize) {
        self.cursor = pos.min(self.len());
    }

    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    pub fn set_scroll_offset(&mut self, offset: usize) {
        self.scroll_offset = offset;
    }

    pub fn move_left(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    pub fn move_right(&mut self) {
        if self.cursor < self.len() {
            self.cursor += 1;
        }
    }

    pub fn move_home(&mut self) {
        self.cursor = 0;
    }

    pub fn move_end(&mut self) {
        self.cursor = self.len();
    }

    pub fn move_word_left(&mut self) {
        let chars: Vec<char> = self.content.chars().collect();
        let mut idx = self.cursor;

        // Skip whitespace to the left
        while idx > 0 && chars[idx - 1].is_whitespace() {
            idx -= 1;
        }
        // Skip word characters to the left
        while idx > 0 && !chars[idx - 1].is_whitespace() {
            idx -= 1;
        }
        self.cursor = idx;
    }

    pub fn move_word_right(&mut self) {
        let chars: Vec<char> = self.content.chars().collect();
        let len = chars.len();
        let mut idx = self.cursor;

        // Skip non-whitespace to the right
        while idx < len && !chars[idx].is_whitespace() {
            idx += 1;
        }
        // Skip whitespace to the right
        while idx < len && chars[idx].is_whitespace() {
            idx += 1;
        }
        self.cursor = idx;
    }

    pub fn insert_char(&mut self, c: char) {
        let byte_pos = char_to_byte_index(&self.content, self.cursor);
        self.content.insert(byte_pos, c);
        self.cursor += 1;
    }

    pub fn insert_str(&mut self, s: &str) {
        let byte_pos = char_to_byte_index(&self.content, self.cursor);
        self.content.insert_str(byte_pos, s);
        self.cursor += s.chars().count();
    }

    pub fn backspace(&mut self) -> bool {
        if self.cursor == 0 || self.content.is_empty() {
            return false;
        }
        let del_idx = self.cursor - 1;
        let start_byte = char_to_byte_index(&self.content, del_idx);
        let end_byte = char_to_byte_index(&self.content, self.cursor);
        self.content.drain(start_byte..end_byte);
        self.cursor = del_idx;
        true
    }

    pub fn delete(&mut self) -> bool {
        if self.cursor >= self.len() || self.content.is_empty() {
            return false;
        }
        let start_byte = char_to_byte_index(&self.content, self.cursor);
        let end_byte = char_to_byte_index(&self.content, self.cursor + 1);
        self.content.drain(start_byte..end_byte);
        true
    }

    pub fn delete_word_left(&mut self) -> bool {
        if self.cursor == 0 || self.content.is_empty() {
            return false;
        }
        let old_cursor = self.cursor;
        self.move_word_left();
        let new_cursor = self.cursor;
        let start_byte = char_to_byte_index(&self.content, new_cursor);
        let end_byte = char_to_byte_index(&self.content, old_cursor);
        self.content.drain(start_byte..end_byte);
        true
    }

    pub fn delete_to_end(&mut self) -> bool {
        if self.cursor >= self.len() {
            return false;
        }
        let start_byte = char_to_byte_index(&self.content, self.cursor);
        self.content.truncate(start_byte);
        true
    }

    pub fn delete_to_start(&mut self) -> bool {
        if self.cursor == 0 {
            return false;
        }
        let end_byte = char_to_byte_index(&self.content, self.cursor);
        self.content.drain(0..end_byte);
        self.cursor = 0;
        true
    }

    pub fn clear(&mut self) {
        self.content.clear();
        self.cursor = 0;
        self.scroll_offset = 0;
    }

    /// Adjusts `scroll_offset` so that `cursor` is visible within `visible_width`.
    pub fn update_viewport(&mut self, visible_width: usize) {
        if visible_width == 0 {
            return;
        }
        if self.cursor < self.scroll_offset {
            self.scroll_offset = self.cursor;
        } else if self.cursor >= self.scroll_offset + visible_width {
            self.scroll_offset = self.cursor.saturating_sub(visible_width) + 1;
        }
    }

    /// Returns the visible substring for `scroll_offset` and `visible_width`.
    pub fn visible_text(&self, visible_width: usize) -> &str {
        if visible_width == 0 || self.content.is_empty() {
            return "";
        }
        let start_byte = char_to_byte_index(&self.content, self.scroll_offset);
        let end_byte = char_to_byte_index(&self.content, self.scroll_offset + visible_width);
        &self.content[start_byte..end_byte]
    }

    /// Processes keyboard events, returning true if content was modified or cursor moved.
    pub fn handle_key(&mut self, key: &KeyEvent) -> bool {
        let is_ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let is_alt = key.modifiers.contains(KeyModifiers::ALT);

        match key.code {
            KeyCode::Char('w') if is_ctrl => self.delete_word_left(),
            KeyCode::Char('k') if is_ctrl => self.delete_to_end(),
            KeyCode::Char('u') if is_ctrl => self.delete_to_start(),
            KeyCode::Char('a') if is_ctrl => {
                self.move_home();
                true
            }
            KeyCode::Char('e') if is_ctrl => {
                self.move_end();
                true
            }
            KeyCode::Char('h') if is_ctrl => self.backspace(),
            KeyCode::Char('d') if is_ctrl => self.delete(),
            KeyCode::Char(c) if !is_ctrl && !is_alt => {
                self.insert_char(c);
                true
            }
            KeyCode::Backspace => {
                if is_alt {
                    self.delete_word_left()
                } else {
                    self.backspace()
                }
            }
            KeyCode::Delete => self.delete(),
            KeyCode::Left => {
                if is_ctrl || is_alt {
                    self.move_word_left();
                } else {
                    self.move_left();
                }
                true
            }
            KeyCode::Right => {
                if is_ctrl || is_alt {
                    self.move_word_right();
                } else {
                    self.move_right();
                }
                true
            }
            KeyCode::Home => {
                self.move_home();
                true
            }
            KeyCode::End => {
                self.move_end();
                true
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_buffer_insert_and_cursor() {
        let mut buf = TextBuffer::new("");
        assert_eq!(buf.cursor(), 0);
        assert_eq!(buf.len(), 0);

        buf.insert_char('H');
        buf.insert_char('i');
        assert_eq!(buf.content(), "Hi");
        assert_eq!(buf.cursor(), 2);

        buf.move_left();
        assert_eq!(buf.cursor(), 1);
        buf.insert_char('e');
        buf.insert_char('y');
        assert_eq!(buf.content(), "Heyi");
        assert_eq!(buf.cursor(), 3);
    }

    #[test]
    fn test_text_buffer_backspace_and_delete() {
        let mut buf = TextBuffer::new("Rust");
        assert_eq!(buf.cursor(), 4);

        assert!(buf.backspace());
        assert_eq!(buf.content(), "Rus");
        assert_eq!(buf.cursor(), 3);

        buf.set_cursor(1);
        assert!(buf.delete());
        assert_eq!(buf.content(), "Rs");
        assert_eq!(buf.cursor(), 1);
    }

    #[test]
    fn test_text_buffer_word_navigation_and_deletion() {
        let mut buf = TextBuffer::new("one two three");
        assert_eq!(buf.cursor(), 13);

        buf.move_word_left();
        assert_eq!(buf.cursor(), 8); // at start of "three"

        buf.move_word_left();
        assert_eq!(buf.cursor(), 4); // at start of "two"

        buf.move_word_right();
        assert_eq!(buf.cursor(), 8); // after "two "

        buf.move_end();
        assert!(buf.delete_word_left());
        assert_eq!(buf.content(), "one two ");
        assert_eq!(buf.cursor(), 8);
    }

    #[test]
    fn test_text_buffer_line_operations() {
        let mut buf = TextBuffer::new("first second third");
        buf.set_cursor(6); // at start of "second"

        assert!(buf.delete_to_end());
        assert_eq!(buf.content(), "first ");

        buf.insert_str("word");
        assert_eq!(buf.content(), "first word");
        buf.set_cursor(6);
        assert!(buf.delete_to_start());
        assert_eq!(buf.content(), "word");
        assert_eq!(buf.cursor(), 0);
    }

    #[test]
    fn test_text_buffer_viewport_scrolling() {
        let mut buf = TextBuffer::new("");
        let visible_width = 5;

        for c in "0123456789".chars() {
            buf.insert_char(c);
            buf.update_viewport(visible_width);
        }

        assert_eq!(buf.content(), "0123456789");
        assert_eq!(buf.cursor(), 10);
        assert_eq!(buf.scroll_offset(), 6);
        assert_eq!(buf.visible_text(visible_width), "6789");

        buf.move_home();
        buf.update_viewport(visible_width);
        assert_eq!(buf.cursor(), 0);
        assert_eq!(buf.scroll_offset(), 0);
        assert_eq!(buf.visible_text(visible_width), "01234");
    }

    #[test]
    fn test_text_buffer_unicode() {
        let mut buf = TextBuffer::new("🦀🚀");
        assert_eq!(buf.len(), 2);
        assert_eq!(buf.cursor(), 2);

        buf.move_left();
        assert_eq!(buf.cursor(), 1);
        buf.insert_char('✨');
        assert_eq!(buf.content(), "🦀✨🚀");
        assert_eq!(buf.cursor(), 2);

        assert!(buf.backspace());
        assert_eq!(buf.content(), "🦀🚀");
    }
}
