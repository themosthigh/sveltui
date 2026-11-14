use sveltui::prelude::*;
use sveltui_components::*;

#[component]
fn ButtonShowcase() -> impl IntoNode {
    let count = state(0);

    view! {
        <View width={Dimension::Percent(1.0)} height={Dimension::Percent(1.0)} align_items=AlignItems::Center justify_content=JustifyContent::Center>
            <View
                border=BorderStyle::Double
                border_color=Color::DarkGrey
                padding={Padding::only(1, 3, 0, 3)}
            >
                <TextSpan> {format!("Clicked {} times.", count.get())} </TextSpan>
                <Divider />
                <Button
                    on_click={move || count.update( |val| *val += 1)}
                    width={Dimension::Percent(1.0)}
                >
                    "Click Me"
                </Button>
            </View>
        </View>
    }
}

fn main() -> std::io::Result<()> {
    App::new(|| view! { <ButtonShowcase /> })
        .on_key(sveltui::app::exit_shortcuts)
        .run()
}
