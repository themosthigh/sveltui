use ::image::{Rgba, RgbaImage};
use std::io::Cursor;
use sveltui::prelude::*;
use sveltui_image::*;

#[cfg(test)]
mod image_tests;

/// Frida Kahlo, by Guillermo Kahlo (cropped), via Wikimedia Commons.
const FRIDA_KAHLO: &str = "https://thumb.wikimedia.org/wikipedia/commons/thumb/0/09/Frida_Kahlo%2C_by_Guillermo_Kahlo_%28cropped%29.jpg/500px-Frida_Kahlo%2C_by_Guillermo_Kahlo_%28cropped%29.jpg";

/// A colour wheel with a transparent corner, drawn in code so the example
/// needs no files.
fn sample_image() -> ImageBytes {
    let size = 96u32;
    let img = RgbaImage::from_fn(size, size, |x, y| {
        let (dx, dy) = (x as f32 / size as f32 - 0.5, y as f32 / size as f32 - 0.5);
        let radius = (dx * dx + dy * dy).sqrt();
        if radius > 0.5 {
            return Rgba([0, 0, 0, 0]);
        }
        let hue = dy.atan2(dx).to_degrees().rem_euclid(360.0) / 60.0;
        let c = 1.0 - radius * 1.2;
        let x1 = c * (1.0 - (hue % 2.0 - 1.0).abs());
        let (r, g, b) = match hue as u32 {
            0 => (c, x1, 0.0),
            1 => (x1, c, 0.0),
            2 => (0.0, c, x1),
            3 => (0.0, x1, c),
            4 => (x1, 0.0, c),
            _ => (c, 0.0, x1),
        };
        Rgba([(r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8, 255])
    });
    let mut out = Cursor::new(Vec::new());
    img.write_to(&mut out, ::image::ImageFormat::Png)
        .expect("encode sample image");
    ImageBytes::new(out.into_inner())
}

#[component]
fn Gallery() -> impl IntoNode {
    // Pass a path as the first argument to show your own picture. Anything
    // that isn't a file (such as a test runner's flags) is ignored.
    let path = std::env::args()
        .nth(1)
        .filter(|path| std::path::Path::new(path).is_file());
    let bytes = use_hook(sample_image);
    // A portrait from Wikimedia Commons. Tests point at a closed port instead
    // so they never need the network.
    let avatar_url = std::env::var("SVELTUI_EXAMPLE_IMAGE_URL").unwrap_or_else(|_| {
        String::from(if cfg!(test) {
            "http://127.0.0.1:9/avatar.png"
        } else {
            FRIDA_KAHLO
        })
    });

    view! {
        <View padding=1 gap=1>
            <TextSpan foreground_color=Color::White bold=true>"Image"</TextSpan>
            <View flex_direction=FlexDirection::Row gap=2>
                {#each [ImageFit::Contain, ImageFit::Cover, ImageFit::Fill] as fit}
                    <View gap=1>
                        <TextSpan foreground_color=Color::DarkGrey>{format!("{fit:?}")}</TextSpan>
                        {#if let Some(path) = path.clone()}
                            <Image src=path alt="Your picture" width=24 height=8 fit=fit />
                        {:else}
                            <Image bytes={bytes.clone()} alt="A colour wheel" width=24 height=8 fit=fit />
                        {/if}
                    </View>
                {/each}
            </View>
            <TextSpan foreground_color=Color::DarkGrey>
                {format!("Drawing with {:?}", ImageProtocol::Auto.resolve())}
            </TextSpan>

            // Sources are tried in order; the fallback shows while none has
            // loaded. Set SVELTUI_EXAMPLE_IMAGE_URL to load a different one.
            <View flex_direction=FlexDirection::Row gap=2>
                <NetworkImage::Root width=24 height=12 fit=ImageFit::Cover>
                    <NetworkImage::Image src={avatar_url.clone()} alt="Frida Kahlo" />
                    <NetworkImage::Fallback delay_ms=200>
                        <TextSpan foreground_color=Color::DarkGrey>"FK"</TextSpan>
                    </NetworkImage::Fallback>
                </NetworkImage::Root>
                <TextSpan foreground_color=Color::DarkGrey>
                    "Frida Kahlo, by Guillermo Kahlo (Wikimedia Commons). Loads in the background; falls back to initials"
                </TextSpan>
            </View>
            <TextSpan foreground_color=Color::DarkGrey>"Q/Esc: quit"</TextSpan>
        </View>
    }
}

fn main() -> std::io::Result<()> {
    App::new(|| view! { <Gallery /> })
        .on_key(sveltui::app::exit_shortcuts)
        .run()
}
