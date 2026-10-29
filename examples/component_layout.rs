use svtui::prelude::*;
use svtui_components::*;

#[component]
fn LayoutDemo() -> impl IntoNode {
    let direction = state(FlexDirection::Row);
    let gap = state(2u16);

    view! {
        <FlexBox width=Size::Fill height=Size::Fill align_items=Align::Center justify_content=Justify::Center>
            <FlexBox
                width=76
                height=Size::Fill
                max_height=30
                border=BorderStyle::Double
                border_color=Color::DarkGrey
                padding=Padding::all(1)
                gap=1
            >
                <FlexBox flex_direction=FlexDirection::Row justify_content=Justify::SpaceBetween align_items=Align::Center>
                    <TextSpan color=Color::White bold=true>"component_layout: Flexbox Engine"</TextSpan>
                    <TextSpan color=Color::DarkGrey>{format!("Axis: {:?}", direction.get())}</TextSpan>
                </FlexBox>
                <Divider />

                <FlexBox flex_direction=FlexDirection::Row gap=1>
                    <Button
                        on_click={move || direction.set(FlexDirection::Row)}
                    >
                        "Row Direction"
                    </Button>
                    <Button
                        on_click={move || direction.set(FlexDirection::Column)}
                    >
                        "Column Direction"
                    </Button>
                    <Button
                        on_click={move || gap.update(|g| *g = if *g >= 4 { 0 } else { *g + 1 })}
                    >
                        {format!("Cycle Gap ({})", gap.get())}
                    </Button>
                </FlexBox>

                <FlexBox height=1 />

                // Interactive Flex Container
                <FlexBox
                    width=Size::Fill
                    height=Size::Fill
                    flex_direction=direction.get()
                    gap=gap.get()
                    border=BorderStyle::Single
                    border_color=Color::DarkGrey
                    padding=Padding::all(1)
                    overflow=Overflow::Auto
                >
                    <FlexBox
                        width=Size::Fill
                        height=Size::Fill
                        min_width=16
                        min_height=3
                        border=BorderStyle::Single
                        border_color=Color::White
                        padding=Padding::all(1)
                    >
                        <TextSpan color=Color::White bold=true>"Child A"</TextSpan>
                        <TextSpan color=Color::DarkGrey>"Flexible share"</TextSpan>
                    </FlexBox>

                    <FlexBox
                        width=Size::Fill
                        height=Size::Fill
                        min_width=16
                        min_height=3
                        border=BorderStyle::Single
                        border_color=Color::White
                        padding=Padding::all(1)
                    >
                        <TextSpan color=Color::White bold=true>"Child B"</TextSpan>
                        <TextSpan color=Color::DarkGrey>"Shares space"</TextSpan>
                    </FlexBox>

                    <FlexBox
                        width=Size::Fill
                        height=Size::Fill
                        min_width=16
                        min_height=3
                        border=BorderStyle::Single
                        border_color=Color::White
                        padding=Padding::all(1)
                    >
                        <TextSpan color=Color::White bold=true>"Child C"</TextSpan>
                        <TextSpan color=Color::DarkGrey>"Even flex split"</TextSpan>
                    </FlexBox>
                </FlexBox>

                <Divider />
                <TextSpan color=Color::DarkGrey>"Tab: Switch controls | Space/Enter: Click | Q/Esc: Quit"</TextSpan>
            </FlexBox>
        </FlexBox>
    }
}

fn main() -> std::io::Result<()> {
    App::new(|| view! { <LayoutDemo /> })
        .on_key(svtui::app::exit_shortcuts)
        .run()
}
