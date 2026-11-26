use crate::element::events::{HitMap, HitTarget, LayoutTarget, ScrollRegion};
use crate::element::focus::next_render_focus_id;
use crate::element::node::{OverlayElement, VNode};
use crate::element::scroll::get_scroll;
use crate::element::taffy_bridge::ComputedLayoutNode;
use crate::element::taffy_bridge::layout;
use crate::layout::{BorderStyle, Rect, compute_popover_rect};
use crate::terminal::{Buffer, ImagePlacement};
use crossterm::style::Color;
use std::rc::Rc;

fn scrollbar_thumb(track: u16, viewport: u16, content: u16, offset: u16) -> std::ops::Range<u16> {
    if track == 0 {
        return 0..0;
    }
    let length = (u32::from(track) * u32::from(viewport) / u32::from(content.max(1)))
        .clamp(1, u32::from(track)) as u16;
    let max_scroll = content.saturating_sub(viewport);
    let start = if max_scroll == 0 {
        0
    } else {
        (u32::from(offset.min(max_scroll)) * u32::from(track - length) / u32::from(max_scroll))
            as u16
    };
    start..start + length
}

/// One scrollbar's track: where it starts on screen along its axis, how
/// long it is, and the sizes that set its thumb.
#[derive(Clone, Copy)]
struct Bar {
    start: i32,
    track: u16,
    viewport: u16,
    content: u16,
}

impl Bar {
    fn max(&self) -> u16 {
        self.content.saturating_sub(self.viewport)
    }

    /// The pointer's place along the track, held to its ends.
    fn along(&self, at: u16) -> u16 {
        (i32::from(at) - self.start).clamp(0, i32::from(self.track) - 1) as u16
    }

    /// The offset that puts the thumb's top (or left) at `start`. It rounds
    /// up, so the thumb drawn for it lands back on `start`.
    fn offset_for(&self, start: u16) -> u16 {
        let length = scrollbar_thumb(self.track, self.viewport, self.content, 0).len() as u16;
        let room = self.track.saturating_sub(length);
        if room == 0 {
            return 0;
        }
        let max = u32::from(self.max());
        (u32::from(start.min(room)) * max).div_ceil(u32::from(room)) as u16
    }
}

/// A scroll view's bars, for presses on them. A press on the track pages
/// one viewport towards it; a press on the thumb grabs it, and dragging
/// then scrolls so the thumb stays under the pointer.
#[derive(Clone, Copy)]
struct ScrollBars {
    id: crate::element::FocusId,
    vertical: Bar,
    horizontal: Bar,
    /// The vertical bar's column, which tells a press on it from one on the
    /// horizontal bar.
    column: i32,
}

impl ScrollBars {
    fn pointer(&self, event: crate::element::PointerEvent, wheel: &Rc<dyn Fn(i16, i16)>) {
        use crate::element::PointerPhase;
        use crate::element::scroll::{ThumbDrag, get_scroll, set_thumb_drag, thumb_drag};

        // The offset is read fresh each time: several drags can arrive
        // before the next frame redraws the bar.
        let offset = |vertical: bool| {
            let (x, y) = get_scroll(self.id);
            if vertical { y } else { x }
        };
        let scroll_to = |vertical: bool, to: u16| {
            let by = (i32::from(to) - i32::from(offset(vertical)))
                .clamp(i32::from(i16::MIN), i32::from(i16::MAX)) as i16;
            match (by, vertical) {
                (0, _) => {}
                (by, true) => wheel(0, by),
                (by, false) => wheel(by, 0),
            }
        };
        let bar = |vertical: bool| {
            if vertical {
                (self.vertical, event.y)
            } else {
                (self.horizontal, event.x)
            }
        };

        match event.phase {
            PointerPhase::Down => {
                let vertical = self.vertical.track > 0 && i32::from(event.x) == self.column;
                let (bar, at) = bar(vertical);
                let at = bar.along(at);
                let current = offset(vertical);
                let thumb = scrollbar_thumb(bar.track, bar.viewport, bar.content, current);
                if thumb.contains(&at) {
                    set_thumb_drag(Some(ThumbDrag {
                        id: self.id,
                        vertical,
                        grab: at - thumb.start,
                    }));
                } else {
                    set_thumb_drag(None);
                    let to = if at < thumb.start {
                        current.saturating_sub(bar.viewport)
                    } else {
                        current.saturating_add(bar.viewport).min(bar.max())
                    };
                    scroll_to(vertical, to);
                }
            }
            PointerPhase::Drag => {
                if let Some(drag) = thumb_drag().filter(|d| d.id == self.id) {
                    let (bar, at) = bar(drag.vertical);
                    let start = bar.along(at).saturating_sub(drag.grab);
                    scroll_to(drag.vertical, bar.offset_for(start));
                }
            }
            PointerPhase::Up => {
                if thumb_drag().is_some_and(|d| d.id == self.id) {
                    set_thumb_drag(None);
                }
            }
        }
    }
}

struct PendingOverlay {
    overlay: OverlayElement,
    anchor_rect: Option<Rect>,
    foreground: Option<Color>,
}

struct RenderContext<'a> {
    buffer: &'a mut Buffer,
    hit_map: &'a mut HitMap,
    layer: Option<crate::element::FocusId>,
    scrolls: Vec<ScrollRegion>,
    overlays: &'a mut Vec<PendingOverlay>,
    active_fg: Option<Color>,
    active_bg: Option<Color>,
}

// Intersect signed layout coordinates before converting to terminal cells.
// Negative scroll offsets must reduce the visible size, not move a full-sized
// hit target to the origin.
fn visible_rect(x: i32, y: i32, width: u16, height: u16, clip: Rect) -> Rect {
    let left = x.max(i32::from(clip.x));
    let top = y.max(i32::from(clip.y));
    let right = (x + i32::from(width)).min(i32::from(clip.x) + i32::from(clip.width));
    let bottom = (y + i32::from(height)).min(i32::from(clip.y) + i32::from(clip.height));
    Rect::new(
        left.max(0).min(i32::from(u16::MAX)) as u16,
        top.max(0).min(i32::from(u16::MAX)) as u16,
        right.saturating_sub(left).max(0) as u16,
        bottom.saturating_sub(top).max(0) as u16,
    )
}

fn register_target(ctx: &mut RenderContext<'_>, x: i32, y: i32, mut target: HitTarget) {
    target.rect = visible_rect(
        x,
        y,
        target.rect.width,
        target.rect.height,
        ctx.buffer.active_clip(),
    );
    // Preserve element-local click coordinates after clipping its hit area.
    if let Some(handler) = target.on_click_at.take() {
        let dx = (i32::from(target.rect.x) - x).max(0) as u16;
        let dy = (i32::from(target.rect.y) - y).max(0) as u16;
        target.on_click_at = Some(Rc::new(move |cx, cy| {
            handler(cx.saturating_add(dx), cy.saturating_add(dy));
        }));
    }
    target.layer = target.layer.or(ctx.layer);
    ctx.hit_map.register(target);
}

pub fn render_node(node: &VNode, rect: Rect, buffer: &mut Buffer) {
    let mut hit_map = HitMap::new();
    render_node_interactive(node, rect, buffer, &mut hit_map);
}

pub fn render_node_interactive(
    node: &VNode,
    rect: Rect,
    buffer: &mut Buffer,
    hit_map: &mut HitMap,
) {
    if node.has_dynamic() {
        return render_node_interactive(&node.clone().resolve(), rect, buffer, hit_map);
    }
    hit_map.collect_interactions(node);
    // One layout for the whole tree. Its root size is what the node
    // measures to, and its children are passed down so nothing below lays
    // out again.
    let root_layout = layout(node, rect.width, rect.height);
    let (width, height) = (root_layout.rect.width, root_layout.rect.height);
    let mut overlays = Vec::new();
    {
        let mut ctx = RenderContext {
            buffer,
            hit_map,
            layer: None,
            scrolls: Vec::new(),
            overlays: &mut overlays,
            active_fg: None,
            active_bg: None,
        };
        render_node_offset_internal(
            node,
            Some(&root_layout),
            rect.x as i32,
            rect.y as i32,
            width,
            height,
            &mut ctx,
        );
    }

    while !overlays.is_empty() {
        let current_overlays = std::mem::take(&mut overlays);
        for pending in current_overlays {
            render_single_overlay(pending, rect, buffer, hit_map, &mut overlays);
        }
    }
    hit_map.finish_interactions();
}

pub fn render_node_offset(
    node: &VNode,
    x: i32,
    y: i32,
    width: u16,
    height: u16,
    buffer: &mut Buffer,
    hit_map: &mut HitMap,
) {
    hit_map.collect_interactions(node);
    let root_layout = layout(node, width, height);
    let (width, height) = (root_layout.rect.width, root_layout.rect.height);
    let mut overlays = Vec::new();
    {
        let mut ctx = RenderContext {
            buffer,
            hit_map,
            layer: None,
            scrolls: Vec::new(),
            overlays: &mut overlays,
            active_fg: None,
            active_bg: None,
        };
        render_node_offset_internal(node, Some(&root_layout), x, y, width, height, &mut ctx);
    }

    let screen_rect = Rect::new(0, 0, buffer.width, buffer.height);
    while !overlays.is_empty() {
        let current_overlays = std::mem::take(&mut overlays);
        for pending in current_overlays {
            render_single_overlay(pending, screen_rect, buffer, hit_map, &mut overlays);
        }
    }
    hit_map.finish_interactions();
}

fn render_single_overlay(
    pending: PendingOverlay,
    screen: Rect,
    buffer: &mut Buffer,
    hit_map: &mut HitMap,
    overlays: &mut Vec<PendingOverlay>,
) {
    let PendingOverlay {
        overlay,
        anchor_rect,
        foreground,
    } = pending;
    if !overlay.open {
        return;
    }

    let anchor_rect = overlay
        .anchor
        .and_then(|id| hit_map.layouts.get(&id))
        .map(|layout| {
            Rect::new(
                layout.x.max(0) as u16,
                layout.y.max(0) as u16,
                layout.width,
                layout.height,
            )
        })
        .or(anchor_rect);
    if overlay.backdrop {
        buffer.occlude(screen);
        for cell in &mut buffer.cells {
            cell.fg = Some(Color::DarkGrey);
            cell.bold = false;
        }
    }

    // A press outside the layer either dismisses it or runs its handler.
    // When it does neither, there's nothing to catch, so presses go through
    // to what's underneath (a modal layer still blocks them, since the
    // manager refuses targets outside it).
    let dismisses = overlay.layer.dismiss_on_outside_press;
    if dismisses || overlay.on_outside_press.is_some() {
        hit_map.register(HitTarget {
            rect: screen,
            layer: Some(overlay.id),
            dismiss: dismisses.then_some(overlay.id),
            on_click: if dismisses {
                None
            } else {
                overlay.on_outside_press.clone()
            },
            on_click_at: None,
            focus_id: None,
            on_key: None,
            on_scroll: None,
            on_pointer_enter: None,
            on_hover: None,
            on_pointer: None,
            pointer: None,
        });
    }

    if let Some(content) = overlay.content {
        let mut content_layout = layout(&content, screen.width, screen.height);
        let (cw, ch) = (content_layout.rect.width, content_layout.rect.height);
        let popover_rect = compute_popover_rect(overlay.placement, (cw, ch), anchor_rect, screen);
        // The popover can come out smaller than the content asked for, near
        // a screen edge. Only then do we lay the content out again, at the
        // size it actually got.
        if (popover_rect.width, popover_rect.height) != (cw, ch) {
            content_layout = layout(&content, popover_rect.width, popover_rect.height);
        }

        hit_map.register(HitTarget {
            rect: popover_rect,
            layer: Some(overlay.id),
            dismiss: None,
            on_click: None,
            on_click_at: None,
            focus_id: None,
            on_key: None,
            on_scroll: None,
            on_pointer_enter: None,
            on_hover: None,
            on_pointer: None,
            pointer: None,
        });

        buffer.occlude(popover_rect);
        for cy in popover_rect.y..(popover_rect.y + popover_rect.height) {
            for cx in popover_rect.x..(popover_rect.x + popover_rect.width) {
                buffer.set_char(cx, cy, ' ', None, None, false);
            }
        }

        let mut ctx = RenderContext {
            buffer,
            hit_map,
            layer: Some(overlay.id),
            scrolls: Vec::new(),
            overlays,
            active_fg: foreground,
            active_bg: None,
        };
        render_node_offset_internal(
            &content,
            Some(&content_layout),
            popover_rect.x as i32,
            popover_rect.y as i32,
            popover_rect.width,
            popover_rect.height,
            &mut ctx,
        );
    }
}

fn render_node_offset_internal(
    node: &VNode,
    layout: Option<&ComputedLayoutNode>,
    x: i32,
    y: i32,
    width: u16,
    height: u16,
    ctx: &mut RenderContext,
) {
    if let VNode::View(v) = node {
        // Remember where every identified view landed, so overlays can
        // anchor to it and focus can scroll it into view.
        let ids = v
            .pressables
            .iter()
            .filter_map(|p| p.focus_id)
            .chain(v.scroll.as_ref().and_then(|s| s.id));
        // The first view to claim an id keeps it. A parent comes before its
        // children, so parts that share their parent's id (to focus it when
        // clicked) don't take over its place.
        for id in ids {
            ctx.hit_map
                .layouts
                .entry(id)
                .or_insert_with(|| LayoutTarget {
                    x,
                    y,
                    width,
                    height,
                    scrolls: ctx.scrolls.clone(),
                });
        }
    }
    if let VNode::Overlay(overlay) = node {
        let anchor_rect = if let Some(ref trigger) = overlay.trigger {
            let anchor = Rect::new(x.max(0) as u16, y.max(0) as u16, width, height);
            render_node_offset_internal(trigger, layout, x, y, width, height, ctx);
            Some(anchor)
        } else {
            None
        };

        if overlay.open {
            ctx.overlays.push(PendingOverlay {
                overlay: overlay.clone(),
                anchor_rect,
                foreground: ctx.active_fg,
            });
        }
        return;
    }

    if width == 0 || height == 0 {
        // Out-of-flow overlays still render when their ancestor has no
        // intrinsic size (for example, a root containing only a dialog).
        let previous_fg = ctx.active_fg;
        if let VNode::View(view) = node
            && let Some(foreground) = view
                .visual
                .as_ref()
                .and_then(|visual| visual.foreground_color)
        {
            ctx.active_fg = Some(foreground);
        }
        let children = match node {
            VNode::View(v) => Some(&v.children),
            VNode::Fragment(children) => Some(children),
            _ => None,
        };
        if let Some(children) = children {
            for child in children {
                render_node_offset_internal(child, None, x, y, 0, 0, ctx);
            }
        }
        ctx.active_fg = previous_fg;
        return;
    }
    if (x >= ctx.buffer.width as i32
        || y >= ctx.buffer.height as i32
        || x + width as i32 <= 0
        || y + height as i32 <= 0)
        && !matches!(node, VNode::View(_) | VNode::Fragment(_))
    {
        return;
    }

    match node {
        // Slots are resolved on the way in (`render_node_interactive`), so
        // there is nothing to paint for one that got this far.
        VNode::Empty | VNode::Overlay(_) | VNode::Dynamic(_) => {}
        VNode::Text(text) => {
            let lines = crate::element::text::lines(text, Some(width));
            for (i, line) in lines.iter().enumerate() {
                let curr_y = y + i as i32;
                if curr_y < 0 || curr_y >= (y + height as i32) || curr_y >= ctx.buffer.height as i32
                {
                    continue;
                }
                let clip = ctx.buffer.active_clip();
                if curr_y < clip.y as i32 || curr_y >= (clip.y as i32 + clip.height as i32) {
                    continue;
                }
                for (char_idx, ch) in line
                    .chars()
                    .chain(text.fill.into_iter().cycle())
                    .take(usize::from(width))
                    .enumerate()
                {
                    let curr_x = x + char_idx as i32;
                    if curr_x >= x + width as i32 || curr_x >= ctx.buffer.width as i32 {
                        break;
                    }
                    if curr_x >= 0 {
                        let fg = text.fg.or(ctx.active_fg);
                        let bg = text.bg.or(ctx.active_bg).or_else(|| {
                            ctx.buffer
                                .get(curr_x as u16, curr_y as u16)
                                .and_then(|c| c.bg)
                        });
                        ctx.buffer
                            .set_char(curr_x as u16, curr_y as u16, ch, fg, bg, text.bold);
                    }
                }
            }
        }
        VNode::Fragment(children) => {
            let mut curr_y = y;
            for (idx, child) in children.iter().enumerate() {
                let Some(child_layout) = layout.and_then(|l| l.children.get(idx)) else {
                    continue;
                };
                let (cw, ch) = (child_layout.rect.width, child_layout.rect.height);
                render_node_offset_internal(child, Some(child_layout), x, curr_y, cw, ch, ctx);
                curr_y = curr_y.saturating_add(ch as i32);
            }
        }
        VNode::View(v) => {
            for p in v.pressables.iter().filter(|p| p.is_hit_target()) {
                let target = HitTarget {
                    rect: Rect::new(x.max(0) as u16, y.max(0) as u16, width, height),
                    layer: ctx.layer,
                    dismiss: None,
                    on_click: p.on_click.clone(),
                    on_click_at: p.on_click_at.clone(),
                    focus_id: p.focus_id,
                    on_key: p.on_key.clone(),
                    on_scroll: None,
                    on_pointer_enter: p.on_pointer_enter.clone(),
                    on_hover: p.on_hover.clone(),
                    on_pointer: p.on_pointer.clone(),
                    pointer: p.pointer,
                };
                register_target(ctx, x, y, target);
            }
            let prev_fg = ctx.active_fg;
            let prev_bg = ctx.active_bg;
            if let Some(vis) = &v.visual {
                if let Some(fg) = vis.foreground_color {
                    ctx.active_fg = Some(fg);
                }
                if let Some(bg) = vis.background_color {
                    ctx.active_bg = Some(bg);
                }
            }

            let (bg, border, border_color, title) = if let Some(vis) = &v.visual {
                (
                    vis.background_color,
                    vis.border,
                    vis.border_color,
                    vis.title.as_deref(),
                )
            } else {
                (None, BorderStyle::None, None, None)
            };

            // 1. Fill background if specified
            if let Some(bg) = bg {
                for curr_y in y.max(0)..(y + height as i32).min(ctx.buffer.height as i32) {
                    for curr_x in x.max(0)..(x + width as i32).min(ctx.buffer.width as i32) {
                        if curr_x >= 0 && curr_y >= 0 {
                            ctx.buffer.set_char(
                                curr_x as u16,
                                curr_y as u16,
                                ' ',
                                None,
                                Some(bg),
                                false,
                            );
                        }
                    }
                }
            }

            // An image goes over the cells, clipped to what shows.
            if let Some(slot) = v.visual.as_ref().and_then(|vis| vis.image.as_ref()) {
                let visible = visible_rect(x, y, width, height, ctx.buffer.active_clip());
                if visible.width > 0 && visible.height > 0 {
                    ctx.buffer.place_image(ImagePlacement {
                        at: (visible.x, visible.y),
                        size: (width, height),
                        visible: Rect::new(
                            (i32::from(visible.x) - x).max(0) as u16,
                            (i32::from(visible.y) - y).max(0) as u16,
                            visible.width,
                            visible.height,
                        ),
                        paint: slot.0.clone(),
                    });
                }
            }

            // 2. Draw border

            if let Some(chars) = border.chars().filter(|_| width >= 2 && height >= 2) {
                let right = x + width as i32 - 1;
                let bottom = y + height as i32 - 1;

                let draw_ch = |cx: i32, cy: i32, ch: char, buf: &mut Buffer| {
                    if cx >= 0 && cy >= 0 && cx < i32::from(buf.width) && cy < i32::from(buf.height)
                    {
                        buf.set_char(cx as u16, cy as u16, ch, border_color, bg, false);
                    }
                };

                draw_ch(x, y, chars.top_left, ctx.buffer);
                draw_ch(right, y, chars.top_right, ctx.buffer);
                draw_ch(x, bottom, chars.bottom_left, ctx.buffer);
                draw_ch(right, bottom, chars.bottom_right, ctx.buffer);

                for cx in (x + 1).max(0)..right.min(i32::from(ctx.buffer.width)) {
                    draw_ch(cx, y, chars.horizontal, ctx.buffer);
                    draw_ch(cx, bottom, chars.horizontal, ctx.buffer);
                }

                for cy in (y + 1).max(0)..bottom.min(i32::from(ctx.buffer.height)) {
                    draw_ch(x, cy, chars.vertical, ctx.buffer);
                    draw_ch(right, cy, chars.vertical, ctx.buffer);
                }
            }

            // 3. Draw title
            if let Some(t) = title.filter(|t| width > 4 && !t.is_empty()) {
                let title_text = format!(" {} ", t);
                let max_len = (width.saturating_sub(4)) as usize;
                let displayed_title: String = title_text.chars().take(max_len).collect();
                let start_x = x + 2;
                for (idx, ch) in displayed_title.chars().enumerate() {
                    let cx = start_x + idx as i32;
                    if cx >= 0
                        && y >= 0
                        && cx < i32::from(ctx.buffer.width)
                        && y < i32::from(ctx.buffer.height)
                    {
                        ctx.buffer
                            .set_char(cx as u16, y as u16, ch, border_color, None, true);
                    }
                }
            }

            // Where the children's area starts: border plus padding, as
            // layout resolved them.
            let inset = layout.map(|l| l.content_inset).unwrap_or_default();
            let inner_x = x + inset.left as i32;
            let inner_y = y + inset.top as i32;

            if let Some(vis) = &v.visual {
                if let Some((cx_off, cy_off)) = vis.cursor_offset {
                    let screen_cx = inner_x + cx_off as i32;
                    let screen_cy = inner_y + cy_off as i32;
                    let clip = ctx.buffer.active_clip();
                    let in_clip = screen_cx >= clip.x as i32
                        && screen_cx < (clip.x + clip.width) as i32
                        && screen_cy >= clip.y as i32
                        && screen_cy < (clip.y + clip.height) as i32;
                    if in_clip
                        && screen_cx >= 0
                        && screen_cy >= 0
                        && screen_cx < ctx.buffer.width as i32
                        && screen_cy < ctx.buffer.height as i32
                    {
                        ctx.buffer.set_cursor(screen_cx as u16, screen_cy as u16);
                        if let Some(style) = vis.cursor_style {
                            ctx.buffer.set_cursor_style(style);
                        }
                    }
                }
            }

            // Every caller passes the node's layout down from the one pass
            // at the root, so there's nothing to lay out here.
            if let Some(computed) =
                layout.filter(|_| width > 0 && height > 0 && !v.children.is_empty())
            {
                let overflow = v.style.overflow;
                let is_scroll_x = overflow.x == taffy::style::Overflow::Scroll;
                let is_scroll_y = overflow.y == taffy::style::Overflow::Scroll;
                let is_scrollable = is_scroll_x || is_scroll_y;
                // A fixed-size overlay may be allocated less than its requested
                // size. Keep its content clip and scroll range inside that allocation.
                let full_w = computed
                    .viewport_size
                    .0
                    .saturating_sub(computed.rect.width.saturating_sub(width));
                let full_h = computed
                    .viewport_size
                    .1
                    .saturating_sub(computed.rect.height.saturating_sub(height));
                let sb_vis = v
                    .scroll
                    .as_ref()
                    .map_or(crate::layout::ScrollbarVisibility::Auto, |s| {
                        s.scrollbar_visibility
                    });
                // Layout leaves no room for the bars of a view that flex
                // sizes, so they would cover its last column and row. Take
                // their cells off the viewport here; one bar can make the
                // other one necessary, so each is checked against the room
                // the other leaves.
                let (viewport_w, viewport_h) = if computed.scrollbar_reserved == (true, true) {
                    (full_w, full_h)
                } else {
                    let wants = |scrolls: bool, content: u16, room: u16| {
                        scrolls
                            && match sb_vis {
                                crate::layout::ScrollbarVisibility::Always => true,
                                crate::layout::ScrollbarVisibility::Never => false,
                                crate::layout::ScrollbarVisibility::Auto => content > room,
                            }
                    };
                    let (content_w, content_h) = computed.content_size;
                    let horizontal = wants(is_scroll_x, content_w, full_w);
                    let vertical = wants(
                        is_scroll_y,
                        content_h,
                        full_h.saturating_sub(u16::from(horizontal)),
                    );
                    let horizontal = wants(
                        is_scroll_x,
                        content_w,
                        full_w.saturating_sub(u16::from(vertical)),
                    );
                    (
                        full_w
                            .saturating_sub(u16::from(vertical && !computed.scrollbar_reserved.0)),
                        full_h.saturating_sub(u16::from(
                            horizontal && !computed.scrollbar_reserved.1,
                        )),
                    )
                };

                let max_scroll_x = if is_scroll_x {
                    computed.content_size.0.saturating_sub(viewport_w)
                } else {
                    0
                };
                let max_scroll_y = if is_scroll_y {
                    computed.content_size.1.saturating_sub(viewport_h)
                } else {
                    0
                };

                let (scroll_x, scroll_y, scroll_fid) = if is_scrollable {
                    let fid = v
                        .scroll
                        .as_ref()
                        .and_then(|s| s.id)
                        .unwrap_or_else(next_render_focus_id);
                    let (mut sx, mut sy) = get_scroll(fid);
                    if let Some(s) = &v.scroll {
                        if s.scroll_x > 0 {
                            sx = s.scroll_x;
                        }
                        if s.scroll_y > 0 {
                            sy = s.scroll_y;
                        }
                    }
                    sx = sx.min(max_scroll_x);
                    sy = sy.min(max_scroll_y);
                    crate::element::scroll::set_scroll(fid, (sx, sy));
                    (sx, sy, Some(fid))
                } else if let Some(s) = &v.scroll {
                    (s.scroll_x, s.scroll_y, None)
                } else {
                    (0, 0, None)
                };

                let mut wheel: Option<Rc<dyn Fn(i16, i16)>> = None;
                if is_scrollable && let Some(id) = scroll_fid {
                    crate::element::scroll::set_viewport(id, (viewport_w, viewport_h));
                    let custom_on_scroll = v.scroll.as_ref().and_then(|s| s.on_scroll.clone());
                    let on_scroll: Rc<dyn Fn(i16, i16)> = Rc::new(move |mut dx, dy| {
                        if is_scroll_x && !is_scroll_y && dx == 0 && dy != 0 {
                            dx = dy * 2;
                        }
                        let (cx, cy) = get_scroll(id);
                        let nx = (cx as i32 + dx as i32).max(0).min(max_scroll_x as i32) as u16;
                        let ny = (cy as i32 + dy as i32).max(0).min(max_scroll_y as i32) as u16;
                        crate::element::scroll::set_scroll(id, (nx, ny));
                        if let Some(custom) = &custom_on_scroll {
                            custom(dx, dy);
                        }
                    });
                    wheel = Some(on_scroll.clone());

                    let clipped_rect = Rect::new(x.max(0) as u16, y.max(0) as u16, width, height)
                        .intersect(&ctx.buffer.active_clip())
                        .unwrap_or(Rect::new(0, 0, 0, 0));

                    let merged = if let Some(last) = ctx.hit_map.targets.last_mut() {
                        if last.focus_id == Some(id) && last.rect == clipped_rect {
                            last.on_scroll = Some(on_scroll.clone());
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    };

                    if !merged {
                        let target = HitTarget {
                            rect: Rect::new(x.max(0) as u16, y.max(0) as u16, width, height),
                            layer: ctx.layer,
                            dismiss: None,
                            on_click: None,
                            on_click_at: None,
                            focus_id: Some(id),
                            on_key: None,
                            on_scroll: Some(on_scroll),
                            on_pointer_enter: None,
                            on_hover: None,
                            on_pointer: None,
                            pointer: None,
                        };
                        register_target(ctx, x, y, target);
                    }

                    ctx.scrolls.push(ScrollRegion {
                        id,
                        x: inner_x,
                        y: inner_y,
                        width: viewport_w,
                        height: viewport_h,
                        max_x: (v.scroll.as_ref().map_or(0, |s| s.scroll_x) == 0)
                            .then_some(max_scroll_x),
                        max_y: (v.scroll.as_ref().map_or(0, |s| s.scroll_y) == 0)
                            .then_some(max_scroll_y),
                    });
                }

                // Rules go in before the content clip so they can reach the
                // border through any padding. They never leave the area inside
                // the border, so even a scrolling grid's rules stay in place.
                crate::element::rules::draw_grid_rules(
                    ctx.buffer,
                    v,
                    computed,
                    (x, y),
                    (width, height),
                    (scroll_x, scroll_y),
                );

                let is_clip_x = is_scroll_x || overflow.x == taffy::style::Overflow::Hidden;
                let is_clip_y = is_scroll_y || overflow.y == taffy::style::Overflow::Hidden;
                let mut clipped = false;
                if is_clip_x || is_clip_y {
                    let clip_x = inner_x.max(0) as u16;
                    let clip_y = inner_y.max(0) as u16;
                    let clip_w = if is_clip_x {
                        viewport_w
                    } else {
                        ctx.buffer.width.saturating_sub(clip_x)
                    };
                    let clip_h = if is_clip_y {
                        viewport_h
                    } else {
                        ctx.buffer.height.saturating_sub(clip_y)
                    };
                    let clip_rect = Rect::new(clip_x, clip_y, clip_w, clip_h);
                    ctx.buffer.push_clip(clip_rect);
                    clipped = true;
                }

                for (idx, child) in v.children.iter().enumerate() {
                    if let Some(child_layout) = computed.children.get(idx) {
                        render_node_offset_internal(
                            child,
                            Some(child_layout),
                            x + child_layout.rect.x as i32 - scroll_x as i32,
                            y + child_layout.rect.y as i32 - scroll_y as i32,
                            child_layout.rect.width,
                            child_layout.rect.height,
                            ctx,
                        );
                    }
                }

                if clipped {
                    ctx.buffer.pop_clip();
                }

                if is_scrollable {
                    ctx.scrolls.pop();
                }

                let default_sb = crate::layout::ScrollbarStyle::default();
                let sb_style = v
                    .scroll
                    .as_ref()
                    .map(|s| &s.scrollbar_style)
                    .unwrap_or(&default_sb);
                let v_track_ch = sb_style.chars.vertical_track;
                let v_thumb_ch = sb_style.chars.vertical_thumb;
                let v_track_fg = sb_style.track_color.or(border_color);
                let v_thumb_fg = sb_style.thumb_color.or(border_color);

                let show_v_sb = is_scroll_y
                    && match sb_vis {
                        crate::layout::ScrollbarVisibility::Always => true,
                        crate::layout::ScrollbarVisibility::Never => false,
                        crate::layout::ScrollbarVisibility::Auto => max_scroll_y > 0,
                    };
                let show_h_sb = is_scroll_x
                    && match sb_vis {
                        crate::layout::ScrollbarVisibility::Always => true,
                        crate::layout::ScrollbarVisibility::Never => false,
                        crate::layout::ScrollbarVisibility::Auto => max_scroll_x > 0,
                    };

                // The bars sit just inside the border, outside any padding,
                // and run its whole length; padding only insets the content.
                let edge = computed.border_inset;
                let bar_x = x + i32::from(edge.left);
                let bar_y = y + i32::from(edge.top);
                let bar_w = width.saturating_sub(edge.left + edge.right);
                let bar_h = height.saturating_sub(edge.top + edge.bottom);
                let v_track = if show_v_sb && bar_w > 0 {
                    bar_h.saturating_sub(u16::from(show_h_sb))
                } else {
                    0
                };
                let h_track = if show_h_sb && bar_h > 0 {
                    bar_w.saturating_sub(u16::from(show_v_sb))
                } else {
                    0
                };
                let v_column = bar_x + i32::from(bar_w) - 1;
                let h_row = bar_y + i32::from(bar_h) - 1;

                if v_track > 0 {
                    let thumb =
                        scrollbar_thumb(v_track, viewport_h, computed.content_size.1, scroll_y);
                    for row in 0..v_track {
                        let sb_y = bar_y + i32::from(row);
                        if v_column >= 0 && sb_y >= 0 {
                            let active = thumb.contains(&row);
                            ctx.buffer.set_char(
                                v_column as u16,
                                sb_y as u16,
                                if active { v_thumb_ch } else { v_track_ch },
                                if active { v_thumb_fg } else { v_track_fg },
                                None,
                                active,
                            );
                        }
                    }
                }

                if h_track > 0 {
                    let h_track_ch = sb_style.chars.horizontal_track;
                    let h_thumb_ch = sb_style.chars.horizontal_thumb;
                    let h_track_fg = sb_style.track_color.or(border_color);
                    let h_thumb_fg = sb_style.thumb_color.or(border_color);

                    let thumb =
                        scrollbar_thumb(h_track, viewport_w, computed.content_size.0, scroll_x);
                    for col in 0..h_track {
                        let sb_x = bar_x + i32::from(col);
                        if sb_x >= 0 && h_row >= 0 {
                            let active = thumb.contains(&col);
                            ctx.buffer.set_char(
                                sb_x as u16,
                                h_row as u16,
                                if active { h_thumb_ch } else { h_track_ch },
                                if active { h_thumb_fg } else { h_track_fg },
                                None,
                                active,
                            );
                        }
                    }
                }
                if v_track > 0 && h_track > 0 && v_column >= 0 && h_row >= 0 {
                    ctx.buffer.set_char(
                        v_column as u16,
                        h_row as u16,
                        sb_style.chars.corner,
                        v_track_fg,
                        None,
                        false,
                    );
                }

                // Each bar takes presses: one on the track pages towards it,
                // and one on the thumb drags it.
                if let (Some(id), Some(wheel)) = (scroll_fid, wheel) {
                    let bars = ScrollBars {
                        id,
                        vertical: Bar {
                            start: bar_y,
                            track: v_track,
                            viewport: viewport_h,
                            content: computed.content_size.1,
                        },
                        horizontal: Bar {
                            start: bar_x,
                            track: h_track,
                            viewport: viewport_w,
                            content: computed.content_size.0,
                        },
                        column: v_column,
                    };
                    let handler: crate::element::PointerHandler = {
                        let wheel = wheel.clone();
                        Rc::new(move |event| bars.pointer(event, &wheel))
                    };
                    let mut bar_target = |bx: i32, by: i32, bw: u16, bh: u16| {
                        let target = HitTarget {
                            rect: Rect::new(bx.max(0) as u16, by.max(0) as u16, bw, bh),
                            layer: ctx.layer,
                            dismiss: None,
                            on_click: None,
                            on_click_at: None,
                            focus_id: Some(id),
                            on_key: None,
                            on_scroll: Some(wheel.clone()),
                            on_pointer_enter: None,
                            on_hover: None,
                            on_pointer: Some(handler.clone()),
                            pointer: None,
                        };
                        register_target(ctx, bx, by, target);
                    };
                    if v_track > 0 {
                        bar_target(v_column, bar_y, 1, v_track);
                    }
                    if h_track > 0 {
                        bar_target(bar_x, h_row, h_track, 1);
                    }
                }
            }

            ctx.active_fg = prev_fg;
            ctx.active_bg = prev_bg;
        }
    }
}
