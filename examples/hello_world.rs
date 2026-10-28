use svtui::prelude::*;

fn main() -> std::io::Result<()> {
    App::new(|| {
        view! {
            <Box
                width=Size::Fill
                height=Size::Fill
                align_items=Align::Center
                justify_content=Justify::Center
            >
                <Text>"Hello, world!"</Text>
            </Box>
        }
    })
    .run()
}
