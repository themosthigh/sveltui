use sveltui::prelude::*;
use sveltui_components::*;

#[cfg(test)]
mod scroll_tests;

#[component]
fn ScrollDemo() -> impl IntoNode {
    let lines_count = state(20usize);
    let scroll_id = use_hook(FocusId::new);

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
        <View width={Dimension::Percent(1.0)} height={Dimension::Percent(1.0)} align_items=AlignItems::Center justify_content=JustifyContent::Center>
            <View
                width=68
                border=BorderStyle::Single
                border_color=Color::DarkGrey
                padding={Padding::all(1)}
                gap=1
            >
                <TextSpan foreground_color=Color::White bold=true>"component_scroll: Virtual Scrolling & Overflow"</TextSpan>
                <Divider />

                <TextSpan foreground_color=Color::DarkGrey>
                    "Mouse Wheel / Trackpad / Shift+Wheel or Arrow Keys scroll both axes seamlessly."
                </TextSpan>

                <View flex_direction=FlexDirection::Row gap=1 align_items=AlignItems::Center>
                    <Button
                        on_click={move || lines_count.update(|c| *c += 5)}
                    >
                        "+ Add 5 Lines"
                    </Button>
                    <Button
                        on_click={move || lines_count.set(10)}
                    >
                        "Reset Lines"
                    </Button>
                    <TextSpan foreground_color=Color::DarkGrey>{format!("Current log lines: {}", lines_count.get())}</TextSpan>
                </View>

                <View height=1 />

                // Scroll container with custom scrollbars
                <Pressable focus_id=scroll_id tab_stop=true>
                    <View
                        scroll_id=scroll_id
                        border=border_style
                        border_color={if focus.is_active() { Color::White } else { Color::DarkGrey }}
                        width=62
                        height=10
                        padding={Padding::symmetric(1, 0)}
                        overflow=Overflow::Scroll
                        scrollbar_visibility=ScrollbarVisibility::Auto
                    >
                        {#each log_entries as entry}
                            <TextSpan foreground_color=Color::White>{entry}</TextSpan>
                        {/each}
                    </View>
                </Pressable>

                <View height=1 />
                <Divider />

                <TextSpan foreground_color=Color::DarkGrey>"Tab: Focus scroll container | Wheel / Shift+Wheel: Scroll axes | Q/Esc: Quit"</TextSpan>
            </View>
        </View>
    }
}

fn main() -> std::io::Result<()> {
    App::new(|| view! { <ScrollDemo /> })
        .on_key(sveltui::app::exit_shortcuts)
        .run()
}
