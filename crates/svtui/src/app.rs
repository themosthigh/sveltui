use crate::watcher::FileWatcher;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;
use svtui_element::{render_node, VNode};
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

    pub fn run(mut self) -> io::Result<()> {
        let mut terminal = Terminal::new()?;
        let (mut term_w, mut term_h) = terminal.size();

        // Default watch path if none specified: current directory
        if self.watch_paths.is_empty() {
            self.watch_paths.push(PathBuf::from("."));
        }

        let mut watcher = FileWatcher::new(&self.watch_paths).ok();

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
                terminal.curr_buffer.clear();
                let root_node = (self.view)();
                let screen_rect = Rect::new(0, 0, term_w, term_h);
                render_node(&root_node, screen_rect, &mut terminal.curr_buffer);
                terminal.flush()?;
                Runtime::set_dirty(false);
            }

            // Poll for terminal events with 16ms timeout (~60 FPS)
            if event::poll(Duration::from_millis(16))? {
                match event::read()? {
                    Event::Key(key) => {
                        // Quit condition: Ctrl+C, 'q', or Esc
                        if (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL))
                            || key.code == KeyCode::Char('q')
                            || key.code == KeyCode::Esc
                        {
                            break;
                        }

                        // Save shortcut: Ctrl+S
                        if key.code == KeyCode::Char('s') && key.modifiers.contains(KeyModifiers::CONTROL) {
                            if let Some(ref mut handler) = self.save_handler {
                                handler(Path::new("shortcut:Ctrl+S"));
                            }
                            // Repaint UI on save
                            Runtime::set_dirty(true);
                            continue;
                        }

                        if let Some(ref mut handler) = self.key_handler {
                            handler(key);
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
