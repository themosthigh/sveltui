use std::io::Result;
use svtui::app::{App, exit_shortcuts};
use svtui::prelude::*;

fn main() -> Result<()> {
    App::new(|| {
        view! {
            <Text>"Hello, world!"</Text>
        }
    })
    .on_key(exit_shortcuts)
    .run()
}
