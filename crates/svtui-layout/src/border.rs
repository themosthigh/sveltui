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
