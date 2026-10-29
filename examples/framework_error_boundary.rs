use svtui::prelude::*;
use svtui_components::*;

#[derive(Props)]
struct FlakyWidgetProps {
    should_crash: State<bool>,
}

#[component]
fn FlakyWidget(props: FlakyWidgetProps) -> impl IntoNode {
    if props.should_crash.get() {
        panic!("FlakyWidget encountered an intentional unhandled division by zero!");
    }

    view! {
        <Box border=BorderStyle::Single border_color=Color::White padding=Padding::all(1) gap=1>
            <Text color=Color::White bold=true>"Child Widget: Running Healthy"</Text>
            <Text color=Color::DarkGrey>"Calculations proceeding normally. Click 'Trigger Panic' below to test isolation."</Text>
        </Box>
    }
}

#[component]
fn ErrorBoundaryDemo() -> impl IntoNode {
    let should_crash = state(false);

    view! {
        <Box width=Size::Fill height=Size::Fill align_items=Align::Center justify_content=Justify::Center>
            <Box
                width=72
                border=BorderStyle::Double
                border_color=Color::DarkGrey
                padding=Padding::all(1)
                gap=1
            >
                <Text color=Color::White bold=true>"framework_error_boundary: Crash Isolation & Recovery"</Text>
                <Divider />

                <Box flex_direction=FlexDirection::Row gap=1 align_items=Align::Center>
                    <Button
                        on_click={move || should_crash.set(true)}
                    >
                        "Trigger Panic"
                    </Button>
                    <Button
                        on_click={move || should_crash.set(false)}
                    >
                        "Recover Normal State"
                    </Button>
                </Box>

                <Box height=1 />

                // Error boundary isolates panics without crashing the entire terminal app
                <ErrorBoundary>
                    {#snippet fallback(error)}
                        <Box
                            border=BorderStyle::Double
                            border_color=Color::White
                            padding=Padding::all(1)
                            gap=1
                        >
                            <Text color=Color::White bold=true>"[ErrorBoundary Caught Failure]"</Text>
                            <Text color=Color::DarkGrey>{format!("Panic payload: {error}")}</Text>
                            <Text color=Color::DarkGrey>"Outer application remains fully interactive. Click 'Recover' above."</Text>
                        </Box>
                    {/snippet}

                    <FlakyWidget should_crash=should_crash />
                </ErrorBoundary>
            </Box>
        </Box>
    }
}

fn main() -> std::io::Result<()> {
    App::new(|| view! { <ErrorBoundaryDemo /> })
        .on_key(svtui::app::exit_shortcuts)
        .run()
}
