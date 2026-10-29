use crate::layout::{Align, Justify, Rect};

/// The main axis of a single-line flex container. Children retain source order.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FlexDirection {
    /// Stack children top to bottom (main axis Y, cross axis X).
    #[default]
    Column,
    /// Place children left to right (main axis X, cross axis Y).
    Row,
}

impl FlexDirection {
    /// Convert physical width/height to main/cross dimensions, or vice versa.
    pub fn axes<T>(self, width: T, height: T) -> (T, T) {
        match self {
            Self::Column => (height, width),
            Self::Row => (width, height),
        }
    }
}

/// A child's main-axis size and limits, in terminal cells.
#[derive(Clone, Copy, Debug)]
pub struct FlexItem {
    pub size: u16,
    pub min: u16,
    pub max: u16,
    pub fill: bool,
}

/// Divide remaining main-axis space equally among Fill children, respecting
/// limits. Minimums win over maximums. Fixed children never implicitly shrink.
pub fn resolve_flex_sizes(items: &mut [FlexItem], available: u16, gap: u16) {
    let gaps = u64::from(gap) * items.len().saturating_sub(1) as u64;
    let fixed: u64 = items
        .iter()
        .filter(|i| !i.fill)
        .map(|i| u64::from(i.size))
        .sum();
    let budget = u64::from(available).saturating_sub(gaps + fixed);
    let total_at = |level: u64| -> u64 {
        items
            .iter()
            .filter(|item| item.fill)
            .map(|item| level.clamp(u64::from(item.min), u64::from(item.max.max(item.min))))
            .sum()
    };
    // Find the highest common water level that fits. Clamping at each level
    // handles lower and upper bounds together without prematurely freezing a
    // minimum that could grow after another child reaches its maximum.
    let mut low = 0;
    let mut high = budget + 1;
    while low + 1 < high {
        let middle = (low + high) / 2;
        if total_at(middle) <= budget {
            low = middle;
        } else {
            high = middle;
        }
    }
    let mut remainder = budget.saturating_sub(total_at(low));
    for item in items.iter_mut().filter(|item| item.fill) {
        item.size = (low as u16).clamp(item.min, item.max.max(item.min));
        // Only children at the common level participate in rounding; a child
        // held above it by its minimum must not consume another child's cell.
        if remainder > 0 && u64::from(item.size) == low && item.size < item.max.max(item.min) {
            item.size += 1;
            remainder -= 1;
        }
    }
}

/// Place already sized children inside a content viewport. Overflow starts at
/// the origin so all content remains reachable by positive scroll offsets.
pub fn place_flex_children(
    direction: FlexDirection,
    viewport: (u16, u16),
    sizes: &[(u16, u16)],
    gap: u16,
    align: Align,
    justify: Justify,
) -> Vec<Rect> {
    let (main, cross) = direction.axes(viewport.0, viewport.1);
    let occupied: u64 = sizes
        .iter()
        .map(|&(w, h)| u64::from(direction.axes(w, h).0))
        .sum::<u64>()
        + u64::from(gap) * sizes.len().saturating_sub(1) as u64;
    let free = u64::from(main).saturating_sub(occupied);
    let mut cursor = match justify {
        Justify::Center => free / 2,
        Justify::End => free,
        _ => 0,
    };
    let intervals = sizes.len().saturating_sub(1) as u64;
    sizes
        .iter()
        .enumerate()
        .map(|(index, &(w, h))| {
            let (child_main, child_cross) = direction.axes(w, h);
            let cross_free = cross.saturating_sub(child_cross);
            let cross_offset = match align {
                Align::Center => cross_free / 2,
                Align::End => cross_free,
                _ => 0,
            };
            let (x, y) = direction.axes(cursor.min(u64::from(u16::MAX)) as u16, cross_offset);
            cursor += u64::from(child_main) + u64::from(gap);
            if justify == Justify::SpaceBetween && intervals > 0 {
                cursor += free / intervals + u64::from((index as u64) < free % intervals);
            }
            Rect::new(x, y, w, h)
        })
        .collect()
}
