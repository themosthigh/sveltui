use crossterm::style::Color;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl Rect {
    pub const fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn inner(&self, top: u16, right: u16, bottom: u16, left: u16) -> Rect {
        let x = self.x.saturating_add(left);
        let y = self.y.saturating_add(top);
        let width = self.width.saturating_sub(left.saturating_add(right));
        let height = self.height.saturating_sub(top.saturating_add(bottom));
        Rect {
            x,
            y,
            width,
            height,
        }
    }

    pub fn contains(&self, x: u16, y: u16) -> bool {
        x >= self.x
            && x < self.x.saturating_add(self.width)
            && y >= self.y
            && y < self.y.saturating_add(self.height)
    }

    pub fn intersect(&self, other: &Rect) -> Option<Rect> {
        let left = self.x.max(other.x);
        let top = self.y.max(other.y);
        let right = (self.x.saturating_add(self.width)).min(other.x.saturating_add(other.width));
        let bottom = (self.y.saturating_add(self.height)).min(other.y.saturating_add(other.height));

        if right > left && bottom > top {
            Some(Rect {
                x: left,
                y: top,
                width: right - left,
                height: bottom - top,
            })
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Align {
    #[default]
    Start,
    Center,
    End,
    Stretch,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Justify {
    #[default]
    Start,
    Center,
    End,
    SpaceBetween,
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum Size {
    #[default]
    Auto,
    Fixed(u16),
    Percent(f32),
    Fill,
}

impl Size {
    pub fn resolve(&self, parent: u16, content: u16) -> u16 {
        match self {
            Size::Fixed(w) => *w,
            Size::Percent(p) => ((parent as f32 * p / 100.0).round() as u16).min(parent),
            Size::Fill => parent,
            Size::Auto => content,
        }
    }
}

pub fn clamp_dimension(val: u16, min: Option<Size>, max: Option<Size>, parent: u16) -> u16 {
    let mut result = val;
    if let Some(min_s) = min {
        let min_v = min_s.resolve(parent, 0);
        result = result.max(min_v);
    }
    if let Some(max_s) = max {
        let max_v = max_s.resolve(parent, parent);
        result = result.min(max_v);
    }
    result
}

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

impl From<u16> for Size {
    fn from(v: u16) -> Self {
        Size::Fixed(v)
    }
}

impl From<i32> for Size {
    fn from(v: i32) -> Self {
        Size::Fixed(v.max(0) as u16)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Padding {
    pub top: u16,
    pub right: u16,
    pub bottom: u16,
    pub left: u16,
}

impl Padding {
    pub const fn zero() -> Self {
        Self {
            top: 0,
            right: 0,
            bottom: 0,
            left: 0,
        }
    }

    pub const fn all(val: u16) -> Self {
        Self {
            top: val,
            right: val,
            bottom: val,
            left: val,
        }
    }

    pub const fn symmetric(horizontal: u16, vertical: u16) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }
}

impl From<u16> for Padding {
    fn from(val: u16) -> Self {
        Padding::all(val)
    }
}

impl From<(u16, u16)> for Padding {
    fn from((h, v): (u16, u16)) -> Self {
        Padding::symmetric(h, v)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum BorderStyle {
    #[default]
    None,
    Single,
    Rounded,
    Double,
    Thick,
    Ascii,
}

impl BorderStyle {
    pub fn chars(&self) -> Option<BorderChars> {
        match self {
            BorderStyle::None => None,
            BorderStyle::Single => Some(BorderChars {
                top_left: '┌',
                top_right: '┐',
                bottom_left: '└',
                bottom_right: '┘',
                horizontal: '─',
                vertical: '│',
            }),
            BorderStyle::Rounded => Some(BorderChars {
                top_left: '╭',
                top_right: '╮',
                bottom_left: '╰',
                bottom_right: '╯',
                horizontal: '─',
                vertical: '│',
            }),
            BorderStyle::Double => Some(BorderChars {
                top_left: '╔',
                top_right: '╗',
                bottom_left: '╚',
                bottom_right: '╝',
                horizontal: '═',
                vertical: '║',
            }),
            BorderStyle::Thick => Some(BorderChars {
                top_left: '┏',
                top_right: '┓',
                bottom_left: '┗',
                bottom_right: '┛',
                horizontal: '━',
                vertical: '┃',
            }),
            BorderStyle::Ascii => Some(BorderChars {
                top_left: '+',
                top_right: '+',
                bottom_left: '+',
                bottom_right: '+',
                horizontal: '-',
                vertical: '|',
            }),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BorderChars {
    pub top_left: char,
    pub top_right: char,
    pub bottom_left: char,
    pub bottom_right: char,
    pub horizontal: char,
    pub vertical: char,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect_inner() {
        let r = Rect::new(5, 5, 20, 10);
        let inner = r.inner(1, 2, 3, 4);
        assert_eq!(inner.x, 9);
        assert_eq!(inner.y, 6);
        assert_eq!(inner.width, 14);
        assert_eq!(inner.height, 6);
    }

    #[test]
    fn test_border_chars() {
        let rounded = BorderStyle::Rounded.chars().unwrap();
        assert_eq!(rounded.top_left, '╭');
        assert_eq!(rounded.top_right, '╮');
        assert_eq!(rounded.bottom_left, '╰');
        assert_eq!(rounded.bottom_right, '╯');
        assert_eq!(rounded.horizontal, '─');
        assert_eq!(rounded.vertical, '│');
    }

    #[test]
    fn test_rect_intersect() {
        let a = Rect::new(0, 0, 10, 10);
        let b = Rect::new(5, 5, 10, 10);
        let inter = a.intersect(&b).unwrap();
        assert_eq!(inter, Rect::new(5, 5, 5, 5));

        let c = Rect::new(20, 20, 5, 5);
        assert_eq!(a.intersect(&c), None);
    }

    #[test]
    fn test_clamp_dimension() {
        // Val=10, min=15, max=25 => 15
        assert_eq!(
            clamp_dimension(10, Some(Size::Fixed(15)), Some(Size::Fixed(25)), 100),
            15
        );
        // Val=30, min=15, max=25 => 25
        assert_eq!(
            clamp_dimension(30, Some(Size::Fixed(15)), Some(Size::Fixed(25)), 100),
            25
        );
        // Val=20, min=15, max=25 => 20
        assert_eq!(
            clamp_dimension(20, Some(Size::Fixed(15)), Some(Size::Fixed(25)), 100),
            20
        );
    }
}
