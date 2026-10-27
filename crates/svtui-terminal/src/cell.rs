use crossterm::style::Color;

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
