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
        <View width={Dimension::Percent(1.0)} height={Dimension::Percent(1.0)} align_items=AlignItems::Center justify_content=JustifyContent::Center>
            <View
                width=72
                border=BorderStyle::Double
                border_color=Color::DarkGrey
                padding={Padding::all(1)}
                gap=1
            >
                <View flex_direction=FlexDirection::Row justify_content=JustifyContent::SpaceBetween align_items=AlignItems::Center>
                    <TextSpan foreground_color=Color::White bold=true>"component_interactive: Full Widgets Dashboard"</TextSpan>
                    <Badge label={status_tag.get()} />
                </View>
                <Divider />

                <TextSpan foreground_color=Color::DarkGrey>"Progress Bar Component:"</TextSpan>
                <View flex_direction=FlexDirection::Row gap=2 align_items=AlignItems::Center>
                    <View width=48>
                        <ProgressBar ratio={progress.get() / 100.0} />
                    </View>
                    <TextSpan foreground_color=Color::White>{format!("{}%", progress.get() as u32)}</TextSpan>
                    <Button on_click={advance_progress}>
                        "+15%"
                    </Button>
                </View>

                <Divider />

                <TextSpan foreground_color=Color::DarkGrey>"Form & Input Controls:"</TextSpan>
                <View flex_direction=FlexDirection::Row gap=2 align_items=AlignItems::Center>
                    <TextSpan foreground_color=Color::DarkGrey>"User:"</TextSpan>
                    <TextInput bind:value=username placeholder="Enter username..." width=32 />
                </View>

                <View flex_direction=FlexDirection::Row gap=4>
                    <Checkbox bind:checked=notifications label="Desktop Notifications" />
                    <Switch bind:checked=auto_save label="Auto-save to disk" />
                </View>

                <Divider />

                <View flex_direction=FlexDirection::Row justify_content=JustifyContent::SpaceBetween align_items=AlignItems::Center>
                    <TextSpan foreground_color=Color::DarkGrey>"Tab: Navigate | Space/Enter: Toggle | Esc: Quit"</TextSpan>
                    <Button
                        on_click={move || status_tag.set(String::from("v0.2.0-rc"))}
                    >
                        "Tag Release"
                    </Button>
                </View>
            </View>
        </View>
    }
}

fn main() -> std::io::Result<()> {
    App::new(|| view! { <InteractiveShowcase /> })
        .on_key(sveltui::app::exit_shortcuts)
        .run()
}
