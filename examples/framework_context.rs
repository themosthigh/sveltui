use svtui::prelude::*;

#[derive(Clone, Copy)]
struct Theme {
    primary_color: Color,
    border_style: BorderStyle,
    badge_label: &'static str,
}

#[component]
fn DeepThemedWidget() -> impl IntoNode {
    let theme = use_context::<Theme>().unwrap_or(Theme {
        primary_color: Color::White,
        border_style: BorderStyle::Single,
        badge_label: "Fallback",
    });

    view! {
        <Box border=theme.border_style border_color=theme.primary_color padding=Padding::all(1) gap=1>
            <Box flex_direction=FlexDirection::Row justify_content=Justify::SpaceBetween>
                <Text color=theme.primary_color bold=true>"Themed Consumer Box"</Text>
                <Badge label=theme.badge_label />
            </Box>
            <Text color=Color::DarkGrey>"Reads theme values injected by ancestor <Provider value=...>"</Text>
        </Box>
    }
}

#[component]
fn ContextDemo() -> impl IntoNode {
    let high_contrast = state(false);

    let current_theme = derived(move || {
        if high_contrast.get() {
            Theme {
                primary_color: Color::White,
                border_style: BorderStyle::Double,
                badge_label: "High Contrast",
            }
        } else {
            Theme {
                primary_color: Color::DarkGrey,
                border_style: BorderStyle::Single,
                badge_label: "Subtle Neutral",
            }
        }
    });

    view! {
        <Provider value=current_theme.get()>
            <Box width=Size::Fill height=Size::Fill align_items=Align::Center justify_content=Justify::Center>
                <Box
                    width=66
                    border=BorderStyle::Single
                    border_color=Color::DarkGrey
                    padding=Padding::all(1)
                    gap=1
                >
                    <Text color=Color::White bold=true>"framework_context: Context API with <Provider>"</Text>
                    <Divider />

                    <Box flex_direction=FlexDirection::Row justify_content=Justify::SpaceBetween align_items=Align::Center>
                        <Switch checked=high_contrast label="Enable High-Contrast Theme" />
                    </Box>

                    <Divider />

                    <DeepThemedWidget />

                    // Local Override with a nested Provider
                    <Provider value=Theme {
                        primary_color: Color::White,
                        border_style: BorderStyle::Double,
                        badge_label: "Local Override",
                    }>
                        <Box padding=Padding::symmetric(0, 1)>
                            <Text color=Color::DarkGrey>"Below is an isolated sub-tree with an overridden Theme Provider:"</Text>
                            <Box height=1 />
                            <DeepThemedWidget />
                        </Box>
                    </Provider>
                </Box>
            </Box>
        </Provider>
    }
}

fn main() -> std::io::Result<()> {
    App::new(|| view! { <ContextDemo /> })
        .on_key(svtui::app::exit_shortcuts)
        .run()
}
