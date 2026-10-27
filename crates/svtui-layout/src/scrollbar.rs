use crossterm::style::Color;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Overflow {
    #[default]
    Visible,
    Hidden,
    Scroll,
    Auto,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ScrollbarVisibility {
    #[default]
    Auto,
    Always,
    Never,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScrollbarChars {
    pub vertical_track: char,
    pub vertical_thumb: char,
    pub horizontal_track: char,
    pub horizontal_thumb: char,
    pub corner: char,
}

impl ScrollbarChars {
    /// Half-block horizontal and full-block vertical for exact 1:1 pixel thickness parity:
    /// - Vertical: █ (track, dim) and █ (thumb, bright) - full cell width (~8-10px)
    /// - Horizontal: ▄ (track, dim) and ▄ (thumb, bright) - half cell height (~8-10px, not full cell height)
    /// - Corner: ▄
    pub const fn half_blocks() -> Self {
        Self {
            vertical_track: '█',
            vertical_thumb: '█',
            horizontal_track: '▄',
            horizontal_thumb: '▄',
            corner: '▄',
        }
    }

    /// Standard box-drawing line characters with matching 1:1 stroke width UI parity:
    /// Vertical: │ (track), ┃ (thumb)
    /// Horizontal: ─ (track), ━ (thumb)
    /// Corner: ┘
    pub const fn lines() -> Self {
        Self {
            vertical_track: '│',
            vertical_thumb: '┃',
            horizontal_track: '─',
            horizontal_thumb: '━',
            corner: '┘',
        }
    }

    /// Double line characters for high-contrast borders:
    /// Vertical: │ (track), ║ (thumb)
    /// Horizontal: ─ (track), ═ (thumb)
    /// Corner: ╝
    pub const fn double() -> Self {
        Self {
            vertical_track: '│',
            vertical_thumb: '║',
            horizontal_track: '─',
            horizontal_thumb: '═',
            corner: '╝',
        }
    }

    /// Block shading characters:
    /// Track: ░, Thumb: █
    pub const fn block() -> Self {
        Self {
            vertical_track: '░',
            vertical_thumb: '█',
            horizontal_track: '░',
            horizontal_thumb: '█',
            corner: '░',
        }
    }

    pub fn new(
        vertical_track: char,
        vertical_thumb: char,
        horizontal_track: char,
        horizontal_thumb: char,
    ) -> Self {
        Self {
            vertical_track,
            vertical_thumb,
            horizontal_track,
            horizontal_thumb,
            corner: '┘',
        }
    }
}

impl Default for ScrollbarChars {
    fn default() -> Self {
        Self::half_blocks()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScrollbarStyle {
    pub chars: ScrollbarChars,
    pub track_color: Option<Color>,
    pub thumb_color: Option<Color>,
}

impl ScrollbarStyle {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn chars(mut self, chars: ScrollbarChars) -> Self {
        self.chars = chars;
        self
    }

    pub fn track_color(mut self, color: Color) -> Self {
        self.track_color = Some(color);
        self
    }

    pub fn thumb_color(mut self, color: Color) -> Self {
        self.thumb_color = Some(color);
        self
    }
}

impl Default for ScrollbarStyle {
    fn default() -> Self {
        Self {
            chars: ScrollbarChars::default(),
            track_color: Some(Color::DarkGrey),
            thumb_color: Some(Color::White),
        }
    }
}
