use sveltui::prelude::*;
use sveltui_components::*;

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
        <View border={theme.border_style} border_color={theme.primary_color} padding={Padding::all(1)} gap=1>
            <View flex_direction=FlexDirection::Row justify_content=JustifyContent::SpaceBetween>
                <TextSpan foreground_color={theme.primary_color} bold=true>"Themed Consumer Box"</TextSpan>
                <Badge label={theme.badge_label} />
            </View>
            <TextSpan foreground_color=Color::DarkGrey>"Reads theme values injected by ancestor <ContextProvider value=...>"</TextSpan>
        </View>
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
        <ContextProvider value={current_theme.get()}>
            <View width={Dimension::Percent(1.0)} height={Dimension::Percent(1.0)} align_items=AlignItems::Center justify_content=JustifyContent::Center>
                <View
                    width=66
                    border=BorderStyle::Single
                    border_color=Color::DarkGrey
                    padding={Padding::all(1)}
                    gap=1
                >
                    <TextSpan foreground_color=Color::White bold=true>"framework_context: Context API with <ContextProvider>"</TextSpan>
                    <Divider />

                    <View flex_direction=FlexDirection::Row justify_content=JustifyContent::SpaceBetween align_items=AlignItems::Center>
                        <Switch bind:checked=high_contrast label="Enable High-Contrast Theme" />
                    </View>

                    <Divider />

                    <DeepThemedWidget />

                    // Local Override with a nested Provider
                    <ContextProvider value={Theme {
                        primary_color: Color::White,
                        border_style: BorderStyle::Double,
                        badge_label: "Local Override",
                    }}>
                        <View padding={Padding::symmetric(0, 1)}>
                            <TextSpan foreground_color=Color::DarkGrey>"Below is an isolated sub-tree with an overridden Theme Provider:"</TextSpan>
                            <View height=1 />
                            <DeepThemedWidget />
                        </View>
                    </ContextProvider>
                </View>
            </View>
        </ContextProvider>
    }
}

fn main() -> std::io::Result<()> {
    App::new(|| view! { <ContextDemo /> })
        .on_key(sveltui::app::exit_shortcuts)
        .run()
}
