use crate::node::VNode;
use svtui_layout::{BorderStyle, Overflow, Size, clamp_dimension};

pub fn measure_node(node: &VNode, max_w: u16, max_h: u16) -> (u16, u16) {
    match node {
        VNode::Empty => (0, 0),
        VNode::Text(text) => {
            let lines: Vec<&str> = text.content.lines().collect();
            let height = (lines.len().max(1) as u16).min(max_h);
            let width = lines
                .iter()
                .map(|line| line.chars().count() as u16)
                .max()
                .unwrap_or(0)
                .min(max_w);
            (width, height)
        }
        VNode::Fragment(children) => {
            let mut total_w: u16 = 0;
            let mut total_h: u16 = 0;
            for child in children {
                let (cw, ch) = measure_node(child, max_w, max_h);
                total_w = total_w.max(cw);
                total_h = total_h.saturating_add(ch);
            }
            (total_w.min(max_w), total_h.min(max_h))
        }
        VNode::Box(b) => {
            let border_w = if b.border != BorderStyle::None { 2 } else { 0 };
            let border_h = if b.border != BorderStyle::None { 2 } else { 0 };

            let pad_w = b.padding.left + b.padding.right;
            let pad_h = b.padding.top + b.padding.bottom;

            let extra_w = border_w + pad_w;
            let extra_h = border_h + pad_h;

            let avail_w = if b.overflow_x == Overflow::Scroll || b.overflow_x == Overflow::Auto {
                u16::MAX / 2
            } else {
                max_w.saturating_sub(extra_w)
            };

            let avail_h = if b.overflow_y == Overflow::Scroll || b.overflow_y == Overflow::Auto {
                u16::MAX / 2
            } else {
                max_h.saturating_sub(extra_h)
            };

            let mut content_w: u16 = 0;
            let mut content_h: u16 = 0;
            for child in &b.children {
                let (cw, ch) = measure_node(child, avail_w, avail_h);
                content_w = content_w.max(cw);
                content_h = content_h.saturating_add(ch);
            }

            if let Some(ref title) = b.title {
                let title_w = title.chars().count() as u16 + 4;
                content_w = content_w.max(title_w);
            }

            let measured_w = match b.width {
                Size::Fixed(w) => w,
                Size::Percent(p) => ((max_w as f32 * p / 100.0).round() as u16).min(max_w),
                Size::Fill => max_w,
                Size::Auto => content_w.saturating_add(extra_w),
            };
            let clamped_w = clamp_dimension(measured_w, b.min_width, b.max_width, max_w);

            let measured_h = match b.height {
                Size::Fixed(h) => h,
                Size::Percent(p) => ((max_h as f32 * p / 100.0).round() as u16).min(max_h),
                Size::Fill => max_h,
                Size::Auto => content_h.saturating_add(extra_h),
            };
            let clamped_h = clamp_dimension(measured_h, b.min_height, b.max_height, max_h);

            (clamped_w.min(max_w), clamped_h.min(max_h))
        }
    }
}
