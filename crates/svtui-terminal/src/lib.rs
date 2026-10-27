use crossterm::{
    cursor::{self, MoveTo},
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    style::{
        Attribute, Color, Print, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor,
    },
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{self, Stdout, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use svtui_layout::{BorderChars, Rect};

pub use crossterm::style::Color as TerminalColor;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cell {
    pub ch: char,
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            ch: ' ',
            fg: None,
            bg: None,
            bold: false,
            italic: false,
            underline: false,
        }
    }
}

impl Cell {
    pub fn new(ch: char) -> Self {
        Self {
            ch,
            ..Default::default()
        }
    }

    pub fn with_fg(mut self, color: Color) -> Self {
        self.fg = Some(color);
        self
    }

    pub fn with_bg(mut self, color: Color) -> Self {
        self.bg = Some(color);
        self
    }

    pub fn with_bold(mut self, bold: bool) -> Self {
        self.bold = bold;
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Buffer {
    pub width: u16,
    pub height: u16,
    pub cells: Vec<Cell>,
    clip_stack: Vec<Rect>,
}

impl Buffer {
    pub fn new(width: u16, height: u16) -> Self {
        let size = (width as usize) * (height as usize);
        Self {
            width,
            height,
            cells: vec![Cell::default(); size],
            clip_stack: Vec::new(),
        }
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
        let size = (width as usize) * (height as usize);
        self.cells = vec![Cell::default(); size];
        self.clip_stack.clear();
    }

    pub fn clear(&mut self) {
        for cell in &mut self.cells {
            *cell = Cell::default();
        }
        self.clip_stack.clear();
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

static CLEANED_UP: AtomicBool = AtomicBool::new(false);

pub struct Terminal {
    stdout: Stdout,
    pub curr_buffer: Buffer,
    pub prev_buffer: Buffer,
    width: u16,
    height: u16,
    needs_full_repaint: bool,
}

impl Terminal {
    pub fn new() -> io::Result<Self> {
        Self::new_with_mouse(true)
    }

    pub fn new_with_mouse(enable_mouse: bool) -> io::Result<Self> {
        let (width, height) = terminal::size()?;
        let mut stdout = io::stdout();

        terminal::enable_raw_mode()?;
        if enable_mouse {
            execute!(
                stdout,
                EnterAlternateScreen,
                cursor::Hide,
                EnableMouseCapture
            )?;
        } else {
            execute!(stdout, EnterAlternateScreen, cursor::Hide)?;
        }

        let default_panic_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            Self::restore_terminal();
            default_panic_hook(info);
        }));

        CLEANED_UP.store(false, Ordering::SeqCst);

        Ok(Self {
            stdout,
            curr_buffer: Buffer::new(width, height),
            prev_buffer: Buffer::new(width, height),
            width,
            height,
            needs_full_repaint: true,
        })
    }

    pub fn size(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
        self.curr_buffer.resize(width, height);
        self.prev_buffer.resize(width, height);
        self.needs_full_repaint = true;
    }

    pub fn request_full_repaint(&mut self) {
        self.needs_full_repaint = true;
    }

    pub fn flush(&mut self) -> io::Result<()> {
        let width = self.width;
        let height = self.height;

        if self.needs_full_repaint {
            execute!(
                self.stdout,
                ResetColor,
                SetAttribute(Attribute::Reset),
                terminal::Clear(terminal::ClearType::All)
            )?;
            self.prev_buffer.clear();
            self.needs_full_repaint = false;
        }

        let mut last_fg: Option<Color> = None;
        let mut last_bg: Option<Color> = None;
        let mut last_bold = false;

        for y in 0..height {
            for x in 0..width {
                let curr_cell = self.curr_buffer.get(x, y).copied().unwrap_or_default();
                let prev_cell = self.prev_buffer.get(x, y).copied().unwrap_or_default();

                if curr_cell != prev_cell {
                    execute!(self.stdout, MoveTo(x, y))?;

                    if curr_cell.fg != last_fg {
                        match curr_cell.fg {
                            Some(c) => execute!(self.stdout, SetForegroundColor(c))?,
                            None => execute!(self.stdout, ResetColor)?,
                        }
                        last_fg = curr_cell.fg;
                    }

                    if curr_cell.bg != last_bg {
                        match curr_cell.bg {
                            Some(c) => execute!(self.stdout, SetBackgroundColor(c))?,
                            None => execute!(self.stdout, ResetColor)?,
                        }
                        last_bg = curr_cell.bg;
                    }

                    if curr_cell.bold != last_bold {
                        if curr_cell.bold {
                            execute!(self.stdout, SetAttribute(Attribute::Bold))?;
                        } else {
                            execute!(self.stdout, SetAttribute(Attribute::NormalIntensity))?;
                        }
                        last_bold = curr_cell.bold;
                    }

                    execute!(self.stdout, Print(curr_cell.ch))?;
                }
            }
        }

        execute!(self.stdout, ResetColor, SetAttribute(Attribute::Reset))?;
        self.stdout.flush()?;

        self.prev_buffer.cells.clone_from(&self.curr_buffer.cells);

        Ok(())
    }

    fn restore_terminal() {
        if !CLEANED_UP.swap(true, Ordering::SeqCst) {
            let mut stdout = io::stdout();
            let _ = execute!(
                stdout,
                ResetColor,
                SetAttribute(Attribute::Reset),
                cursor::Show,
                DisableMouseCapture,
                LeaveAlternateScreen
            );
            let _ = stdout.flush();
            let _ = terminal::disable_raw_mode();
        }
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        Self::restore_terminal();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use svtui_layout::BorderStyle;

    #[test]
    fn test_buffer_set_string() {
        let mut buf = Buffer::new(20, 5);
        buf.set_string(2, 2, "Hello", None, None, false, 10);
        assert_eq!(buf.get(2, 2).unwrap().ch, 'H');
        assert_eq!(buf.get(3, 2).unwrap().ch, 'e');
        assert_eq!(buf.get(4, 2).unwrap().ch, 'l');
        assert_eq!(buf.get(5, 2).unwrap().ch, 'l');
        assert_eq!(buf.get(6, 2).unwrap().ch, 'o');
        assert_eq!(buf.get(7, 2).unwrap().ch, ' ');
    }

    #[test]
    fn test_buffer_clipping() {
        let mut buf = Buffer::new(20, 10);
        buf.push_clip(Rect::new(2, 2, 4, 3));

        // Drawing inside clip (3, 3) should work
        buf.set_char(3, 3, 'X', None, None, false);
        assert_eq!(buf.get(3, 3).unwrap().ch, 'X');

        // Drawing outside clip (1, 1) should be ignored
        buf.set_char(1, 1, 'Y', None, None, false);
        assert_eq!(buf.get(1, 1).unwrap().ch, ' ');

        // Drawing outside clip (10, 3) should be ignored
        buf.set_char(10, 3, 'Z', None, None, false);
        assert_eq!(buf.get(10, 3).unwrap().ch, ' ');

        buf.pop_clip();
        // After popping, clip is restored to whole buffer
        buf.set_char(1, 1, 'Y', None, None, false);
        assert_eq!(buf.get(1, 1).unwrap().ch, 'Y');
    }

    #[test]
    fn test_buffer_draw_border() {
        let mut buf = Buffer::new(10, 5);
        let rect = Rect::new(0, 0, 6, 3);
        let chars = BorderStyle::Rounded.chars().unwrap();
        buf.draw_border(rect, chars, None);

        assert_eq!(buf.get(0, 0).unwrap().ch, '╭');
        assert_eq!(buf.get(5, 0).unwrap().ch, '╮');
        assert_eq!(buf.get(0, 2).unwrap().ch, '╰');
        assert_eq!(buf.get(5, 2).unwrap().ch, '╯');
        assert_eq!(buf.get(1, 0).unwrap().ch, '─');
        assert_eq!(buf.get(0, 1).unwrap().ch, '│');
    }
}
