use svtui::prelude::*;

#[component]
fn ButtonShowcase() -> impl IntoNode {
    let last_clicked = state(String::from("None"));
    let clicks_count = state(0);
    let disable_buttons = state(false);

    let click = |name: &'static str| {
        move || {
            last_clicked.set(String::from(name));
            clicks_count.update(|c| *c += 1);
        }
    };

    view! {
        <Box width=Size::Fill height=Size::Fill align_items=Align::Center justify_content=Justify::Center>
            <Box
                width=68
                border=BorderStyle::Double
                border_color=Color::DarkGrey
                padding=Padding::all(1)
                gap=1
            >
                <Text color=Color::White bold=true>"component_button: Variants & Focus Navigation"</Text>
                <Divider />

                <Text color=Color::DarkGrey>"Standard Button Variants:"</Text>
                <Box flex_direction=FlexDirection::Row gap=1>
                    <Button
                        variant=ButtonVariant::Primary
                        disabled=disable_buttons.get()
                        on_click={click("Primary Action")}
                    >
                        "Primary"
                    </Button>
                    <Button
                        variant=ButtonVariant::Secondary
                        disabled=disable_buttons.get()
                        on_click={click("Secondary Action")}
                    >
                        "Secondary"
                    </Button>
                    <Button
                        variant=ButtonVariant::Outline
                        disabled=disable_buttons.get()
                        on_click={click("Outline Action")}
                    >
                        "Outline"
                    </Button>
                    <Button
                        variant=ButtonVariant::Success
                        disabled=disable_buttons.get()
                        on_click={click("Success Action")}
                    >
                        "Success"
                    </Button>
                    <Button
                        variant=ButtonVariant::Danger
                        disabled=disable_buttons.get()
                        on_click={click("Danger Action")}
                    >
                        "Danger"
                    </Button>
                </Box>

                <Box height=1 />
                <Divider />

                <Box flex_direction=FlexDirection::Row justify_content=Justify::SpaceBetween align_items=Align::Center>
                    <Switch checked=disable_buttons label="Disable all buttons" />
                    <Button variant=ButtonVariant::Outline on_click={move || clicks_count.set(0)}>
                        "Reset Counter"
                    </Button>
                </Box>

                <Box height=1 />

                <Box border=BorderStyle::Single border_color=Color::DarkGrey padding=Padding::all(1)>
                    <Text>{format!("Last button triggered: {}", last_clicked.get())}</Text>
                    <Text color=Color::DarkGrey>{format!("Total click count: {}", clicks_count.get())}</Text>
                    <Text color=Color::DarkGrey>"Tab: Navigate | Space/Enter: Click | Esc: Exit"</Text>
                </Box>
            </Box>
        </Box>
    }
}

fn main() -> std::io::Result<()> {
    App::new(|| view! { <ButtonShowcase /> })
        .on_key(svtui::app::exit_shortcuts)
        .run()
}
