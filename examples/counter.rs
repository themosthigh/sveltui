use svtui::prelude::*;

fn main() -> Result<(), std::io::Error> {
    let (count, set_count) = create_signal(0);

    App::new(move || {
        let value = count.get();

        view! {
            // Outer container
            <Box
                width=Size::Fill
                height=Size::Fill
                align_items=Align::Center
                justify_content=Justify::Center
            >

                // Inner box
                <Box
                    padding=Padding::all(2)
                    border=BorderStyle::Double
                    border_color=Color::DarkGrey
                    align_items=Align::Center
                    justify_content=Justify::Center
                    flex_direction=FlexDirection::Row
                    gap=4
                >

                   // Decrement
                    <Button on_click={move || set_count.update(|val| *val -= 1)}>
                        "-"
                    </Button>

                    // Result
                    <Box width=14 align_items=Align::Center>
                        <Text>{format!("Counting {}", value)}</Text>
                    </Box>

                   // Increment
                    <Button on_click={move || set_count.update(|val| *val += 1)}>
                        "+"
                    </Button>
                </Box>
            </Box>
        }
    })
    .run()
}
