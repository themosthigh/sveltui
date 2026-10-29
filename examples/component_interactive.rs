use sveltui::prelude::*;
use sveltui_components::*;

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
        <FlexBox width=Size::Fill height=Size::Fill align_items=Align::Center justify_content=Justify::Center>
            <FlexBox
                width=72
                border=BorderStyle::Double
                border_color=Color::DarkGrey
                padding=Padding::all(1)
                gap=1
            >
                <FlexBox flex_direction=FlexDirection::Row justify_content=Justify::SpaceBetween align_items=Align::Center>
                    <TextSpan color=Color::White bold=true>"component_interactive: Full Widgets Dashboard"</TextSpan>
                    <Badge label=status_tag.get() />
                </FlexBox>
                <Divider />

                <TextSpan color=Color::DarkGrey>"Progress Bar Component:"</TextSpan>
                <FlexBox flex_direction=FlexDirection::Row gap=2 align_items=Align::Center>
                    <FlexBox width=48>
                        <ProgressBar ratio={progress.get() / 100.0} />
                    </FlexBox>
                    <TextSpan color=Color::White>{format!("{}%", progress.get() as u32)}</TextSpan>
                    <Button on_click={advance_progress}>
                        "+15%"
                    </Button>
                </FlexBox>

                <Divider />

                <TextSpan color=Color::DarkGrey>"Form & Input Controls:"</TextSpan>
                <FlexBox flex_direction=FlexDirection::Row gap=2 align_items=Align::Center>
                    <TextSpan color=Color::DarkGrey>"User:"</TextSpan>
                    <TextInput value=username placeholder="Enter username..." width=32 />
                </FlexBox>

                <FlexBox flex_direction=FlexDirection::Row gap=4>
                    <Checkbox checked=notifications label="Desktop Notifications" />
                    <Switch checked=auto_save label="Auto-save to disk" />
                </FlexBox>

                <Divider />

                <FlexBox flex_direction=FlexDirection::Row justify_content=Justify::SpaceBetween align_items=Align::Center>
                    <TextSpan color=Color::DarkGrey>"Tab: Navigate | Space/Enter: Toggle | Esc: Quit"</TextSpan>
                    <Button
                        on_click={move || status_tag.set(String::from("v0.2.0-rc"))}
                    >
                        "Tag Release"
                    </Button>
                </FlexBox>
            </FlexBox>
        </FlexBox>
    }
}

fn main() -> std::io::Result<()> {
    App::new(|| view! { <InteractiveShowcase /> })
        .on_key(sveltui::app::exit_shortcuts)
        .run()
}
