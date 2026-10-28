use svtui::prelude::*;

#[component]
fn LayoutDemo() -> impl IntoNode {
    let direction = state(FlexDirection::Row);
    let gap = state(2u16);

    let is_row = direction.get() == FlexDirection::Row;

    view! {
        <Box width=Size::Fill height=Size::Fill align_items=Align::Center justify_content=Justify::Center>
            <Box
                width=76
                height=Size::Fill
                max_height=30
                border=BorderStyle::Double
                border_color=Color::DarkGrey
                padding=Padding::all(1)
                gap=1
            >
                <Box flex_direction=FlexDirection::Row justify_content=Justify::SpaceBetween align_items=Align::Center>
                    <Text color=Color::White bold=true>"component_layout: Flexbox Engine"</Text>
                    <Text color=Color::DarkGrey>{format!("Axis: {:?}", direction.get())}</Text>
                </Box>
                <Divider />

                <Box flex_direction=FlexDirection::Row gap=1>
                    <Button
                        variant=if is_row { ButtonVariant::Primary } else { ButtonVariant::Outline }
                        on_click={move || direction.set(FlexDirection::Row)}
                    >
                        "Row Direction"
                    </Button>
                    <Button
                        variant=if !is_row { ButtonVariant::Primary } else { ButtonVariant::Outline }
                        on_click={move || direction.set(FlexDirection::Column)}
                    >
                        "Column Direction"
                    </Button>
                    <Button
                        variant=ButtonVariant::Secondary
                        on_click={move || gap.update(|g| *g = if *g >= 4 { 0 } else { *g + 1 })}
                    >
                        {format!("Cycle Gap ({})", gap.get())}
                    </Button>
                </Box>

                <Box height=1 />

                // Interactive Flex Container
                <Box
                    width=Size::Fill
                    height=Size::Fill
                    flex_direction=direction.get()
                    gap=gap.get()
                    border=BorderStyle::Single
                    border_color=Color::DarkGrey
                    padding=Padding::all(1)
                    overflow=Overflow::Auto
                >
                    <Box
                        width=Size::Fill
                        height=Size::Fill
                        min_width=16
                        min_height=3
                        border=BorderStyle::Single
                        border_color=Color::White
                        padding=Padding::all(1)
                    >
                        <Text color=Color::White bold=true>"Child A"</Text>
                        <Text color=Color::DarkGrey>"Flexible share"</Text>
                    </Box>

                    <Box
                        width=Size::Fill
                        height=Size::Fill
                        min_width=16
                        min_height=3
                        border=BorderStyle::Single
                        border_color=Color::White
                        padding=Padding::all(1)
                    >
                        <Text color=Color::White bold=true>"Child B"</Text>
                        <Text color=Color::DarkGrey>"Shares space"</Text>
                    </Box>

                    <Box
                        width=Size::Fill
                        height=Size::Fill
                        min_width=16
                        min_height=3
                        border=BorderStyle::Single
                        border_color=Color::White
                        padding=Padding::all(1)
                    >
                        <Text color=Color::White bold=true>"Child C"</Text>
                        <Text color=Color::DarkGrey>"Even flex split"</Text>
                    </Box>
                </Box>

                <Divider />
                <Text color=Color::DarkGrey>"Tab: Switch controls | Space/Enter: Click | Q/Esc: Quit"</Text>
            </Box>
        </Box>
    }
}

fn main() -> std::io::Result<()> {
    App::new(|| view! { <LayoutDemo /> })
        .on_key(svtui::app::exit_shortcuts)
        .run()
}
