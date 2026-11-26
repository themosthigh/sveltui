//! Fetching and decoding images off the UI thread. Every URL is loaded at
//! most once and the result is kept for the life of the program.

use std::collections::HashMap;
use std::io::Read;
use std::sync::{Arc, LazyLock, Mutex};
use std::time::Duration;
use sveltui::Runtime;

#[derive(Clone)]
pub(crate) enum Entry {
    Loading,
    Ready(Arc<image::RgbaImage>),
    Failed(String),
}

static CACHE: LazyLock<Mutex<HashMap<String, Entry>>> = LazyLock::new(Mutex::default);

/// Forgets every loaded image, so the next request fetches again.
pub fn clear_cache() {
    if let Ok(mut cache) = CACHE.lock() {
        cache.clear();
    }
}

/// Why `src` failed to load, once it has.
pub fn load_error(src: &str) -> Option<String> {
    match CACHE.lock().ok()?.get(src)? {
        Entry::Failed(error) => Some(error.clone()),
        _ => None,
    }
}

/// What is known about `src`, starting a load the first time it is asked
/// for. The load finishes on its own thread and repaints the app.
pub(crate) fn request(src: &str, timeout: Duration, max_bytes: usize) -> Entry {
    let Ok(mut cache) = CACHE.lock() else {
        return Entry::Failed(String::from("image cache is unavailable"));
    };
    if let Some(entry) = cache.get(src) {
        return entry.clone();
    }
    cache.insert(src.to_owned(), Entry::Loading);
    let src = src.to_owned();
    std::thread::spawn(move || {
        let entry = match fetch(&src, timeout, max_bytes).and_then(|bytes| decode(&bytes)) {
            Ok(img) => Entry::Ready(Arc::new(img)),
            Err(error) => Entry::Failed(error),
        };
        if let Ok(mut cache) = CACHE.lock() {
            cache.insert(src, entry);
        }
        Runtime::set_dirty(true);
    });
    Entry::Loading
}

fn decode(bytes: &[u8]) -> Result<image::RgbaImage, String> {
    image::load_from_memory(bytes)
        .map(|img| img.to_rgba8())
        .map_err(|error| error.to_string())
}

fn fetch(src: &str, timeout: Duration, max_bytes: usize) -> Result<Vec<u8>, String> {
    if !(src.starts_with("http://") || src.starts_with("https://")) {
        let bytes = std::fs::read(src).map_err(|error| format!("{src}: {error}"))?;
        return if bytes.len() > max_bytes {
            Err(format!("{src}: larger than {max_bytes} bytes"))
        } else {
            Ok(bytes)
        };
    }
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(timeout))
        .max_redirects(5)
        // Wikimedia and others turn away clients that don't say who they are.
        .user_agent(concat!(
            "sveltui-image/",
            env!("CARGO_PKG_VERSION"),
            " (https://github.com/sveltui/sveltui)"
        ))
        .build()
        .into();
    let response = agent.get(src).call().map_err(|error| error.to_string())?;
    let mut bytes = Vec::new();
    response
        .into_body()
        .into_reader()
        .take(max_bytes as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    if bytes.len() > max_bytes {
        return Err(format!("{src}: larger than {max_bytes} bytes"));
    }
    Ok(bytes)
}
