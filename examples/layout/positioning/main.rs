use sveltui::prelude::*;
use sveltui_components::*;

#[cfg(test)]
mod positioning_tests;

#[component]
fn PositioningDemo() -> impl IntoNode {
    let saved = state(false);

    view! {
        // Absolute children are placed against this view.
        <View width={Dimension::Percent(1.0)} height={Dimension::Percent(1.0)} padding=1 gap=1>
            <TextSpan foreground_color=Color::DarkGrey>
                "Position::Absolute takes a view out of the flow; inset places it in its parent."
            </TextSpan>

            <View width=44 height=7 border=BorderStyle::Single border_color=Color::DarkGrey
                title="Inbox" padding={(1, 0)}>
                <TextSpan>"Ada: The engine runs."</TextSpan>
                <TextSpan>"Grace: Found a moth in relay 70."</TextSpan>
                // Pinned to the top-right corner, over whatever is there.
                <View position=Position::Absolute inset={Inset::new().top(0).right(1)}>
                    <Badge label="2 new" />
                </View>
            </View>

            <Button on_click={move || saved.update(|s| *s = !*s)}>"Toggle toast"</Button>

            // Pinned to the screen's bottom-right corner. It never pushes the
            // content above it around.
            {saved.get().then(|| view! {
                <View position=Position::Absolute inset={Inset::new().bottom(1).right(2)}
                    border=BorderStyle::Single padding={(1, 0)}>
                    <TextSpan>"Saved"</TextSpan>
                </View>
            })}

            <View position=Position::Absolute inset={Inset::new().bottom(0).left(1)}>
                <TextSpan foreground_color=Color::DarkGrey>"Tab: focus · Enter: toggle · Esc: quit"</TextSpan>
            </View>
        </View>
    }
}

fn main() -> std::io::Result<()> {
    App::new(|| view! { <PositioningDemo /> })
        .on_key(sveltui::app::exit_shortcuts)
        .run()
}
