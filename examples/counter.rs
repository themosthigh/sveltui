use svtui::{app, prelude::*};
use svtui_components::*;

fn main() -> std::io::Result<()> {
    App::new(|| {
        view! {
            <CounterApp />
        }
    })
    .on_key(app::exit_shortcuts)
    .run()
}

#[component]
fn CounterApp() -> impl IntoNode {
    let count = state(0);

    view! {

        <Box
            height=Size::Fill
            width=Size::Fill
            align_items=Align::Center
            justify_content=Justify::Center
        >

            <Box
                flex_direction=FlexDirection::Row
                align_items=Align::Center
                justify_content=Justify::Center
                border=BorderStyle::Double
                border_color=Color::Red
                padding=Padding::symmetric(6, 1)
                gap=4
            >

                 <Button on_click={move || count.set(count.get() - 1)}>
                     "-"
                 </Button>

                 <Text>{count.get().to_string()}</Text>

                 <Button on_click={move || count.set(count.get() + 1)}>
                     "+"
                 </Button>

            </Box>

        </Box>
    }
}
