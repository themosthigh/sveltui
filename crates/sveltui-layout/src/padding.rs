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
