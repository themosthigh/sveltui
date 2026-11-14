use crate::image_render::{cell_size, runs, sample};
use crate::*;
use image::{Rgba, RgbaImage};
use std::io::Cursor;
use sveltui::element::*;
use sveltui::layout::Rect;
use sveltui::prelude::*;
use sveltui::terminal::Buffer;

const RED: [u8; 3] = [255, 0, 0];
const BLUE: [u8; 3] = [0, 0, 255];

/// Two pixels wide, two tall: red over blue.
fn red_over_blue() -> RgbaImage {
    RgbaImage::from_fn(2, 2, |_, y| {
        if y == 0 {
            Rgba([255, 0, 0, 255])
        } else {
            Rgba([0, 0, 255, 255])
        }
    })
}

fn png(img: &RgbaImage) -> ImageBytes {
    let mut out = Cursor::new(Vec::new());
    img.write_to(&mut out, image::ImageFormat::Png).unwrap();
    ImageBytes::new(out.into_inner())
}

#[test]
fn a_cell_holds_two_pixels_one_above_the_other() {
    let grid = sample(&red_over_blue(), 2, 1, ImageFit::Fill, [0; 3]);
    let rows = runs(&grid);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].len(), 1, "equal cells share a run");
    assert_eq!(rows[0][0].text, "▀▀");
    assert_eq!((rows[0][0].fg, rows[0][0].bg), (Some(RED), Some(BLUE)));
}

#[test]
fn sizes_follow_the_aspect_ratio_with_square_pixels() {
    assert_eq!(cell_size((40, 20), Some(20), None, (1, 2)), (20, 5));
    assert_eq!(cell_size((40, 20), None, Some(5), (1, 2)), (20, 5));
    assert_eq!(cell_size((40, 20), Some(7), Some(3), (1, 2)), (7, 3));
    // Without a size the width is capped at 60 cells.
    assert_eq!(cell_size((200, 100), None, None, (1, 2)), (60, 15));
    assert_eq!(cell_size((10, 10), None, None, (1, 2)), (10, 5));
}

#[test]
fn contain_centers_the_image_and_leaves_the_rest_empty() {
    // A 2x2 image in a 6x1 cell box (6x2 pixels) scales to 2x2 and sits in
    // the middle columns.
    let grid = sample(&red_over_blue(), 6, 1, ImageFit::Contain, [0; 3]);
    assert_eq!(grid[0][0], None);
    assert_eq!(grid[0][2], Some(RED));
    assert_eq!(grid[1][3], Some(BLUE));
    assert_eq!(grid[0][5], None);
}

#[test]
fn cover_fills_the_box_and_crops_the_overflow() {
    // A wide red strip covering a 2x2-cell box (2x4 pixels) is scaled up
    // until it fills the height and cropped to the width; contain would
    // leave the top and bottom empty.
    let strip = RgbaImage::from_pixel(8, 1, Rgba([255, 0, 0, 255]));
    let covered = sample(&strip, 2, 2, ImageFit::Cover, [0; 3]);
    assert!(covered.iter().flatten().all(|p| *p == Some(RED)));
    let contained = sample(&strip, 2, 2, ImageFit::Contain, [0; 3]);
    assert_eq!(contained[0][0], None);
}

#[test]
fn transparent_pixels_blend_into_the_background() {
    let img = RgbaImage::from_pixel(1, 2, Rgba([255, 0, 0, 0]));
    let grid = sample(&img, 1, 1, ImageFit::Fill, [10, 20, 30]);
    assert_eq!(grid[0][0], Some([10, 20, 30]));
    let half = RgbaImage::from_pixel(1, 2, Rgba([200, 0, 0, 128]));
    let grid = sample(&half, 1, 1, ImageFit::Fill, [0, 0, 100]);
    let px = grid[0][0].unwrap();
    assert!(
        (95..=105).contains(&px[0]) && (45..=55).contains(&px[2]),
        "{px:?}"
    );
}

fn draw(build: impl Fn() -> VNode, width: u16, height: u16) -> (Buffer, HitMap) {
    let manager = FocusManager::new();
    reset_focus_counter();
    begin_frame();
    let node = manager.with_context(build);
    end_frame();
    let mut hits = HitMap::new();
    let mut buffer = Buffer::new(width, height);
    manager.with_context(|| {
        render_node_interactive(
            &node,
            Rect::new(0, 0, width, height),
            &mut buffer,
            &mut hits,
        )
    });
    (buffer, hits)
}

#[test]
fn the_component_draws_half_blocks_in_color() {
    let bytes = png(&red_over_blue());
    let (buffer, hits) = draw(
        || view! { <Image bytes={bytes.clone()} alt="A flag" width=2 height=1 fit=ImageFit::Fill protocol=ImageProtocol::HalfBlock /> },
        10,
        3,
    );
    let cell = buffer.get(0, 0).unwrap();
    assert_eq!(cell.ch, '▀');
    assert_eq!(cell.fg, Some(Color::Rgb { r: 255, g: 0, b: 0 }));
    assert_eq!(cell.bg, Some(Color::Rgb { r: 0, g: 0, b: 255 }));
    assert_eq!(buffer.get(2, 0).unwrap().ch, ' ', "nothing past the width");
    let node = hits
        .interactions
        .nodes()
        .iter()
        .find(|n| n.interaction.semantics.role == Role::Image)
        .expect("image semantics");
    assert_eq!(node.interaction.semantics.label, "A flag");
}

#[test]
fn an_image_that_cannot_load_shows_its_alt_text() {
    let (buffer, _) = draw(
        || view! { <Image src="/no/such/file.png" alt="Missing logo" protocol=ImageProtocol::HalfBlock /> },
        20,
        2,
    );
    let row: String = (0..12).map(|x| buffer.get(x, 0).unwrap().ch).collect();
    assert_eq!(row, "Missing logo");
}

#[cfg(feature = "kitty")]
mod graphics {
    use super::*;

    /// Cell size is process-wide, and every test here sets the same one.
    fn with_cells() {
        sveltui::terminal::set_cell_pixels(8, 16);
    }

    #[test]
    fn a_graphics_protocol_places_one_picture_over_blank_cells() {
        with_cells();
        let bytes = png(&red_over_blue());
        let (buffer, _) = draw(
            || view! { <Image bytes={bytes.clone()} alt="A flag" width=4 height=2 protocol=ImageProtocol::Kitty /> },
            20,
            6,
        );
        assert_eq!(buffer.images.len(), 1);
        let image = &buffer.images[0];
        assert_eq!((image.at, image.size), ((0, 0), (4, 2)));
        assert_eq!(
            buffer.get(0, 0).unwrap().ch,
            ' ',
            "the cells hold no pixels"
        );
        let sent = image.paint.draw(image.size, image.visible, (8, 16));
        let sent = String::from_utf8(sent).unwrap();
        assert!(sent.starts_with("\x1b_Ga=T,f=32,s=32,v=32,"), "{sent}");
        assert!(sent.contains("c=4,r=2"));
        assert!(image.paint.erase().is_some());
    }

    #[test]
    fn only_the_visible_part_is_sent() {
        with_cells();
        let bytes = png(&red_over_blue());
        let (buffer, _) = draw(
            || view! { <Image bytes={bytes.clone()} alt="A flag" width=4 height=2 protocol=ImageProtocol::Kitty /> },
            20,
            6,
        );
        let image = &buffer.images[0];
        // Show only the right-hand two columns of the second row.
        let visible = Rect::new(2, 1, 2, 1);
        let sent = String::from_utf8(image.paint.draw(image.size, visible, (8, 16))).unwrap();
        assert!(sent.contains("s=16,v=16,"), "{sent}");
        assert!(sent.contains("c=2,r=1"), "{sent}");
    }

    #[test]
    fn half_blocks_are_used_when_the_cell_size_is_unknown() {
        sveltui::terminal::set_cell_pixels(0, 0);
        let bytes = png(&red_over_blue());
        let (buffer, _) = draw(
            || view! { <Image bytes={bytes.clone()} alt="A flag" width=2 height=1 protocol=ImageProtocol::Kitty /> },
            10,
            3,
        );
        assert!(buffer.images.is_empty());
        assert_eq!(buffer.get(0, 0).unwrap().ch, '▀');
        with_cells();
    }

    #[test]
    fn the_same_picture_keeps_its_key_across_frames() {
        with_cells();
        let bytes = png(&red_over_blue());
        let build = || {
            view! { <Image bytes={bytes.clone()} alt="A flag" width=4 height=2 protocol=ImageProtocol::Kitty /> }
        };
        let manager = FocusManager::new();
        let mut keys = Vec::new();
        for _ in 0..2 {
            reset_focus_counter();
            begin_frame();
            let node = manager.with_context(build);
            end_frame();
            let mut buffer = Buffer::new(20, 6);
            let mut hits = HitMap::new();
            manager.with_context(|| {
                render_node_interactive(&node, Rect::new(0, 0, 20, 6), &mut buffer, &mut hits)
            });
            keys.push(buffer.images[0].paint.key());
        }
        assert_eq!(keys[0], keys[1]);
    }
}
