use crate::watcher::FileWatcher;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEventKind};
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;
use svtui_element::{
    FocusId, HitMap, VNode, get_focused, render_node_interactive, reset_focus_counter, set_focused,
    update_focus_context,
};
use svtui_layout::Rect;
use svtui_reactive::Runtime;
use svtui_terminal::Terminal;

pub struct App<V, H = fn(KeyEvent), S = fn(&Path)> {
    view: V,
    key_handler: Option<H>,
    save_handler: Option<S>,
    watch_paths: Vec<PathBuf>,
}

impl<V> App<V, fn(KeyEvent), fn(&Path)>
where
    V: FnMut() -> VNode,
{
    pub fn new(view: V) -> Self {
        Self {
            view,
            key_handler: None,
            save_handler: None,
            watch_paths: Vec::new(),
        }
    }
}

impl<V, H, S> App<V, H, S>
where
    V: FnMut() -> VNode,
    H: FnMut(KeyEvent),
    S: FnMut(&Path),
{
    pub fn on_key<H2>(self, handler: H2) -> App<V, H2, S>
    where
        H2: FnMut(KeyEvent),
    {
        App {
            view: self.view,
            key_handler: Some(handler),
            save_handler: self.save_handler,
            watch_paths: self.watch_paths,
        }
    }

    pub fn on_save<S2>(self, handler: S2) -> App<V, H, S2>
    where
        S2: FnMut(&Path),
    {
        App {
            view: self.view,
            key_handler: self.key_handler,
            save_handler: Some(handler),
            watch_paths: self.watch_paths,
        }
    }

    pub fn watch_path(mut self, path: impl AsRef<Path>) -> Self {
        self.watch_paths.push(path.as_ref().to_path_buf());
        self
    }

    #[allow(clippy::collapsible_if)]
    pub fn run(mut self) -> io::Result<()> {
        let mut terminal = Terminal::new()?;
        let (mut term_w, mut term_h) = terminal.size();

        // Default watch path if none specified: current directory
        if self.watch_paths.is_empty() {
            self.watch_paths.push(PathBuf::from("."));
        }

        let mut watcher = FileWatcher::new(&self.watch_paths).ok();
        let mut hit_map = HitMap::new();
        let mut focused_id: Option<FocusId> = None;

        // Initial render
        Runtime::set_dirty(true);

        loop {
            // Check for file save/modification events
            if let Some(ref mut w) = watcher {
                if let Some(saved_path) = w.poll_save() {
                    if let Some(ref mut handler) = self.save_handler {
                        handler(&saved_path);
                    }
                    // Repaint the UI on save
                    Runtime::set_dirty(true);
                }
            }

            // Re-render if dirty
            if Runtime::is_dirty() {
                if let Some(new_f) = get_focused() {
                    focused_id = Some(new_f);
                }
                terminal.curr_buffer.clear();
                reset_focus_counter();
                update_focus_context(focused_id, &hit_map);

                hit_map.clear();
                let root_node = (self.view)();
                let screen_rect = Rect::new(0, 0, term_w, term_h);
                render_node_interactive(
                    &root_node,
                    screen_rect,
                    &mut terminal.curr_buffer,
                    &mut hit_map,
                );

                if hit_map.modal_focus_start.is_some() {
                    let modal_focusables = hit_map.focusable_ids();
                    if !modal_focusables.is_empty()
                        && !focused_id.is_some_and(|id| modal_focusables.contains(&id))
                    {
                        focused_id = modal_focusables.first().copied();
                    }
                }

                update_focus_context(focused_id, &hit_map);

                terminal.flush()?;
                Runtime::set_dirty(false);
            }

            // Poll for terminal events with 16ms timeout (~60 FPS)
            if event::poll(Duration::from_millis(16))? {
                match event::read()? {
                    Event::Mouse(mouse) => match mouse.kind {
                        MouseEventKind::Down(MouseButton::Left) => {
                            if let Some(target) = hit_map.hit_test(mouse.column, mouse.row) {
                                if let Some(fid) = target.focus_id {
                                    focused_id = Some(fid);
                                }
                                if let Some(ref on_click) = target.on_click {
                                    on_click();
                                }
                                if let Some(ref on_click_at) = target.on_click_at {
                                    let rel_x = mouse.column.saturating_sub(target.rect.x);
                                    let rel_y = mouse.row.saturating_sub(target.rect.y);
                                    on_click_at(rel_x, rel_y);
                                }
                                Runtime::set_dirty(true);
                            }
                        }
                        MouseEventKind::ScrollDown => {
                            if let Some(target) = hit_map.scroll_target(mouse.column, mouse.row) {
                                if let Some(ref on_scroll) = target.on_scroll {
                                    if mouse.modifiers.contains(KeyModifiers::SHIFT) {
                                        on_scroll(2, 0);
                                    } else {
                                        on_scroll(0, 1);
                                    }
                                    Runtime::set_dirty(true);
                                }
                            }
                        }
                        MouseEventKind::ScrollUp => {
                            if let Some(target) = hit_map.scroll_target(mouse.column, mouse.row) {
                                if let Some(ref on_scroll) = target.on_scroll {
                                    if mouse.modifiers.contains(KeyModifiers::SHIFT) {
                                        on_scroll(-2, 0);
                                    } else {
                                        on_scroll(0, -1);
                                    }
                                    Runtime::set_dirty(true);
                                }
                            }
                        }
                        MouseEventKind::ScrollLeft => {
                            if let Some(target) = hit_map.scroll_target(mouse.column, mouse.row) {
                                if let Some(ref on_scroll) = target.on_scroll {
                                    on_scroll(-2, 0);
                                    Runtime::set_dirty(true);
                                }
                            }
                        }
                        MouseEventKind::ScrollRight => {
                            if let Some(target) = hit_map.scroll_target(mouse.column, mouse.row) {
                                if let Some(ref on_scroll) = target.on_scroll {
                                    on_scroll(2, 0);
                                    Runtime::set_dirty(true);
                                }
                            }
                        }
                        _ => {}
                    },
                    Event::Key(key) => {
                        // Quit condition: Ctrl+C, or 'q' (if no input focused)
                        if (key.code == KeyCode::Char('c')
                            && key.modifiers.contains(KeyModifiers::CONTROL))
                            || (key.code == KeyCode::Char('q') && focused_id.is_none())
                        {
                            break;
                        }

                        // Esc key: Light dismiss topmost submenu or overlay if any is open, otherwise quit
                        if key.code == KeyCode::Esc {
                            if svtui_components::close_topmost_submenu().is_some() {
                                if let Some(new_f) = get_focused() {
                                    focused_id = Some(new_f);
                                }
                                Runtime::set_dirty(true);
                                continue;
                            }
                            if let Some(closer) = hit_map.overlay_closers.pop() {
                                closer();
                                Runtime::set_dirty(true);
                                continue;
                            } else {
                                break;
                            }
                        }

                        // Save shortcut: Ctrl+S
                        if key.code == KeyCode::Char('s')
                            && key.modifiers.contains(KeyModifiers::CONTROL)
                        {
                            if let Some(ref mut handler) = self.save_handler {
                                handler(Path::new("shortcut:Ctrl+S"));
                            }
                            Runtime::set_dirty(true);
                            continue;
                        }

                        // Tab / BackTab navigation between focusable widgets
                        if key.code == KeyCode::Tab || key.code == KeyCode::BackTab {
                            let focusable = hit_map.focusable_ids();
                            if !focusable.is_empty() {
                                let is_back = key.code == KeyCode::BackTab
                                    || key.modifiers.contains(KeyModifiers::SHIFT);
                                let current_idx = focused_id
                                    .and_then(|id| focusable.iter().position(|&x| x == id));
                                let next_idx = match current_idx {
                                    Some(idx) => {
                                        if is_back {
                                            if idx == 0 {
                                                focusable.len() - 1
                                            } else {
                                                idx - 1
                                            }
                                        } else {
                                            (idx + 1) % focusable.len()
                                        }
                                    }
                                    None => {
                                        if is_back {
                                            focusable.len() - 1
                                        } else {
                                            0
                                        }
                                    }
                                };
                                focused_id = Some(focusable[next_idx]);
                                set_focused(focused_id);
                                Runtime::set_dirty(true);
                                continue;
                            }
                        }

                        // Up / Down arrow navigation is STRICTLY within dropdown menu items.
                        // Every other focus is TAB ONLY.
                        if (key.code == KeyCode::Down || key.code == KeyCode::Up)
                            && focused_id.is_some_and(svtui_components::is_menu_item)
                        {
                            let current_id = focused_id.unwrap();
                            let container_id = hit_map.ancestors.get(&current_id).and_then(|anc| {
                                hit_map
                                    .focus_stack
                                    .iter()
                                    .rev()
                                    .find(|id| anc.contains(id))
                                    .copied()
                            });

                            let siblings: Vec<FocusId> = hit_map
                                .focusable_ids()
                                .into_iter()
                                .filter(|&id| {
                                    svtui_components::is_menu_item(id)
                                        && match container_id {
                                            Some(cid) => hit_map
                                                .ancestors
                                                .get(&id)
                                                .is_some_and(|anc| anc.contains(&cid)),
                                            None => true,
                                        }
                                })
                                .collect();

                            if !siblings.is_empty() {
                                let is_up = key.code == KeyCode::Up;
                                let current_idx = siblings.iter().position(|&x| x == current_id);
                                let next_idx = match current_idx {
                                    Some(idx) => {
                                        if is_up {
                                            if idx == 0 {
                                                siblings.len() - 1
                                            } else {
                                                idx - 1
                                            }
                                        } else {
                                            (idx + 1) % siblings.len()
                                        }
                                    }
                                    None => {
                                        if is_up {
                                            siblings.len() - 1
                                        } else {
                                            0
                                        }
                                    }
                                };
                                focused_id = Some(siblings[next_idx]);
                                set_focused(focused_id);
                                Runtime::set_dirty(true);
                                continue;
                            }
                        } else if key.code == KeyCode::Down {
                            // If a dropdown menu is open and trigger is focused, Down enters the first menu item
                            let first_menu_item = hit_map
                                .focusable_ids()
                                .into_iter()
                                .find(|&id| svtui_components::is_menu_item(id));
                            if let Some(first_id) = first_menu_item {
                                focused_id = Some(first_id);
                                set_focused(focused_id);
                                Runtime::set_dirty(true);
                                continue;
                            }
                        }

                        // Route key to focused component if applicable
                        let mut consumed = false;
                        if let Some(fid) = focused_id {
                            if let Some(target) =
                                hit_map.targets.iter().find(|t| t.focus_id == Some(fid))
                            {
                                if let Some(ref on_key) = target.on_key {
                                    on_key(key);
                                    if let Some(new_f) = get_focused() {
                                        focused_id = Some(new_f);
                                    }
                                    Runtime::set_dirty(true);
                                    consumed = true;
                                }
                                if !consumed {
                                    if let Some(ref on_scroll) = target.on_scroll {
                                        match key.code {
                                            KeyCode::Up => {
                                                on_scroll(0, -1);
                                                Runtime::set_dirty(true);
                                                consumed = true;
                                            }
                                            KeyCode::Down => {
                                                on_scroll(0, 1);
                                                Runtime::set_dirty(true);
                                                consumed = true;
                                            }
                                            KeyCode::PageUp => {
                                                on_scroll(0, -5);
                                                Runtime::set_dirty(true);
                                                consumed = true;
                                            }
                                            KeyCode::PageDown => {
                                                on_scroll(0, 5);
                                                Runtime::set_dirty(true);
                                                consumed = true;
                                            }
                                            KeyCode::Left => {
                                                on_scroll(-2, 0);
                                                Runtime::set_dirty(true);
                                                consumed = true;
                                            }
                                            KeyCode::Right => {
                                                on_scroll(2, 0);
                                                Runtime::set_dirty(true);
                                                consumed = true;
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                                if !consumed {
                                    if let Some(ref on_click) = target.on_click {
                                        if key.code == KeyCode::Enter
                                            || key.code == KeyCode::Char(' ')
                                        {
                                            on_click();
                                            Runtime::set_dirty(true);
                                            consumed = true;
                                        }
                                    }
                                }
                            }
                        }

                        // Fallback: If no widget consumed the key, route scroll keys to the primary scroll container
                        if !consumed {
                            if let Some(target) =
                                hit_map.targets.iter().find(|t| t.on_scroll.is_some())
                            {
                                if let Some(ref on_scroll) = target.on_scroll {
                                    match key.code {
                                        KeyCode::Up => {
                                            on_scroll(0, -1);
                                            Runtime::set_dirty(true);
                                            consumed = true;
                                        }
                                        KeyCode::Down => {
                                            on_scroll(0, 1);
                                            Runtime::set_dirty(true);
                                            consumed = true;
                                        }
                                        KeyCode::PageUp => {
                                            on_scroll(0, -5);
                                            Runtime::set_dirty(true);
                                            consumed = true;
                                        }
                                        KeyCode::PageDown => {
                                            on_scroll(0, 5);
                                            Runtime::set_dirty(true);
                                            consumed = true;
                                        }
                                        KeyCode::Left => {
                                            on_scroll(-2, 0);
                                            Runtime::set_dirty(true);
                                            consumed = true;
                                        }
                                        KeyCode::Right => {
                                            on_scroll(2, 0);
                                            Runtime::set_dirty(true);
                                            consumed = true;
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }

                        if !consumed {
                            if let Some(ref mut handler) = self.key_handler {
                                handler(key);
                            }
                        }
                    }
                    Event::Resize(w, h) => {
                        let (real_w, real_h) = crossterm::terminal::size().unwrap_or((w, h));
                        term_w = real_w;
                        term_h = real_h;
                        terminal.resize(real_w, real_h);
                        Runtime::set_dirty(true);
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }
}
