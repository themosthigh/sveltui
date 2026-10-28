use crate::watcher::FileWatcher;
use crossterm::event::{
    self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEventKind,
};
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;
use svtui_element::{
    DispatchOutcome, EventContext, EventResult, FocusManager, HitMap, VNode,
    render_node_interactive, reset_focus_counter,
};
use svtui_layout::Rect;
use svtui_reactive::Runtime;
use svtui_terminal::Terminal;

pub struct App<V, H = fn(KeyEvent, &mut EventContext) -> EventResult, S = fn(&Path)> {
    view: V,
    key_handler: Option<H>,
    save_handler: Option<S>,
    watch_paths: Vec<PathBuf>,
}

impl<V> App<V, fn(KeyEvent, &mut EventContext) -> EventResult, fn(&Path)>
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
    H: FnMut(KeyEvent, &mut EventContext) -> EventResult,
    S: FnMut(&Path),
{
    pub fn on_key<H2>(self, handler: H2) -> App<V, H2, S>
    where
        H2: FnMut(KeyEvent, &mut EventContext) -> EventResult,
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

    /// Dispatch without a terminal, useful for application tests and adapters.
    pub fn dispatch_key(&mut self, focus: &mut FocusManager, key: KeyEvent) -> DispatchOutcome {
        let mut outcome = focus.dispatch(key);
        if !outcome.handled
            && key.kind != KeyEventKind::Release
            && let Some(handler) = &mut self.key_handler
        {
            let mut context = focus.event_context();
            outcome.handled =
                focus.with_context(|| handler(key, &mut context)) == EventResult::Handled;
            let commands = focus.apply(context);
            outcome.changed |= commands.changed || outcome.handled;
            outcome.exit |= commands.exit;
        }
        outcome
    }

    pub fn run(mut self) -> io::Result<()> {
        let mut terminal = Terminal::new()?;
        let (mut term_w, mut term_h) = terminal.size();

        // Default watch path if none specified: current directory
        if self.watch_paths.is_empty() {
            self.watch_paths.push(PathBuf::from("."));
        }

        let mut watcher = FileWatcher::new(&self.watch_paths).ok();
        let mut hit_map = HitMap::new();
        let mut focus = FocusManager::new();

        // Initial render
        Runtime::set_dirty(true);

        loop {
            // Check for file save/modification events
            if let Some(ref mut w) = watcher
                && let Some(saved_path) = w.poll_save()
            {
                if let Some(ref mut handler) = self.save_handler {
                    handler(&saved_path);
                }
                // Repaint the UI on save
                Runtime::set_dirty(true);
            }

            // Re-render if dirty
            if Runtime::is_dirty() {
                // Clear before construction so writes during rendering survive.
                Runtime::set_dirty(false);
                reset_focus_counter();
                svtui_reactive::begin_frame();
                let root_node = focus.with_context(&mut self.view);
                svtui_reactive::end_frame();
                terminal.curr_buffer.clear();
                hit_map.clear();
                let screen_rect = Rect::new(0, 0, term_w, term_h);
                focus.with_context(|| {
                    render_node_interactive(
                        &root_node,
                        screen_rect,
                        &mut terminal.curr_buffer,
                        &mut hit_map,
                    )
                });
                let changed = focus.reconcile(hit_map.interactions.clone());
                let revealed = focus.focused().is_some_and(|id| hit_map.reveal(id));
                if changed || revealed {
                    Runtime::set_dirty(true);
                    continue; // paint the new focus before the first flush
                }
                if Runtime::is_dirty() {
                    continue;
                }
                terminal.flush()?;
            }

            // Poll for terminal events with 16ms timeout (~60 FPS)
            if event::poll(Duration::from_millis(16))? {
                match event::read()? {
                    Event::Mouse(mouse) => {
                        let outcome = match mouse.kind {
                            MouseEventKind::Down(MouseButton::Left) => hit_map
                                .hit_test(mouse.column, mouse.row)
                                .map(|target| focus.pointer_down(target, mouse.column, mouse.row)),
                            MouseEventKind::ScrollDown
                            | MouseEventKind::ScrollUp
                            | MouseEventKind::ScrollLeft
                            | MouseEventKind::ScrollRight => {
                                let (dx, dy) = match mouse.kind {
                                    MouseEventKind::ScrollDown
                                        if mouse.modifiers.contains(KeyModifiers::SHIFT) =>
                                    {
                                        (2, 0)
                                    }
                                    MouseEventKind::ScrollUp
                                        if mouse.modifiers.contains(KeyModifiers::SHIFT) =>
                                    {
                                        (-2, 0)
                                    }
                                    MouseEventKind::ScrollDown => (0, 1),
                                    MouseEventKind::ScrollUp => (0, -1),
                                    MouseEventKind::ScrollLeft => (-2, 0),
                                    _ => (2, 0),
                                };
                                hit_map
                                    .scroll_target(mouse.column, mouse.row)
                                    .map(|target| focus.scroll(target, dx, dy))
                            }
                            _ => None,
                        };
                        if let Some(outcome) = outcome {
                            if outcome.exit {
                                break;
                            }
                            if outcome.changed {
                                Runtime::set_dirty(true);
                            }
                        }
                    }
                    Event::Key(key) => {
                        let outcome = self.dispatch_key(&mut focus, key);
                        if outcome.exit {
                            break;
                        }
                        if outcome.changed {
                            Runtime::set_dirty(true);
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

/// Explicit example/application exit policy. App itself reserves no shortcuts.
/// Focused widgets and scopes receive these keys before this fallback runs.
pub fn exit_shortcuts(key: KeyEvent, context: &mut EventContext) -> EventResult {
    if key.code == KeyCode::Esc
        || key.code == KeyCode::Char('q')
        || (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL))
    {
        context.exit();
        EventResult::Handled
    } else {
        EventResult::Ignored
    }
}
