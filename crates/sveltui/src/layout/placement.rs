use crate::layout::geometry::Rect;

/// Target placement for a floating popover or overlay.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Placement {
    /// Placed in the exact center of the screen or viewport.
    #[default]
    Center,
    /// Centered above the anchor.
    Top,
    /// Above the anchor, aligned to the start (left) edge.
    TopStart,
    /// Above the anchor, aligned to the end (right) edge.
    TopEnd,
    /// Centered below the anchor.
    Bottom,
    /// Below the anchor, aligned to the start (left) edge.
    BottomStart,
    /// Below the anchor, aligned to the end (right) edge.
    BottomEnd,
    /// Centered to the left of the anchor.
    Left,
    /// To the left of the anchor, aligned to the top edge.
    LeftStart,
    /// To the left of the anchor, aligned to the bottom edge.
    LeftEnd,
    /// Centered to the right of the anchor.
    Right,
    /// To the right of the anchor, aligned to the top edge.
    RightStart,
    /// To the right of the anchor, aligned to the bottom edge.
    RightEnd,
    /// Anchored to explicit screen coordinates (e.g. mouse click for context menus).
    Anchor { x: u16, y: u16 },
}

/// Computes the layout [`Rect`] for a popover given its preferred placement,
/// desired content dimensions, optional anchor target, and screen bounds.
///
/// Features collision auto-flipping (e.g. Bottom -> Top when hitting bottom edge)
/// and viewport boundary clamping.
pub fn compute_popover_rect(
    placement: Placement,
    content_size: (u16, u16),
    anchor: Option<Rect>,
    screen: Rect,
) -> Rect {
    let width = content_size.0.min(screen.width);
    let height = content_size.1.min(screen.height);

    if screen.width == 0 || screen.height == 0 {
        return Rect::new(0, 0, 0, 0);
    }

    let max_x = screen.x + screen.width.saturating_sub(width);
    let max_y = screen.y + screen.height.saturating_sub(height);

    match placement {
        Placement::Center => {
            let x = screen.x + (screen.width.saturating_sub(width)) / 2;
            let y = screen.y + (screen.height.saturating_sub(height)) / 2;
            Rect::new(
                x.clamp(screen.x, max_x),
                y.clamp(screen.y, max_y),
                width,
                height,
            )
        }
        Placement::Anchor { x, y } => {
            let x = x.clamp(screen.x, max_x);
            let y = y.clamp(screen.y, max_y);
            Rect::new(x, y, width, height)
        }
        _ => {
            let anchor = match anchor {
                Some(a) => a,
                None => {
                    // Fall back to center if anchor is not available
                    let x = screen.x + (screen.width.saturating_sub(width)) / 2;
                    let y = screen.y + (screen.height.saturating_sub(height)) / 2;
                    return Rect::new(
                        x.clamp(screen.x, max_x),
                        y.clamp(screen.y, max_y),
                        width,
                        height,
                    );
                }
            };

            let (target_x, target_y) = match placement {
                Placement::Bottom | Placement::BottomStart | Placement::BottomEnd => {
                    let space_below =
                        (screen.y + screen.height).saturating_sub(anchor.y + anchor.height);
                    let space_above = anchor.y.saturating_sub(screen.y);

                    let y = if space_below >= height || space_below >= space_above {
                        anchor.y + anchor.height
                    } else {
                        // Auto-flip to top
                        anchor.y.saturating_sub(height)
                    };

                    let x = match placement {
                        Placement::BottomStart => anchor.x,
                        Placement::BottomEnd => (anchor.x + anchor.width).saturating_sub(width),
                        _ => anchor.x + (anchor.width.saturating_sub(width)) / 2,
                    };
                    (x, y)
                }
                Placement::Top | Placement::TopStart | Placement::TopEnd => {
                    let space_above = anchor.y.saturating_sub(screen.y);
                    let space_below =
                        (screen.y + screen.height).saturating_sub(anchor.y + anchor.height);

                    let y = if space_above >= height || space_above >= space_below {
                        anchor.y.saturating_sub(height)
                    } else {
                        // Auto-flip to bottom
                        anchor.y + anchor.height
                    };

                    let x = match placement {
                        Placement::TopStart => anchor.x,
                        Placement::TopEnd => (anchor.x + anchor.width).saturating_sub(width),
                        _ => anchor.x + (anchor.width.saturating_sub(width)) / 2,
                    };
                    (x, y)
                }
                Placement::Right | Placement::RightStart | Placement::RightEnd => {
                    let space_right =
                        (screen.x + screen.width).saturating_sub(anchor.x + anchor.width);
                    let space_left = anchor.x.saturating_sub(screen.x);

                    let x = if space_right >= width || space_right >= space_left {
                        anchor.x + anchor.width
                    } else {
                        // Auto-flip to left
                        anchor.x.saturating_sub(width)
                    };

                    let y = match placement {
                        Placement::RightStart => anchor.y,
                        Placement::RightEnd => (anchor.y + anchor.height).saturating_sub(height),
                        _ => anchor.y + (anchor.height.saturating_sub(height)) / 2,
                    };
                    (x, y)
                }
                Placement::Left | Placement::LeftStart | Placement::LeftEnd => {
                    let space_left = anchor.x.saturating_sub(screen.x);
                    let space_right =
                        (screen.x + screen.width).saturating_sub(anchor.x + anchor.width);

                    let x = if space_left >= width || space_left >= space_right {
                        anchor.x.saturating_sub(width)
                    } else {
                        // Auto-flip to right
                        anchor.x + anchor.width
                    };

                    let y = match placement {
                        Placement::LeftStart => anchor.y,
                        Placement::LeftEnd => (anchor.y + anchor.height).saturating_sub(height),
                        _ => anchor.y + (anchor.height.saturating_sub(height)) / 2,
                    };
                    (x, y)
                }
                _ => unreachable!(),
            };

            Rect::new(
                target_x.clamp(screen.x, max_x),
                target_y.clamp(screen.y, max_y),
                width,
                height,
            )
        }
    }
}
