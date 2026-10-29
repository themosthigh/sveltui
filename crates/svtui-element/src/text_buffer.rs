use crate::focus::FocusId;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    pub(crate) static BUFFER_MAP: RefCell<HashMap<FocusId, TextBuffer>> = RefCell::new(HashMap::new());
}

pub fn with_buffer<R>(id: FocusId, f: impl FnOnce(&TextBuffer) -> R) -> R {
    BUFFER_MAP.with(|m| {
        let mut map = m.borrow_mut();
        let buf = map.entry(id).or_default();
        f(buf)
    })
}

pub fn with_buffer_mut<R>(id: FocusId, f: impl FnOnce(&mut TextBuffer) -> R) -> R {
    BUFFER_MAP.with(|m| {
        let mut map = m.borrow_mut();
        let buf = map.entry(id).or_default();
        f(buf)
    })
}

pub fn get_buffer(id: FocusId) -> TextBuffer {
    with_buffer(id, |b| b.clone())
}

pub fn set_buffer(id: FocusId, buf: TextBuffer) {
    BUFFER_MAP.with(|m| {
        m.borrow_mut().insert(id, buf);
    });
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextBuffer {
    lines: Vec<String>,
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub scroll_row: usize,
    pub scroll_col: usize,
    preferred_col: Option<usize>,
    multiline: bool,
}

impl Default for TextBuffer {
    fn default() -> Self {
        Self::new("")
    }
}

pub fn char_to_byte_index(s: &str, char_idx: usize) -> usize {
    s.char_indices()
        .nth(char_idx)
        .map(|(b, _)| b)
        .unwrap_or(s.len())
}

impl TextBuffer {
    pub fn new(content: impl Into<String>) -> Self {
        let s = content.into();
        let lines: Vec<String> = if s.is_empty() {
            vec![String::new()]
        } else {
            let mut l: Vec<String> = s.lines().map(|line| line.to_string()).collect();
            if s.ends_with('\n') {
                l.push(String::new());
            }
            if l.is_empty() { vec![String::new()] } else { l }
        };
        let cursor_row = lines.len().saturating_sub(1);
        let cursor_col = lines[cursor_row].chars().count();
        Self {
            lines,
            cursor_row,
            cursor_col,
            scroll_row: 0,
            scroll_col: 0,
            preferred_col: None,
            multiline: true,
        }
    }

    pub fn multiline(&self) -> bool {
        self.multiline
    }

    pub fn set_multiline(&mut self, multiline: bool) {
        self.multiline = multiline;
    }

    pub fn content(&self) -> String {
        self.lines.join("\n")
    }

    pub fn as_str(&self) -> String {
        self.lines.join("\n")
    }

    pub fn set_content(&mut self, text: impl Into<String>) {
        let s = text.into();
        self.lines = if s.is_empty() {
            vec![String::new()]
        } else {
            let mut l: Vec<String> = s.lines().map(|line| line.to_string()).collect();
            if s.ends_with('\n') {
                l.push(String::new());
            }
            if l.is_empty() { vec![String::new()] } else { l }
        };
        let max_row = self.lines.len().saturating_sub(1);
        self.cursor_row = self.cursor_row.min(max_row);
        self.cursor_col = self.cursor_col.min(self.line_len(self.cursor_row));
        self.preferred_col = None;
    }

    pub fn lines(&self) -> &[String] {
        &self.lines
    }

    pub fn line(&self, row: usize) -> Option<&str> {
        self.lines.get(row).map(|s| s.as_str())
    }

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    pub fn line_len(&self, row: usize) -> usize {
        self.lines.get(row).map(|l| l.chars().count()).unwrap_or(0)
    }

    pub fn len(&self) -> usize {
        if self.lines.is_empty() {
            return 0;
        }
        let chars: usize = self.lines.iter().map(|l| l.chars().count()).sum();
        chars + (self.lines.len() - 1)
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty() || (self.lines.len() == 1 && self.lines[0].is_empty())
    }

    pub fn cursor(&self) -> usize {
        self.cursor_col
    }

    pub fn set_cursor(&mut self, pos: usize) {
        self.cursor_col = pos.min(self.line_len(self.cursor_row));
        self.preferred_col = None;
    }

    pub fn cursor_pos(&self) -> (usize, usize) {
        (self.cursor_row, self.cursor_col)
    }

    pub fn set_cursor_pos(&mut self, row: usize, col: usize) {
        let max_row = self.lines.len().saturating_sub(1);
        self.cursor_row = row.min(max_row);
        self.cursor_col = col.min(self.line_len(self.cursor_row));
        self.preferred_col = None;
    }

    pub fn scroll_offset(&self) -> usize {
        self.scroll_col
    }

    pub fn set_scroll_offset(&mut self, offset: usize) {
        self.scroll_col = offset;
    }

    pub fn scroll_pos(&self) -> (usize, usize) {
        (self.scroll_row, self.scroll_col)
    }

    pub fn set_scroll_pos(&mut self, row: usize, col: usize) {
        self.scroll_row = row;
        self.scroll_col = col;
    }

    pub fn move_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
            self.preferred_col = None;
        } else if self.multiline && self.cursor_row > 0 {
            self.cursor_row -= 1;
            self.cursor_col = self.line_len(self.cursor_row);
            self.preferred_col = None;
        }
    }

    pub fn move_right(&mut self) {
        let cur_line_len = self.line_len(self.cursor_row);
        if self.cursor_col < cur_line_len {
            self.cursor_col += 1;
            self.preferred_col = None;
        } else if self.multiline && self.cursor_row + 1 < self.lines.len() {
            self.cursor_row += 1;
            self.cursor_col = 0;
            self.preferred_col = None;
        }
    }

    pub fn move_up(&mut self) {
        if self.cursor_row > 0 {
            let target_col = self.preferred_col.unwrap_or(self.cursor_col);
            self.preferred_col = Some(target_col);
            self.cursor_row -= 1;
            self.cursor_col = target_col.min(self.line_len(self.cursor_row));
        }
    }

    pub fn move_down(&mut self) {
        if self.cursor_row + 1 < self.lines.len() {
            let target_col = self.preferred_col.unwrap_or(self.cursor_col);
            self.preferred_col = Some(target_col);
            self.cursor_row += 1;
            self.cursor_col = target_col.min(self.line_len(self.cursor_row));
        }
    }

    pub fn move_home(&mut self) {
        self.cursor_col = 0;
        self.preferred_col = None;
    }

    pub fn move_end(&mut self) {
        self.cursor_col = self.line_len(self.cursor_row);
        self.preferred_col = None;
    }

    pub fn move_top(&mut self) {
        self.cursor_row = 0;
        self.cursor_col = 0;
        self.preferred_col = None;
    }

    pub fn move_bottom(&mut self) {
        self.cursor_row = self.lines.len().saturating_sub(1);
        self.cursor_col = self.line_len(self.cursor_row);
        self.preferred_col = None;
    }

    pub fn page_up(&mut self, page_size: usize) {
        let target_col = self.preferred_col.unwrap_or(self.cursor_col);
        self.preferred_col = Some(target_col);
        self.cursor_row = self.cursor_row.saturating_sub(page_size);
        self.cursor_col = target_col.min(self.line_len(self.cursor_row));
    }

    pub fn page_down(&mut self, page_size: usize) {
        let target_col = self.preferred_col.unwrap_or(self.cursor_col);
        self.preferred_col = Some(target_col);
        let max_row = self.lines.len().saturating_sub(1);
        self.cursor_row = (self.cursor_row + page_size).min(max_row);
        self.cursor_col = target_col.min(self.line_len(self.cursor_row));
    }

    pub fn move_word_left(&mut self) {
        if self.cursor_col == 0 {
            if self.multiline && self.cursor_row > 0 {
                self.move_left();
            }
            return;
        }
        let line = &self.lines[self.cursor_row];
        let chars: Vec<char> = line.chars().collect();
        let mut idx = self.cursor_col;

        while idx > 0 && chars[idx - 1].is_whitespace() {
            idx -= 1;
        }
        while idx > 0 && !chars[idx - 1].is_whitespace() {
            idx -= 1;
        }
        self.cursor_col = idx;
        self.preferred_col = None;
    }

    pub fn move_word_right(&mut self) {
        let cur_len = self.line_len(self.cursor_row);
        if self.cursor_col >= cur_len {
            if self.multiline && self.cursor_row + 1 < self.lines.len() {
                self.move_right();
            }
            return;
        }
        let line = &self.lines[self.cursor_row];
        let chars: Vec<char> = line.chars().collect();
        let mut idx = self.cursor_col;

        while idx < cur_len && !chars[idx].is_whitespace() {
            idx += 1;
        }
        while idx < cur_len && chars[idx].is_whitespace() {
            idx += 1;
        }
        self.cursor_col = idx;
        self.preferred_col = None;
    }

    pub fn insert_char(&mut self, c: char) {
        if c == '\n' {
            if !self.multiline {
                return;
            }
            let byte_pos = char_to_byte_index(&self.lines[self.cursor_row], self.cursor_col);
            let rest = self.lines[self.cursor_row][byte_pos..].to_string();
            self.lines[self.cursor_row].truncate(byte_pos);
            self.cursor_row += 1;
            self.lines.insert(self.cursor_row, rest);
            self.cursor_col = 0;
            self.preferred_col = None;
        } else {
            let byte_pos = char_to_byte_index(&self.lines[self.cursor_row], self.cursor_col);
            self.lines[self.cursor_row].insert(byte_pos, c);
            self.cursor_col += 1;
            self.preferred_col = None;
        }
    }

    pub fn insert_str(&mut self, s: &str) {
        for c in s.chars() {
            self.insert_char(c);
        }
    }

    pub fn backspace(&mut self) -> bool {
        if self.cursor_col > 0 {
            let del_idx = self.cursor_col - 1;
            let start_byte = char_to_byte_index(&self.lines[self.cursor_row], del_idx);
            let end_byte = char_to_byte_index(&self.lines[self.cursor_row], self.cursor_col);
            self.lines[self.cursor_row].drain(start_byte..end_byte);
            self.cursor_col = del_idx;
            self.preferred_col = None;
            true
        } else if self.multiline && self.cursor_row > 0 {
            let curr_line = self.lines.remove(self.cursor_row);
            self.cursor_row -= 1;
            self.cursor_col = self.line_len(self.cursor_row);
            self.lines[self.cursor_row].push_str(&curr_line);
            self.preferred_col = None;
            true
        } else {
            false
        }
    }

    pub fn delete(&mut self) -> bool {
        let cur_len = self.line_len(self.cursor_row);
        if self.cursor_col < cur_len {
            let start_byte = char_to_byte_index(&self.lines[self.cursor_row], self.cursor_col);
            let end_byte = char_to_byte_index(&self.lines[self.cursor_row], self.cursor_col + 1);
            self.lines[self.cursor_row].drain(start_byte..end_byte);
            self.preferred_col = None;
            true
        } else if self.multiline && self.cursor_row + 1 < self.lines.len() {
            let next_line = self.lines.remove(self.cursor_row + 1);
            self.lines[self.cursor_row].push_str(&next_line);
            self.preferred_col = None;
            true
        } else {
            false
        }
    }

    pub fn delete_word_left(&mut self) -> bool {
        if self.cursor_col > 0 {
            let old_col = self.cursor_col;
            self.move_word_left();
            let new_col = self.cursor_col;
            let start_byte = char_to_byte_index(&self.lines[self.cursor_row], new_col);
            let end_byte = char_to_byte_index(&self.lines[self.cursor_row], old_col);
            self.lines[self.cursor_row].drain(start_byte..end_byte);
            self.preferred_col = None;
            true
        } else if self.multiline && self.cursor_row > 0 {
            self.backspace()
        } else {
            false
        }
    }

    pub fn delete_to_end(&mut self) -> bool {
        let cur_len = self.line_len(self.cursor_row);
        if self.cursor_col < cur_len {
            let start_byte = char_to_byte_index(&self.lines[self.cursor_row], self.cursor_col);
            self.lines[self.cursor_row].truncate(start_byte);
            self.preferred_col = None;
            true
        } else if self.multiline && self.cursor_row + 1 < self.lines.len() {
            let next_line = self.lines.remove(self.cursor_row + 1);
            self.lines[self.cursor_row].push_str(&next_line);
            self.preferred_col = None;
            true
        } else {
            false
        }
    }

    pub fn delete_to_start(&mut self) -> bool {
        if self.cursor_col > 0 {
            let end_byte = char_to_byte_index(&self.lines[self.cursor_row], self.cursor_col);
            self.lines[self.cursor_row].drain(0..end_byte);
            self.cursor_col = 0;
            self.preferred_col = None;
            true
        } else {
            false
        }
    }

    pub fn clear(&mut self) {
        self.lines = vec![String::new()];
        self.cursor_row = 0;
        self.cursor_col = 0;
        self.scroll_row = 0;
        self.scroll_col = 0;
        self.preferred_col = None;
    }

    pub fn update_viewport(&mut self, visible_width: usize, visible_height: usize) {
        if visible_height > 0 {
            if self.cursor_row < self.scroll_row {
                self.scroll_row = self.cursor_row;
            } else if self.cursor_row >= self.scroll_row + visible_height {
                self.scroll_row = self.cursor_row.saturating_sub(visible_height) + 1;
            }
        }
        if visible_width > 0 {
            if self.cursor_col < self.scroll_col {
                self.scroll_col = self.cursor_col;
            } else if self.cursor_col >= self.scroll_col + visible_width {
                self.scroll_col = self.cursor_col.saturating_sub(visible_width) + 1;
            }
        }
    }

    pub fn visible_text(&self, visible_width: usize) -> String {
        if visible_width == 0 || self.lines.is_empty() {
            return String::new();
        }
        let row = self.cursor_row.min(self.lines.len().saturating_sub(1));
        let line = &self.lines[row];
        let start_byte = char_to_byte_index(line, self.scroll_col);
        let end_byte = char_to_byte_index(line, self.scroll_col + visible_width);
        line[start_byte..end_byte].to_string()
    }

    pub fn visible_lines(&self, visible_width: usize, visible_height: usize) -> Vec<String> {
        if visible_height == 0 || self.lines.is_empty() {
            return Vec::new();
        }
        let end_row = (self.scroll_row + visible_height).min(self.lines.len());
        let mut result = Vec::with_capacity(end_row.saturating_sub(self.scroll_row));
        for r in self.scroll_row..end_row {
            let line = &self.lines[r];
            let start_byte = char_to_byte_index(line, self.scroll_col);
            let end_byte = char_to_byte_index(line, self.scroll_col + visible_width);
            result.push(line[start_byte..end_byte].to_string());
        }
        result
    }

    /// Whether the editor owns this key, including no-op edits at boundaries.
    /// Tab remains available for focus traversal in the standard components.
    pub fn accepts_key(&self, key: &KeyEvent) -> bool {
        let control = key.modifiers.contains(KeyModifiers::CONTROL);
        let alt = key.modifiers.contains(KeyModifiers::ALT);
        match key.code {
            KeyCode::Char(c) if control => matches!(c, 'w' | 'k' | 'u' | 'a' | 'e' | 'h' | 'd'),
            KeyCode::Char(_) => !alt,
            KeyCode::Enter | KeyCode::Up | KeyCode::Down | KeyCode::PageUp | KeyCode::PageDown => {
                self.multiline
            }
            KeyCode::Backspace
            | KeyCode::Delete
            | KeyCode::Left
            | KeyCode::Right
            | KeyCode::Home
            | KeyCode::End => true,
            _ => false,
        }
    }

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
            KeyCode::Enter => {
                if self.multiline {
                    self.insert_char('\n');
                    true
                } else {
                    false
                }
            }
            KeyCode::Tab => {
                if self.multiline {
                    self.insert_str("    ");
                    true
                } else {
                    false
                }
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
            KeyCode::Up => {
                if is_ctrl {
                    self.move_top();
                } else {
                    self.move_up();
                }
                true
            }
            KeyCode::Down => {
                if is_ctrl {
                    self.move_bottom();
                } else {
                    self.move_down();
                }
                true
            }
            KeyCode::PageUp => {
                self.page_up(10);
                true
            }
            KeyCode::PageDown => {
                self.page_down(10);
                true
            }
            KeyCode::Home => {
                if is_ctrl {
                    self.move_top();
                } else {
                    self.move_home();
                }
                true
            }
            KeyCode::End => {
                if is_ctrl {
                    self.move_bottom();
                } else {
                    self.move_end();
                }
                true
            }
            _ => false,
        }
    }
}
