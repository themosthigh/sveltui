use crate::layout::{BorderChars, Rect};
use crate::terminal::cell::Cell;
use crossterm::style::Color;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Buffer {
    pub width: u16,
    pub height: u16,
    pub cells: Vec<Cell>,
    pub cursor: Option<(u16, u16)>,
    clip_stack: Vec<Rect>,
}

impl Buffer {
    pub fn new(width: u16, height: u16) -> Self {
        let size = (width as usize) * (height as usize);
        Self {
            width,
            height,
            cells: vec![Cell::default(); size],
            cursor: None,
            clip_stack: Vec::new(),
        }
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
        let size = (width as usize) * (height as usize);
        self.cells = vec![Cell::default(); size];
        self.cursor = None;
        self.clip_stack.clear();
    }

    pub fn clear(&mut self) {
        for cell in &mut self.cells {
            *cell = Cell::default();
        }
        self.cursor = None;
        self.clip_stack.clear();
    }

    pub fn set_cursor(&mut self, x: u16, y: u16) {
        self.cursor = Some((x, y));
    }

    pub fn clear_cursor(&mut self) {
        self.cursor = None;
    }

    pub fn cursor(&self) -> Option<(u16, u16)> {
        self.cursor
    }

    pub fn push_clip(&mut self, rect: Rect) {
        let current = self.active_clip();
        let intersected = current.intersect(&rect).unwrap_or(Rect::new(0, 0, 0, 0));
        self.clip_stack.push(intersected);
    }

    pub fn pop_clip(&mut self) {
        self.clip_stack.pop();
    }

    pub fn active_clip(&self) -> Rect {
        self.clip_stack
            .last()
            .copied()
            .unwrap_or(Rect::new(0, 0, self.width, self.height))
    }

    #[inline]
    fn index_of(&self, x: u16, y: u16) -> Option<usize> {
        if x < self.width && y < self.height {
            Some((y as usize) * (self.width as usize) + (x as usize))
        } else {
            None
        }
    }

    pub fn get(&self, x: u16, y: u16) -> Option<&Cell> {
        self.index_of(x, y).map(|idx| &self.cells[idx])
    }

    pub fn get_mut(&mut self, x: u16, y: u16) -> Option<&mut Cell> {
        self.index_of(x, y).map(|idx| &mut self.cells[idx])
    }

    pub fn set_cell(&mut self, x: u16, y: u16, cell: Cell) {
        if let Some(idx) = self
            .index_of(x, y)
            .filter(|_| self.active_clip().contains(x, y))
        {
            self.cells[idx] = cell;
        }
    }

    pub fn set_char(
        &mut self,
        x: u16,
        y: u16,
        ch: char,
        fg: Option<Color>,
        bg: Option<Color>,
        bold: bool,
    ) {
        if !self.active_clip().contains(x, y) {
            return;
        }
        if let Some(cell) = self.get_mut(x, y) {
            cell.ch = ch;
            cell.fg = fg;
            cell.bg = bg;
            cell.bold = bold;
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn set_string(
        &mut self,
        x: u16,
        y: u16,
        text: &str,
        fg: Option<Color>,
        bg: Option<Color>,
        bold: bool,
        max_width: u16,
    ) {
        let clip = self.active_clip();
        if y < clip.y || y >= clip.y.saturating_add(clip.height) {
            return;
        }
        for (idx, ch) in text.chars().enumerate() {
            let curr_x = x.saturating_add(idx as u16);
            if curr_x >= x.saturating_add(max_width) || curr_x >= self.width {
                break;
            }
            self.set_char(curr_x, y, ch, fg, bg, bold);
        }
    }

    pub fn draw_border(&mut self, rect: Rect, border: BorderChars, color: Option<Color>) {
        if rect.width < 2 || rect.height < 2 {
            return;
        }

        let right = rect.x + rect.width - 1;
        let bottom = rect.y + rect.height - 1;

        // Corners
        self.set_char(rect.x, rect.y, border.top_left, color, None, false);
        self.set_char(right, rect.y, border.top_right, color, None, false);
        self.set_char(rect.x, bottom, border.bottom_left, color, None, false);
        self.set_char(right, bottom, border.bottom_right, color, None, false);

        // Top and bottom horizontal edges
        for x in (rect.x + 1)..right {
            self.set_char(x, rect.y, border.horizontal, color, None, false);
            self.set_char(x, bottom, border.horizontal, color, None, false);
        }

        // Left and right vertical edges
        for y in (rect.y + 1)..bottom {
            self.set_char(rect.x, y, border.vertical, color, None, false);
            self.set_char(right, y, border.vertical, color, None, false);
        }
    }

    pub fn draw_title(&mut self, rect: Rect, title: &str, color: Option<Color>) {
        if rect.width <= 4 || title.is_empty() {
            return;
        }

        let title_text = format!(" {} ", title);
        let max_len = (rect.width.saturating_sub(4)) as usize;
        let displayed_title: String = title_text.chars().take(max_len).collect();

        let start_x = rect.x + 2;
        self.set_string(
            start_x,
            rect.y,
            &displayed_title,
            color,
            None,
            true,
            max_len as u16,
        );
    }
}
