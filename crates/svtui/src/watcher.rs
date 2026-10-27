use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver};
use std::time::{Duration, Instant};

pub struct FileWatcher {
    _watcher: RecommendedWatcher,
    rx: Receiver<PathBuf>,
    last_event: Instant,
}

impl FileWatcher {
    pub fn new<P: AsRef<Path>>(paths: &[P]) -> notify::Result<Self> {
        let (tx, rx) = channel();

        let mut watcher = RecommendedWatcher::new(
            move |res: notify::Result<Event>| {
                if let Ok(event) = res {
                    match event.kind {
                        EventKind::Modify(_) | EventKind::Create(_) => {
                            for path in event.paths {
                                // Filter out git or target folders
                                let path_str = path.to_string_lossy();
                                if !path_str.contains("/.git/") && !path_str.contains("/target/") {
                                    let _ = tx.send(path);
                                }
                            }
                        }
                        _ => {}
                    }
                }
            },
            Config::default(),
        )?;

        for p in paths {
            let path = p.as_ref();
            if path.exists() {
                let mode = if path.is_dir() {
                    RecursiveMode::Recursive
                } else {
                    RecursiveMode::NonRecursive
                };
                let _ = watcher.watch(path, mode);
            }
        }

        Ok(Self {
            _watcher: watcher,
            rx,
            last_event: Instant::now() - Duration::from_secs(1),
        })
    }

    /// Checks if a file modification event occurred (with 100ms debouncing).
    pub fn poll_save(&mut self) -> Option<PathBuf> {
        let mut triggered = None;
        while let Ok(path) = self.rx.try_recv() {
            triggered = Some(path);
        }

        if let Some(path) = triggered {
            if self.last_event.elapsed() >= Duration::from_millis(100) {
                self.last_event = Instant::now();
                return Some(path);
            }
        }
        None
    }
}
