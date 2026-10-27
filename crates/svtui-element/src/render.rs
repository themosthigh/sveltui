use crate::events::{HitMap, HitTarget};
use crate::focus::{is_focused, next_render_focus_id};
use crate::measure::measure_node;
use crate::node::{OverlayElement, VNode};
use crate::scroll::{get_scroll, set_scroll};
use crossterm::style::Color;
use std::rc::Rc;
use svtui_layout::{
    Align, BorderStyle, Justify, Overflow, Rect, ScrollbarVisibility, compute_popover_rect,
};
use svtui_terminal::Buffer;

struct RenderContext<'a> {
    buffer: &'a mut Buffer,
    hit_map: &'a mut HitMap,
    overlays: &'a mut Vec<(OverlayElement, Option<Rect>)>,
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
    let mut overlays = Vec::new();
    {
        let mut ctx = RenderContext {
            buffer,
            hit_map,
            overlays: &mut overlays,
        };
        render_node_offset_internal(
            node,
            rect.x as i32,
            rect.y as i32,
            rect.width,
            rect.height,
            &mut ctx,
        );
    }

    while !overlays.is_empty() {
        let current_overlays = std::mem::take(&mut overlays);
        for (overlay, anchor_rect) in current_overlays {
            render_single_overlay(overlay, anchor_rect, rect, buffer, hit_map, &mut overlays);
        }
    }
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
    let mut overlays = Vec::new();
    {
        let mut ctx = RenderContext {
            buffer,
            hit_map,
            overlays: &mut overlays,
        };
        render_node_offset_internal(node, x, y, width, height, &mut ctx);
    }

    let screen_rect = Rect::new(0, 0, buffer.width, buffer.height);
    while !overlays.is_empty() {
        let current_overlays = std::mem::take(&mut overlays);
        for (overlay, anchor_rect) in current_overlays {
            render_single_overlay(
                overlay,
                anchor_rect,
                screen_rect,
                buffer,
                hit_map,
                &mut overlays,
            );
        }
    }
}

fn render_single_overlay(
    overlay: OverlayElement,
    anchor_rect: Option<Rect>,
    screen: Rect,
    buffer: &mut Buffer,
    hit_map: &mut HitMap,
    overlays: &mut Vec<(OverlayElement, Option<Rect>)>,
) {
    if !overlay.open {
        return;
    }

    if let Some(ref closer) = overlay.on_close {
        hit_map.overlay_closers.push(closer.clone());
    }

    if overlay.backdrop {
        for cell in &mut buffer.cells {
            cell.fg = Some(Color::DarkGrey);
            cell.bold = false;
        }
    }

    if let Some(ref on_close) = overlay.on_close {
        hit_map.register(HitTarget {
            rect: screen,
            on_click: Some(on_close.clone()),
            on_click_at: None,
            focus_id: None,
            on_key: None,
            on_scroll: None,
        });
    }

    if overlay.modal {
        hit_map.modal_focus_start = Some(hit_map.targets.len());
    }

    if let Some(content) = overlay.content {
        let (cw, ch) = measure_node(&content, screen.width, screen.height);
        let popover_rect = compute_popover_rect(overlay.placement, (cw, ch), anchor_rect, screen);

        for cy in popover_rect.y..(popover_rect.y + popover_rect.height) {
            for cx in popover_rect.x..(popover_rect.x + popover_rect.width) {
                buffer.set_char(cx, cy, ' ', None, None, false);
            }
        }

        let mut ctx = RenderContext {
            buffer,
            hit_map,
            overlays,
        };
        render_node_offset_internal(
            &content,
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
    x: i32,
    y: i32,
    width: u16,
    height: u16,
    ctx: &mut RenderContext,
) {
    if let VNode::Overlay(overlay) = node {
        let anchor_rect = if let Some(ref trigger) = overlay.trigger {
            let (tw, th) = measure_node(trigger, width, height);
            let anchor = Rect::new(
                x.max(0) as u16,
                y.max(0) as u16,
                tw.min(width),
                th.min(height),
            );
            render_node_offset_internal(trigger, x, y, tw.min(width), th.min(height), ctx);
            Some(anchor)
        } else {
            None
        };

        if overlay.open {
            ctx.overlays.push((overlay.clone(), anchor_rect));
        }
        return;
    }

    if width == 0 || height == 0 {
        return;
    }
    if x >= ctx.buffer.width as i32 || y >= ctx.buffer.height as i32 {
        return;
    }
    if x + width as i32 <= 0 || y + height as i32 <= 0 {
        return;
    }

    match node {
        VNode::Empty | VNode::Overlay(_) => {}
        VNode::Text(text) => {
            let lines: Vec<&str> = text.content.lines().collect();
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
                for (char_idx, ch) in line.chars().enumerate() {
                    let curr_x = x + char_idx as i32;
                    if curr_x >= x + width as i32 || curr_x >= ctx.buffer.width as i32 {
                        break;
                    }
                    if curr_x >= 0 {
                        let bg = text.bg.or_else(|| {
                            ctx.buffer
                                .get(curr_x as u16, curr_y as u16)
                                .and_then(|c| c.bg)
                        });
                        ctx.buffer.set_char(
                            curr_x as u16,
                            curr_y as u16,
                            ch,
                            text.fg,
                            bg,
                            text.bold,
                        );
                    }
                }
            }
        }
        VNode::Fragment(children) => {
            let mut curr_y = y;
            for child in children {
                let (_, ch) = measure_node(child, width, height);
                render_node_offset_internal(child, x, curr_y, width, ch.min(height), ctx);
                curr_y = curr_y.saturating_add(ch as i32);
                if curr_y >= y + height as i32 {
                    break;
                }
            }
        }
        VNode::Box(b) => {
            // Track focus ancestry hierarchy
            if let Some(id) = b.focus_id {
                for &anc in &ctx.hit_map.focus_stack {
                    ctx.hit_map.ancestors.entry(id).or_default().insert(anc);
                }
                ctx.hit_map.focus_stack.push(id);
            }

            // Fill background if specified
            if let Some(bg) = b.bg {
                for curr_y in y..(y + height as i32) {
                    for curr_x in x..(x + width as i32) {
                        if curr_x >= 0 && curr_y >= 0 {
                            ctx.buffer.set_char(
                                curr_x as u16,
                                curr_y as u16,
                                ' ',
                                b.fg,
                                Some(bg),
                                false,
                            );
                        }
                    }
                }
            }

            let border_w: u16 = if b.border != BorderStyle::None { 1 } else { 0 };
            let border_h: u16 = if b.border != BorderStyle::None { 1 } else { 0 };

            if let Some(chars) = b.border.chars().filter(|_| width >= 2 && height >= 2) {
                let right = x + width as i32 - 1;
                let bottom = y + height as i32 - 1;

                let draw_ch = |cx: i32, cy: i32, ch: char, buf: &mut Buffer| {
                    if cx >= 0 && cy >= 0 {
                        buf.set_char(cx as u16, cy as u16, ch, b.border_color, None, false);
                    }
                };

                draw_ch(x, y, chars.top_left, ctx.buffer);
                draw_ch(right, y, chars.top_right, ctx.buffer);
                draw_ch(x, bottom, chars.bottom_left, ctx.buffer);
                draw_ch(right, bottom, chars.bottom_right, ctx.buffer);

                for cx in (x + 1)..right {
                    draw_ch(cx, y, chars.horizontal, ctx.buffer);
                    draw_ch(cx, bottom, chars.horizontal, ctx.buffer);
                }

                for cy in (y + 1)..bottom {
                    draw_ch(x, cy, chars.vertical, ctx.buffer);
                    draw_ch(right, cy, chars.vertical, ctx.buffer);
                }
            }

            if let Some(title) = b.title.as_deref().filter(|t| width > 4 && !t.is_empty()) {
                let title_text = format!(" {} ", title);
                let max_len = (width.saturating_sub(4)) as usize;
                let displayed_title: String = title_text.chars().take(max_len).collect();
                let start_x = x + 2;
                for (idx, ch) in displayed_title.chars().enumerate() {
                    let cx = start_x + idx as i32;
                    if cx >= 0 && y >= 0 {
                        ctx.buffer
                            .set_char(cx as u16, y as u16, ch, b.border_color, None, true);
                    }
                }
            }

            let inner_x = x + border_w as i32 + b.padding.left as i32;
            let inner_y = y + border_h as i32 + b.padding.top as i32;
            let inner_w = width.saturating_sub(border_w * 2 + b.padding.left + b.padding.right);
            let inner_h = height.saturating_sub(border_h * 2 + b.padding.top + b.padding.bottom);

            if let Some((cx_off, cy_off)) = b.cursor_offset
                && b.focus_id.is_some_and(is_focused)
            {
                let screen_cx = inner_x + cx_off as i32;
                let screen_cy = inner_y + cy_off as i32;
                let clip = ctx.buffer.active_clip();
                let in_clip = clip.width == 0
                    || (screen_cx >= clip.x as i32
                        && screen_cx < (clip.x + clip.width) as i32
                        && screen_cy >= clip.y as i32
                        && screen_cy < (clip.y + clip.height) as i32);
                if in_clip
                    && screen_cx >= 0
                    && screen_cy >= 0
                    && screen_cx < ctx.buffer.width as i32
                    && screen_cy < ctx.buffer.height as i32
                {
                    ctx.buffer.set_cursor(screen_cx as u16, screen_cy as u16);
                }
            }

            if inner_w > 0 && inner_h > 0 {
                let is_scroll_x =
                    b.overflow_x == Overflow::Scroll || b.overflow_x == Overflow::Auto;
                let is_scroll_y =
                    b.overflow_y == Overflow::Scroll || b.overflow_y == Overflow::Auto;

                let avail_w = if is_scroll_x { u16::MAX / 2 } else { inner_w };
                let avail_h = if is_scroll_y { u16::MAX / 2 } else { inner_h };

                let measured_children: Vec<(u16, u16)> = b
                    .children
                    .iter()
                    .map(|c| measure_node(c, avail_w, avail_h))
                    .collect();

                let total_children_w: u16 =
                    measured_children.iter().map(|(w, _)| *w).max().unwrap_or(0);
                let total_children_h: u16 = measured_children.iter().map(|(_, h)| *h).sum();

                let show_v_scrollbar = match b.scrollbar_visibility {
                    ScrollbarVisibility::Always => is_scroll_y,
                    ScrollbarVisibility::Auto => is_scroll_y && total_children_h > inner_h,
                    ScrollbarVisibility::Never => false,
                };

                let show_h_scrollbar = match b.scrollbar_visibility {
                    ScrollbarVisibility::Always => is_scroll_x,
                    ScrollbarVisibility::Auto => is_scroll_x && total_children_w > inner_w,
                    ScrollbarVisibility::Never => false,
                };

                let mut viewport_w = inner_w;
                let mut viewport_h = inner_h;
                if show_v_scrollbar {
                    viewport_w = viewport_w.saturating_sub(1);
                }
                if show_h_scrollbar {
                    viewport_h = viewport_h.saturating_sub(1);
                }

                let is_scrollable = is_scroll_x || is_scroll_y;
                let max_scroll_x = if is_scroll_x {
                    total_children_w.saturating_sub(viewport_w)
                } else {
                    0
                };
                let max_scroll_y = if is_scroll_y {
                    total_children_h.saturating_sub(viewport_h)
                } else {
                    0
                };

                // Resolve current scroll offsets (ensure stable ID for any scrollable container)
                let (scroll_x, scroll_y, scroll_fid) = if is_scrollable {
                    let fid = b.focus_id.unwrap_or_else(next_render_focus_id);
                    let (mut sx, mut sy) = get_scroll(fid);

                    if let Some(x) = b.scroll_x {
                        sx = x;
                    }
                    if let Some(y) = b.scroll_y {
                        sy = y;
                    }

                    sx = sx.min(max_scroll_x);
                    sy = sy.min(max_scroll_y);
                    set_scroll(fid, (sx, sy));
                    (sx, sy, Some(fid))
                } else {
                    (0, 0, b.focus_id)
                };

                // Draw vertical scrollbar
                if show_v_scrollbar && inner_h > 0 && inner_w > 0 {
                    let sb_x = inner_x + inner_w as i32 - 1;
                    let track_h = viewport_h;
                    let thumb_len = if total_children_h > 0 {
                        ((track_h as f32 / total_children_h as f32) * track_h as f32)
                            .round()
                            .clamp(1.0, track_h as f32) as u16
                    } else {
                        track_h
                    };

                    let thumb_travel = track_h.saturating_sub(thumb_len);
                    let thumb_start_y = if max_scroll_y > 0 {
                        inner_y
                            + ((scroll_y as f32 / max_scroll_y as f32) * thumb_travel as f32)
                                .round() as i32
                    } else {
                        inner_y
                    };

                    if sb_x >= 0 {
                        for row in 0..track_h {
                            let cy = inner_y + row as i32;
                            if cy >= 0 {
                                if cy >= thumb_start_y && cy < thumb_start_y + thumb_len as i32 {
                                    ctx.buffer.set_char(
                                        sb_x as u16,
                                        cy as u16,
                                        b.scrollbar_style.chars.vertical_thumb,
                                        b.scrollbar_style.thumb_color,
                                        None,
                                        true,
                                    );
                                } else {
                                    ctx.buffer.set_char(
                                        sb_x as u16,
                                        cy as u16,
                                        b.scrollbar_style.chars.vertical_track,
                                        b.scrollbar_style.track_color,
                                        None,
                                        false,
                                    );
                                }
                            }
                        }
                    }
                }

                // Draw horizontal scrollbar
                if show_h_scrollbar && inner_h > 0 && inner_w > 0 {
                    let sb_y = inner_y + inner_h as i32 - 1;
                    let track_w = viewport_w;
                    let thumb_len = if total_children_w > 0 {
                        ((track_w as f32 / total_children_w as f32) * track_w as f32)
                            .round()
                            .clamp(1.0, track_w as f32) as u16
                    } else {
                        track_w
                    };

                    let thumb_travel = track_w.saturating_sub(thumb_len);
                    let thumb_start_x = if max_scroll_x > 0 {
                        inner_x
                            + ((scroll_x as f32 / max_scroll_x as f32) * thumb_travel as f32)
                                .round() as i32
                    } else {
                        inner_x
                    };

                    if sb_y >= 0 {
                        for col in 0..track_w {
                            let cx = inner_x + col as i32;
                            if cx >= 0 {
                                if cx >= thumb_start_x && cx < thumb_start_x + thumb_len as i32 {
                                    ctx.buffer.set_char(
                                        cx as u16,
                                        sb_y as u16,
                                        b.scrollbar_style.chars.horizontal_thumb,
                                        b.scrollbar_style.thumb_color,
                                        None,
                                        true,
                                    );
                                } else {
                                    ctx.buffer.set_char(
                                        cx as u16,
                                        sb_y as u16,
                                        b.scrollbar_style.chars.horizontal_track,
                                        b.scrollbar_style.track_color,
                                        None,
                                        false,
                                    );
                                }
                            }
                        }
                    }
                }

                // Draw corner intersection if both scrollbars are active
                if show_v_scrollbar && show_h_scrollbar && inner_h > 0 && inner_w > 0 {
                    let corner_x = inner_x + inner_w as i32 - 1;
                    let corner_y = inner_y + inner_h as i32 - 1;
                    if corner_x >= 0 && corner_y >= 0 {
                        ctx.buffer.set_char(
                            corner_x as u16,
                            corner_y as u16,
                            b.scrollbar_style.chars.corner,
                            b.scrollbar_style.track_color,
                            None,
                            false,
                        );
                    }
                }

                // Register interaction
                let on_scroll_handler: Option<Rc<dyn Fn(i16, i16)>> =
                    b.on_scroll.clone().or_else(|| {
                        if is_scrollable {
                            scroll_fid.map(|fid| {
                                Rc::new(move |dx: i16, dy: i16| {
                                    let (cur_x, cur_y) = get_scroll(fid);
                                    let (actual_dx, actual_dy) = if max_scroll_y == 0
                                        && max_scroll_x > 0
                                        && dx == 0
                                        && dy != 0
                                    {
                                        (dy * 2, 0)
                                    } else {
                                        (dx, dy)
                                    };
                                    let new_x = if actual_dx < 0 {
                                        cur_x.saturating_sub((-actual_dx) as u16)
                                    } else {
                                        cur_x.saturating_add(actual_dx as u16).min(max_scroll_x)
                                    };
                                    let new_y = if actual_dy < 0 {
                                        cur_y.saturating_sub((-actual_dy) as u16)
                                    } else {
                                        cur_y.saturating_add(actual_dy as u16).min(max_scroll_y)
                                    };
                                    set_scroll(fid, (new_x, new_y));
                                }) as Rc<dyn Fn(i16, i16)>
                            })
                        } else {
                            None
                        }
                    });

                if b.on_click.is_some()
                    || b.on_click_at.is_some()
                    || b.focus_id.is_some()
                    || b.on_key.is_some()
                    || on_scroll_handler.is_some()
                {
                    let target_rect = Rect::new(x.max(0) as u16, y.max(0) as u16, width, height);
                    ctx.hit_map.register(HitTarget {
                        rect: target_rect,
                        on_click: b.on_click.clone(),
                        on_click_at: b.on_click_at.clone(),
                        focus_id: scroll_fid,
                        on_key: b.on_key.clone(),
                        on_scroll: on_scroll_handler,
                    });
                }

                // Render children clipped to viewport
                let needs_clip =
                    b.overflow_x != Overflow::Visible || b.overflow_y != Overflow::Visible;
                if needs_clip {
                    let clip_rect = Rect::new(
                        inner_x.max(0) as u16,
                        inner_y.max(0) as u16,
                        viewport_w,
                        viewport_h,
                    );
                    ctx.buffer.push_clip(clip_rect);
                }

                if !b.children.is_empty() {
                    let start_y = match b.align_items {
                        Align::Center => {
                            inner_y + (viewport_h as i32 - total_children_h as i32) / 2
                        }
                        Align::End => inner_y + (viewport_h as i32 - total_children_h as i32),
                        _ => inner_y,
                    } - scroll_y as i32;

                    let mut curr_y = start_y;
                    for (child, &(cw, ch)) in b.children.iter().zip(measured_children.iter()) {
                        let start_x = match b.justify_content {
                            Justify::Center => inner_x + (viewport_w as i32 - cw as i32) / 2,
                            Justify::End => inner_x + (viewport_w as i32 - cw as i32),
                            _ => inner_x,
                        } - scroll_x as i32;

                        render_node_offset_internal(child, start_x, curr_y, cw, ch, ctx);
                        curr_y += ch as i32;
                    }
                }

                if needs_clip {
                    ctx.buffer.pop_clip();
                }
            } else if b.on_click.is_some()
                || b.on_click_at.is_some()
                || b.focus_id.is_some()
                || b.on_key.is_some()
                || b.on_scroll.is_some()
            {
                let target_rect = Rect::new(x.max(0) as u16, y.max(0) as u16, width, height);
                ctx.hit_map.register(HitTarget {
                    rect: target_rect,
                    on_click: b.on_click.clone(),
                    on_click_at: b.on_click_at.clone(),
                    focus_id: b.focus_id,
                    on_key: b.on_key.clone(),
                    on_scroll: b.on_scroll.clone(),
                });
            }

            // Pop from focus ancestry stack
            if b.focus_id.is_some() {
                ctx.hit_map.focus_stack.pop();
            }
        }
    }
}
