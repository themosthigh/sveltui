use svtui::prelude::*;

fn main() -> std::io::Result<()> {
    let (count, set_count) = create_signal(0);

    App::new(move || {
        let val = count.get();

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
                    title=" Counter "
                    padding=1
                    width=58
                    align_items=Align::Center
                    justify_content=Justify::Center
                >
                    <Text color=Color::White bold=true>
                        { format!("Current Value: {}", val) }
                    </Text>

                    <Divider />

                    <Box
                        flex_direction=FlexDirection::Row
                        gap=1
                        align_items=Align::Center
                        justify_content=Justify::Center
                    >
                        <Button
                            variant=ButtonVariant::Secondary
                            on_click={move || set_count.update(|c| *c -= 1)}
                        >
                            "- Decrement"
                        </Button>
                        <Button
                            variant=ButtonVariant::Primary
                            on_click={move || set_count.update(|c| *c += 1)}
                        >
                            "+ Increment"
                        </Button>
                        <Button
                            variant=ButtonVariant::Outline
                            on_click={move || set_count.set(0)}
                        >
                            "Reset"
                        </Button>
                    </Box>

                    <Divider />

                    <Text color=Color::DarkGrey>
                        "Tab: Navigate | Space/Enter: Click | Esc: Exit"
                    </Text>
                </Box>
            </Box>
        }
    })
    .on_key(move |key| match key.code {
        crossterm::event::KeyCode::Char('+') | crossterm::event::KeyCode::Up => {
            set_count.update(|c| *c += 1);
        }
        crossterm::event::KeyCode::Char('-') | crossterm::event::KeyCode::Down => {
            set_count.update(|c| *c -= 1);
        }
        crossterm::event::KeyCode::Char('r') => {
            set_count.set(0);
        }
        _ => {}
    })
    .run()
}
