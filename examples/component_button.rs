use sveltui::prelude::*;
use sveltui_components::*;

#[component]
fn ButtonShowcase() -> impl IntoNode {
    let count = state(0);

    view! {
        <FlexBox width=Size::Fill height=Size::Fill align_items=Align::Center justify_content=Justify::Center>
            <FlexBox
                border=BorderStyle::Double
                border_color=Color::DarkGrey
                padding=Padding::only(1, 3, 0, 3)
            >
                <TextSpan> {format!("Clicked {} times.", count.get())} </TextSpan>
                <Divider />
                <Button
                    on_click={move || count.update( |val| *val += 1)}
                    width=Size::Fill
                >
                    "Click Me"
                </Button>
            </FlexBox>
        </FlexBox>
    }
}

fn main() -> std::io::Result<()> {
    App::new(|| view! { <ButtonShowcase /> })
        .on_key(sveltui::app::exit_shortcuts)
        .run()
}
