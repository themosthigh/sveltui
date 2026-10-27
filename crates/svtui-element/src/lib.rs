use crossterm::style::Color;
use svtui_layout::{Align, BorderStyle, Justify, Padding, Rect, Size};
use svtui_terminal::Buffer;

#[derive(Clone, Debug, Default)]
pub struct BoxElement {
    pub border: BorderStyle,
    pub border_color: Option<Color>,
    pub title: Option<String>,
    pub align_items: Align,
    pub justify_content: Justify,
    pub width: Size,
    pub height: Size,
    pub padding: Padding,
    pub children: Vec<VNode>,
}

impl BoxElement {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn border(mut self, border: BorderStyle) -> Self {
        self.border = border;
        self
    }

    pub fn border_color(mut self, color: Color) -> Self {
        self.border_color = Some(color);
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn align_items(mut self, align: Align) -> Self {
        self.align_items = align;
        self
    }

    pub fn justify_content(mut self, justify: Justify) -> Self {
        self.justify_content = justify;
        self
    }

    pub fn width(mut self, width: impl Into<Size>) -> Self {
        self.width = width.into();
        self
    }

    pub fn height(mut self, height: impl Into<Size>) -> Self {
        self.height = height.into();
        self
    }

    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = padding.into();
        self
    }

    pub fn child(mut self, child: impl IntoNode) -> Self {
        self.children.push(child.into_node());
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = impl IntoNode>) -> Self {
        for c in children {
            self.children.push(c.into_node());
        }
        self
    }
}

#[derive(Clone, Debug, Default)]
pub struct TextElement {
    pub content: String,
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

impl TextElement {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            ..Default::default()
        }
    }

    pub fn color(mut self, color: Color) -> Self {
        self.fg = Some(color);
        self
    }

    pub fn fg(mut self, color: Color) -> Self {
        self.fg = Some(color);
        self
    }

    pub fn bg(mut self, color: Color) -> Self {
        self.bg = Some(color);
        self
    }

    pub fn bold(mut self, bold: bool) -> Self {
        self.bold = bold;
        self
    }

    pub fn italic(mut self, italic: bool) -> Self {
        self.italic = italic;
        self
    }

    pub fn underline(mut self, underline: bool) -> Self {
        self.underline = underline;
        self
    }
}

#[derive(Clone, Debug)]
pub enum VNode {
    Box(BoxElement),
    Text(TextElement),
    Fragment(Vec<VNode>),
    Empty,
}

pub trait IntoNode {
    fn into_node(self) -> VNode;
}

impl IntoNode for VNode {
    fn into_node(self) -> VNode {
        self
    }
}

impl IntoNode for BoxElement {
    fn into_node(self) -> VNode {
        VNode::Box(self)
    }
}

impl IntoNode for TextElement {
    fn into_node(self) -> VNode {
        VNode::Text(self)
    }
}

impl IntoNode for &str {
    fn into_node(self) -> VNode {
        VNode::Text(TextElement::new(self))
    }
}

impl IntoNode for String {
    fn into_node(self) -> VNode {
        VNode::Text(TextElement::new(self))
    }
}

impl IntoNode for &String {
    fn into_node(self) -> VNode {
        VNode::Text(TextElement::new(self.as_str()))
    }
}

impl<T: IntoNode> IntoNode for Option<T> {
    fn into_node(self) -> VNode {
        match self {
            Some(v) => v.into_node(),
            None => VNode::Empty,
        }
    }
}

impl<T: IntoNode> IntoNode for Vec<T> {
    fn into_node(self) -> VNode {
        VNode::Fragment(self.into_iter().map(|item| item.into_node()).collect())
    }
}

impl IntoNode for () {
    fn into_node(self) -> VNode {
        VNode::Empty
    }
}

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

            let avail_w = max_w.saturating_sub(extra_w);
            let avail_h = max_h.saturating_sub(extra_h);

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

            let measured_h = match b.height {
                Size::Fixed(h) => h,
                Size::Percent(p) => ((max_h as f32 * p / 100.0).round() as u16).min(max_h),
                Size::Fill => max_h,
                Size::Auto => content_h.saturating_add(extra_h),
            };

            (measured_w.min(max_w), measured_h.min(max_h))
        }
    }
}

pub fn render_node(node: &VNode, rect: Rect, buffer: &mut Buffer) {
    if rect.width == 0 || rect.height == 0 {
        return;
    }

    match node {
        VNode::Empty => {}
        VNode::Text(text) => {
            let lines: Vec<&str> = text.content.lines().collect();
            for (i, line) in lines.iter().enumerate() {
                let y = rect.y.saturating_add(i as u16);
                if y >= rect.y.saturating_add(rect.height) {
                    break;
                }
                buffer.set_string(rect.x, y, line, text.fg, text.bg, text.bold, rect.width);
            }
        }
        VNode::Fragment(children) => {
            let mut curr_y = rect.y;
            for child in children {
                let (_, ch) = measure_node(child, rect.width, rect.height);
                let child_rect = Rect::new(rect.x, curr_y, rect.width, ch.min(rect.height));
                render_node(child, child_rect, buffer);
                curr_y = curr_y.saturating_add(ch);
                if curr_y >= rect.y + rect.height {
                    break;
                }
            }
        }
        VNode::Box(b) => {
            let border_w = if b.border != BorderStyle::None { 1 } else { 0 };
            let border_h = if b.border != BorderStyle::None { 1 } else { 0 };

            if let Some(chars) = b.border.chars() {
                buffer.draw_border(rect, chars, b.border_color);
            }

            if let Some(ref title) = b.title {
                buffer.draw_title(rect, title, b.border_color);
            }

            let inner = rect.inner(
                border_h + b.padding.top,
                border_w + b.padding.right,
                border_h + b.padding.bottom,
                border_w + b.padding.left,
            );

            if inner.width == 0 || inner.height == 0 || b.children.is_empty() {
                return;
            }

            let measured_children: Vec<(u16, u16)> = b
                .children
                .iter()
                .map(|c| measure_node(c, inner.width, inner.height))
                .collect();

            let total_children_h: u16 = measured_children.iter().map(|(_, h)| *h).sum();

            let start_y = match b.align_items {
                Align::Center => inner.y + inner.height.saturating_sub(total_children_h) / 2,
                Align::End => inner.y + inner.height.saturating_sub(total_children_h),
                _ => inner.y,
            };

            let mut curr_y = start_y;
            for (child, &(cw, ch)) in b.children.iter().zip(measured_children.iter()) {
                let start_x = match b.justify_content {
                    Justify::Center => inner.x + inner.width.saturating_sub(cw) / 2,
                    Justify::End => inner.x + inner.width.saturating_sub(cw),
                    _ => inner.x,
                };

                let child_rect = Rect::new(start_x, curr_y, cw.min(inner.width), ch.min(inner.height));
                render_node(child, child_rect, buffer);

                curr_y = curr_y.saturating_add(ch);
                if curr_y >= inner.y + inner.height {
                    break;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_measure_text() {
        let text = TextElement::new("Hello, world!").into_node();
        let (w, h) = measure_node(&text, 80, 24);
        assert_eq!(w, 13);
        assert_eq!(h, 1);
    }
}
