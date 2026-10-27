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
                <Box
                    border=BorderStyle::Single
                    border_color=Color::White
                    title=" Hello "
                    padding=1
                    align_items=Align::Center
                    justify_content=Justify::Center
                >
                    <Text color=Color::White bold=true>
                        "Hello, world!"
                    </Text>
                    <Text color=Color::DarkGrey>
                        "Press 'q' or Esc to exit"
                    </Text>
                </Box>
            </Box>
        }
    })
    .run()
}
