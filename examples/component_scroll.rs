use svtui::prelude::*;

#[component]
fn ScrollDemo() -> impl IntoNode {
    let lines_count = state(20usize);
    let scroll_id = FocusId::new();

    let focus = use_focus(scroll_id);
    let border_style = if focus.is_active() {
        BorderStyle::Double
    } else {
        BorderStyle::Single
    };

    let log_entries = (1..=lines_count.get())
        .map(|i| {
            format!(
                "[{:02}:{:02}] Log stream packet #{} — Double-buffered scroll pipeline running at 60 FPS across terminal cells.",
                i / 60,
                i % 60,
                i
            )
        })
        .collect::<Vec<_>>();

    view! {
        <Box width=Size::Fill height=Size::Fill align_items=Align::Center justify_content=Justify::Center>
            <Box
                width=68
                border=BorderStyle::Single
                border_color=Color::DarkGrey
                padding=Padding::all(1)
                gap=1
            >
                <Text color=Color::White bold=true>"component_scroll: Virtual Scrolling & Overflow"</Text>
                <Divider />

                <Text color=Color::DarkGrey>
                    "Mouse Wheel / Trackpad / Shift+Wheel or Arrow Keys scroll both axes seamlessly."
                </Text>

                <Box flex_direction=FlexDirection::Row gap=1 align_items=Align::Center>
                    <Button
                        variant=ButtonVariant::Primary
                        on_click={move || lines_count.update(|c| *c += 5)}
                    >
                        "+ Add 5 Lines"
                    </Button>
                    <Button
                        variant=ButtonVariant::Secondary
                        on_click={move || lines_count.set(10)}
                    >
                        "Reset Lines"
                    </Button>
                    <Text color=Color::DarkGrey>{format!("Current log lines: {}", lines_count.get())}</Text>
                </Box>

                <Box height=1 />

                // Scroll container with custom scrollbars
                <Box
                    border=border_style
                    border_color=if focus.is_active() { Color::White } else { Color::DarkGrey }
                    focus_id=scroll_id
                    tab_stop=true
                    width=62
                    height=10
                    padding=Padding::symmetric(1, 0)
                    overflow=Overflow::Scroll
                    scrollbar_visibility=ScrollbarVisibility::Auto
                >
                    {#each log_entries as entry}
                        <Text color=Color::White>{entry}</Text>
                    {/each}
                </Box>

                <Box height=1 />
                <Divider />

                <Text color=Color::DarkGrey>"Tab: Focus scroll container | Wheel / Shift+Wheel: Scroll axes | Q/Esc: Quit"</Text>
            </Box>
        </Box>
    }
}

fn main() -> std::io::Result<()> {
    App::new(|| view! { <ScrollDemo /> })
        .on_key(svtui::app::exit_shortcuts)
        .run()
}
