use sveltui::prelude::*;
use sveltui_components::*;

#[component]
fn ButtonShowcase() -> impl IntoNode {
    let last_clicked = state(String::from("None"));
    let clicks_count = state(0);
    let disable_buttons = state(false);

    let click = |name: &'static str| {
        move || {
            last_clicked.set(String::from(name));
            clicks_count.update(|c| *c += 1);
        }
    };

    view! {
        <FlexBox width=Size::Fill height=Size::Fill align_items=Align::Center justify_content=Justify::Center>
            <FlexBox
                width=68
                border=BorderStyle::Double
                border_color=Color::DarkGrey
                padding=Padding::all(1)
                gap=1
            >
                <TextSpan color=Color::White bold=true>"component_button: Headless Base-UI Button"</TextSpan>
                <Divider />

                <TextSpan color=Color::DarkGrey>"Interactive Buttons:"</TextSpan>
                <FlexBox flex_direction=FlexDirection::Row gap=1>
                    <Button
                        disabled=disable_buttons.get()
                        on_click={click("Action A")}
                    >
                        "Action A"
                    </Button>
                    <Button
                        disabled=disable_buttons.get()
                        on_click={click("Action B")}
                    >
                        "Action B"
                    </Button>
                    <Button
                        disabled=disable_buttons.get()
                        on_click={click("Action C")}
                    >
                        "Action C"
                    </Button>
                </FlexBox>

                <FlexBox height=1 />
                <Divider />

                <FlexBox flex_direction=FlexDirection::Row justify_content=Justify::SpaceBetween align_items=Align::Center>
                    <Switch checked=disable_buttons label="Disable all buttons" />
                    <Button on_click={move || clicks_count.set(0)}>
                        "Reset Counter"
                    </Button>
                </FlexBox>

                <FlexBox height=1 />

                <FlexBox border=BorderStyle::Single border_color=Color::DarkGrey padding=Padding::all(1)>
                    <TextSpan>{format!("Last button triggered: {}", last_clicked.get())}</TextSpan>
                    <TextSpan color=Color::DarkGrey>{format!("Total click count: {}", clicks_count.get())}</TextSpan>
                    <TextSpan color=Color::DarkGrey>"Tab: Navigate | Space/Enter: Click | Esc: Exit"</TextSpan>
                </FlexBox>
            </FlexBox>
        </FlexBox>
    }
}

fn main() -> std::io::Result<()> {
    App::new(|| view! { <ButtonShowcase /> })
        .on_key(sveltui::app::exit_shortcuts)
        .run()
}
