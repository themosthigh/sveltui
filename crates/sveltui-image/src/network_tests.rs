use crate::*;
use image::{Rgba, RgbaImage};
use std::cell::RefCell;
use std::io::{Cursor, Read, Write};
use std::net::TcpListener;
use std::rc::Rc;
use std::time::{Duration, Instant};
use sveltui::element::*;
use sveltui::layout::Rect;
use sveltui::prelude::*;
use sveltui::terminal::Buffer;

fn png() -> Vec<u8> {
    let img = RgbaImage::from_fn(2, 2, |_, y| {
        if y == 0 {
            Rgba([255, 0, 0, 255])
        } else {
            Rgba([0, 0, 255, 255])
        }
    });
    let mut out = Cursor::new(Vec::new());
    img.write_to(&mut out, image::ImageFormat::Png).unwrap();
    out.into_inner()
}

/// Serves one response (after `delay`) to one request on a free port, and
/// returns the URL.
fn serve(status: &'static str, body: Vec<u8>, delay: Duration) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}/picture.png", listener.local_addr().unwrap());
    std::thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut request = [0u8; 2048];
            let _ = stream.read(&mut request);
            std::thread::sleep(delay);
            let head = format!(
                "HTTP/1.1 {status}\r\nContent-Type: image/png\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            let _ = stream.write_all(head.as_bytes());
            let _ = stream.write_all(&body);
        }
    });
    url
}

fn text(buffer: &Buffer) -> String {
    (0..buffer.height)
        .map(|y| {
            (0..buffer.width)
                .map(|x| buffer.get(x, y).unwrap().ch)
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn frame(build: &impl Fn() -> VNode) -> Buffer {
    let manager = FocusManager::new();
    reset_focus_counter();
    begin_frame();
    let node = manager.with_context(build);
    end_frame();
    let mut buffer = Buffer::new(30, 6);
    let mut hits = HitMap::new();
    manager.with_context(|| {
        render_node_interactive(&node, Rect::new(0, 0, 30, 6), &mut buffer, &mut hits)
    });
    buffer
}

/// Renders until `done` holds for the screen, or three seconds pass.
fn until(build: &impl Fn() -> VNode, done: impl Fn(&Buffer) -> bool) -> Buffer {
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        let buffer = frame(build);
        if done(&buffer) || Instant::now() > deadline {
            return buffer;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}

fn has_pixels(buffer: &Buffer) -> bool {
    text(buffer).contains('▀')
}

#[test]
fn an_image_loads_in_the_background_and_replaces_the_fallback() {
    let url = serve("200 OK", png(), Duration::from_millis(50));
    let build = {
        let url = url.clone();
        move || {
            let url = url.clone();
            view! {
                <NetworkImage::Root width=4 height=2 protocol=ImageProtocol::HalfBlock>
                    <NetworkImage::Image src={url.clone()} alt="A flag" />
                    <NetworkImage::Fallback>"..."</NetworkImage::Fallback>
                </NetworkImage::Root>
            }
        }
    };
    let first = frame(&build);
    assert!(text(&first).starts_with("..."), "{}", text(&first));
    assert!(!has_pixels(&first));

    let loaded = until(&build, has_pixels);
    assert!(has_pixels(&loaded), "never loaded: {:?}", load_error(&url));
    assert!(!text(&loaded).contains("..."), "fallback should be gone");
}

#[test]
fn a_failed_request_shows_the_fallback_straight_away() {
    let url = serve("404 Not Found", b"nope".to_vec(), Duration::ZERO);
    let build = {
        let url = url.clone();
        move || {
            let url = url.clone();
            view! {
                <NetworkImage::Root width=4 height=2 protocol=ImageProtocol::HalfBlock>
                    <NetworkImage::Image src={url.clone()} alt="Ada" />
                    <NetworkImage::Fallback delay_ms=60_000>"AL"</NetworkImage::Fallback>
                </NetworkImage::Root>
            }
        }
    };
    let buffer = until(&build, |b| text(b).starts_with("AL"));
    assert!(text(&buffer).starts_with("AL"), "{}", text(&buffer));
    assert!(load_error(&url).is_some());
}

#[test]
fn the_fallback_waits_out_its_delay_while_loading() {
    let url = serve("200 OK", png(), Duration::from_millis(400));
    let build = {
        let url = url.clone();
        move || {
            let url = url.clone();
            view! {
                <NetworkImage::Root width=4 height=2 protocol=ImageProtocol::HalfBlock>
                    <NetworkImage::Image src={url.clone()} alt="Ada" />
                    <NetworkImage::Fallback delay_ms=60_000>"AL"</NetworkImage::Fallback>
                </NetworkImage::Root>
            }
        }
    };
    let early = frame(&build);
    assert!(!text(&early).contains("AL"), "shown before its delay");
    let loaded = until(&build, has_pixels);
    assert!(has_pixels(&loaded));
}

#[test]
fn sources_are_tried_in_order() {
    let broken = serve("500 Internal Server Error", Vec::new(), Duration::ZERO);
    let working = serve("200 OK", png(), Duration::ZERO);
    let build = {
        let (broken, working) = (broken.clone(), working.clone());
        move || {
            let (broken, working) = (broken.clone(), working.clone());
            view! {
                <NetworkImage::Root width=4 height=2 protocol=ImageProtocol::HalfBlock>
                    <NetworkImage::Image src={broken.clone()} alt="Ada" />
                    <NetworkImage::Image src={working.clone()} alt="Ada" />
                    <NetworkImage::Fallback>"AL"</NetworkImage::Fallback>
                </NetworkImage::Root>
            }
        }
    };
    let buffer = until(&build, has_pixels);
    assert!(has_pixels(&buffer), "{}", text(&buffer));
    assert!(!text(&buffer).contains("AL"));
}

#[test]
fn a_response_over_the_size_limit_is_an_error() {
    let url = serve("200 OK", png(), Duration::ZERO);
    let build = {
        let url = url.clone();
        move || {
            let url = url.clone();
            view! {
                <NetworkImage::Root width=4 height=2 protocol=ImageProtocol::HalfBlock>
                    <NetworkImage::Image src={url.clone()} alt="Ada" max_bytes=10 />
                    <NetworkImage::Fallback>"AL"</NetworkImage::Fallback>
                </NetworkImage::Root>
            }
        }
    };
    until(&build, |_| load_error(&url).is_some());
    let buffer = frame(&build);
    assert!(text(&buffer).starts_with("AL"));
    assert!(load_error(&url).unwrap().contains("larger than 10 bytes"));
}

#[test]
fn a_local_file_works_as_the_last_resort() {
    let path = std::env::temp_dir().join(format!("sveltui-image-{}.png", std::process::id()));
    std::fs::write(&path, png()).unwrap();
    let src = path.to_string_lossy().into_owned();
    let build = {
        let src = src.clone();
        move || {
            let src = src.clone();
            view! {
                <NetworkImage::Root width=4 height=2 protocol=ImageProtocol::HalfBlock>
                    <NetworkImage::Image src="http://127.0.0.1:1/never.png" alt="Ada" />
                    <NetworkImage::Image src={src.clone()} alt="Ada" />
                </NetworkImage::Root>
            }
        }
    };
    let buffer = until(&build, has_pixels);
    let _ = std::fs::remove_file(path);
    assert!(has_pixels(&buffer), "{}", text(&buffer));
}

#[test]
fn the_status_is_reported_as_loading_progresses() {
    let url = serve("200 OK", png(), Duration::from_millis(100));
    let seen = Rc::new(RefCell::new(Vec::new()));
    let build = {
        let (url, seen) = (url.clone(), seen.clone());
        move || {
            let url = url.clone();
            let seen = seen.clone();
            let on_change: Rc<dyn Fn(ImageStatus)> =
                Rc::new(move |status| seen.borrow_mut().push(status));
            view! {
                <NetworkImage::Root width=4 height=2 protocol=ImageProtocol::HalfBlock
                    on_loading_status_change=on_change>
                    <NetworkImage::Image src={url.clone()} alt="Ada" />
                </NetworkImage::Root>
            }
        }
    };
    until(&build, has_pixels);
    let seen = seen.borrow();
    assert_eq!(seen.first(), Some(&ImageStatus::Loading));
    assert_eq!(seen.last(), Some(&ImageStatus::Loaded));
}

/// Needs the internet: `cargo test -p sveltui-image -- --ignored`.
#[test]
#[ignore = "needs internet access"]
fn a_wikimedia_thumbnail_loads_over_https() {
    let url = String::from(
        "https://thumb.wikimedia.org/wikipedia/commons/thumb/0/09/Frida_Kahlo%2C_by_Guillermo_Kahlo_%28cropped%29.jpg/500px-Frida_Kahlo%2C_by_Guillermo_Kahlo_%28cropped%29.jpg",
    );
    let build = {
        let url = url.clone();
        move || {
            let url = url.clone();
            view! {
                <NetworkImage::Root width=20 height=10 protocol=ImageProtocol::HalfBlock>
                    <NetworkImage::Image src={url.clone()} alt="Frida Kahlo" />
                </NetworkImage::Root>
            }
        }
    };
    let deadline = Instant::now() + Duration::from_secs(20);
    let buffer = loop {
        let buffer = frame(&build);
        if has_pixels(&buffer) || Instant::now() > deadline {
            break buffer;
        }
        std::thread::sleep(Duration::from_millis(50));
    };
    assert!(has_pixels(&buffer), "did not load: {:?}", load_error(&url));
}
