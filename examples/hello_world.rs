use svtui::{app::exit_shortcuts, prelude::*};

fn main() -> std::io::Result<()> {
    App::new(|| {
        view! {
            <Box
                width=Size::Fill
                height=Size::Fill
                align_items=Align::End
                justify_content=Justify::Center
            >
                <Text>"Hello, world!"</Text>
            </Box>
        }
    })
    .on_key(exit_shortcuts)
    .run()
}
