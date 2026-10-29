use std::io::Result;
use sveltui::app::{App, exit_shortcuts};
use sveltui::prelude::*;

fn main() -> Result<()> {
    App::new(|| {
        view! {
            <TextSpan>"Hello, world!"</TextSpan>
        }
    })
    .on_key(exit_shortcuts)
    .run()
}
