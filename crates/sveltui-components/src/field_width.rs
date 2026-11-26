use sveltui::prelude::*;

/// How many text columns fit in a field once `chrome` columns of border,
/// padding and gutter are taken off. The width the layout gave the field in
/// the last frame wins, so percentage and auto widths follow their container.
/// Before the first paint it falls back to the `width` prop, then to
/// `fallback` when that isn't a fixed length either.
pub(crate) fn text_columns(id: FocusId, width: Dimension, chrome: u16, fallback: usize) -> usize {
    let outer = use_layout(id).map(|rect| rect.width).or(match width {
        Dimension::Length(w) => Some(w.round() as u16),
        _ => None,
    });
    match outer {
        Some(w) => w.saturating_sub(chrome).max(1) as usize,
        None => fallback,
    }
}
