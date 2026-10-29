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
