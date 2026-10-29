use sveltui::prelude::*;
use sveltui_components::*;

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
        <FlexBox border=BorderStyle::Single border_color=Color::White padding=Padding::all(1) gap=1>
            <TextSpan color=Color::White bold=true>"Child Widget: Running Healthy"</TextSpan>
            <TextSpan color=Color::DarkGrey>"Calculations proceeding normally. Click 'Trigger Panic' below to test isolation."</TextSpan>
        </FlexBox>
    }
}

#[component]
fn ErrorBoundaryDemo() -> impl IntoNode {
    let should_crash = state(false);

    view! {
        <FlexBox width=Size::Fill height=Size::Fill align_items=Align::Center justify_content=Justify::Center>
            <FlexBox
                width=72
                border=BorderStyle::Double
                border_color=Color::DarkGrey
                padding=Padding::all(1)
                gap=1
            >
                <TextSpan color=Color::White bold=true>"framework_error_boundary: Crash Isolation & Recovery"</TextSpan>
                <Divider />

                <FlexBox flex_direction=FlexDirection::Row gap=1 align_items=Align::Center>
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
                </FlexBox>

                <FlexBox height=1 />

                // Error boundary isolates panics without crashing the entire terminal app
                <ErrorBoundary>
                    {#snippet fallback(error)}
                        <FlexBox
                            border=BorderStyle::Double
                            border_color=Color::White
                            padding=Padding::all(1)
                            gap=1
                        >
                            <TextSpan color=Color::White bold=true>"[ErrorBoundary Caught Failure]"</TextSpan>
                            <TextSpan color=Color::DarkGrey>{format!("Panic payload: {error}")}</TextSpan>
                            <TextSpan color=Color::DarkGrey>"Outer application remains fully interactive. Click 'Recover' above."</TextSpan>
                        </FlexBox>
                    {/snippet}

                    <FlakyWidget should_crash=should_crash />
                </ErrorBoundary>
            </FlexBox>
        </FlexBox>
    }
}

fn main() -> std::io::Result<()> {
    App::new(|| view! { <ErrorBoundaryDemo /> })
        .on_key(sveltui::app::exit_shortcuts)
        .run()
}
