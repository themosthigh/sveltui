use svtui::prelude::*;

#[component]
fn StateDemo() -> impl IntoNode {
    let count = state(0);
    let name = state(String::from("Explorer"));
    let debug_mode = state(false);
    let doubled = derived(move || count.get() * 2);

    view! {
        <Box width=Size::Fill height=Size::Fill align_items=Align::Center justify_content=Justify::Center>
            <Box
                width=64
                border=BorderStyle::Double
                border_color=Color::DarkGrey
                padding=Padding::all(1)
                gap=1
            >
                <Text color=Color::White bold=true>"framework_state: Reactivity & Runes"</Text>
                <Divider />

                <Box flex_direction=FlexDirection::Row gap=2 align_items=Align::Center>
                    <Text color=Color::DarkGrey>"Name:"</Text>
                    <TextInput value=name placeholder="Type name..." />
                </Box>

                <Box flex_direction=FlexDirection::Row gap=2 align_items=Align::Center>
                    <Switch checked=debug_mode label="Debug metrics" />
                </Box>

                <Divider />

                <Box gap=1>
                    <Text>{format!("Hello, {}! Current count: {}", name.get(), count.get())}</Text>
                    <Text color=Color::DarkGrey>{format!("Derived (count * 2): {}", doubled.get())}</Text>
                    {#if debug_mode.get()}
                        <Text color=Color::DarkGrey>{format!("[Debug] signal_id: {:?}, raw_len: {}", count.id(), name.get().len())}</Text>
                    {/if}
                </Box>

                <Box height=1 />

                <Box flex_direction=FlexDirection::Row gap=1>
                    <Button variant=ButtonVariant::Primary on_click={move || count.update(|v| *v += 1)}>
                        "+ Increment"
                    </Button>
                    <Button variant=ButtonVariant::Secondary on_click={move || count.update(|v| *v -= 1)}>
                        "- Decrement"
                    </Button>
                    <Button variant=ButtonVariant::Outline on_click={move || count.set(0)}>
                        "Reset"
                    </Button>
                    <Button variant=ButtonVariant::Secondary on_click={move || {
                        batch(|| {
                            count.set(100);
                            name.set(String::from("Batched"));
                        });
                    }}>
                        "Batch Update"
                    </Button>
                </Box>
            </Box>
        </Box>
    }
}

fn main() -> std::io::Result<()> {
    App::new(|| view! { <StateDemo /> })
        .on_key(svtui::app::exit_shortcuts)
        .run()
}
