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
