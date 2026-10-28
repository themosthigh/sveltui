use svtui::prelude::*;

#[component]
fn InteractiveShowcase() -> impl IntoNode {
    let progress = state(45f32);
    let username = state(String::from("Ada Lovelace"));
    let notifications = state(true);
    let auto_save = state(false);
    let status_tag = state(String::from("v0.1.0"));

    let advance_progress = move || {
        progress.update(|p| {
            *p = if *p >= 100.0 { 0.0 } else { *p + 15.0 };
        });
    };

    view! {
        <Box width=Size::Fill height=Size::Fill align_items=Align::Center justify_content=Justify::Center>
            <Box
                width=72
                border=BorderStyle::Double
                border_color=Color::DarkGrey
                padding=Padding::all(1)
                gap=1
            >
                <Box flex_direction=FlexDirection::Row justify_content=Justify::SpaceBetween align_items=Align::Center>
                    <Text color=Color::White bold=true>"component_interactive: Full Widgets Dashboard"</Text>
                    <Badge label=status_tag.get() />
                </Box>
                <Divider />

                <Text color=Color::DarkGrey>"Progress Bar Component:"</Text>
                <Box flex_direction=FlexDirection::Row gap=2 align_items=Align::Center>
                    <Box width=48>
                        <ProgressBar ratio={progress.get() / 100.0} />
                    </Box>
                    <Text color=Color::White>{format!("{}%", progress.get() as u32)}</Text>
                    <Button variant=ButtonVariant::Outline on_click={advance_progress}>
                        "+15%"
                    </Button>
                </Box>

                <Divider />

                <Text color=Color::DarkGrey>"Form & Input Controls:"</Text>
                <Box flex_direction=FlexDirection::Row gap=2 align_items=Align::Center>
                    <Text color=Color::DarkGrey>"User:"</Text>
                    <TextInput value=username placeholder="Enter username..." width=32 />
                </Box>

                <Box flex_direction=FlexDirection::Row gap=4>
                    <Checkbox checked=notifications label="Desktop Notifications" />
                    <Switch checked=auto_save label="Auto-save to disk" />
                </Box>

                <Divider />

                <Box flex_direction=FlexDirection::Row justify_content=Justify::SpaceBetween align_items=Align::Center>
                    <Text color=Color::DarkGrey>"Tab: Navigate | Space/Enter: Toggle | Esc: Quit"</Text>
                    <Button
                        variant=ButtonVariant::Primary
                        on_click={move || status_tag.set(String::from("v0.2.0-rc"))}
                    >
                        "Tag Release"
                    </Button>
                </Box>
            </Box>
        </Box>
    }
}

fn main() -> std::io::Result<()> {
    App::new(|| view! { <InteractiveShowcase /> })
        .on_key(svtui::app::exit_shortcuts)
        .run()
}
