use sveltui::prelude::*;
use sveltui_components::*;

#[cfg(test)]
mod sizing_tests;

#[component]
fn SizingDemo() -> impl IntoNode {
    let sidebar = state(true);
    let wide = state(true);
    let panel_width: u16 = if wide.get() { 70 } else { 30 };

    view! {
        <View width={Dimension::Percent(1.0)} height={Dimension::Percent(1.0)} padding=1 gap=1>
            <View flex_direction=FlexDirection::Row gap=1>
                <Button on_click={move || sidebar.update(|s| *s = !*s)}>
                    {if sidebar.get() { "Hide sidebar" } else { "Show sidebar" }}
                </Button>
                <Button on_click={move || wide.update(|w| *w = !*w)}>
                    {format!("Panel width: {panel_width}")}
                </Button>
            </View>

            <View flex_direction=FlexDirection::Row gap=1 width=panel_width height=12>
                // `display: none` keeps the view in the tree but gives it no
                // space, so the content next to it takes over.
                <View display={if sidebar.get() { Display::Flex } else { Display::None }}
                    width=16 border=BorderStyle::Single border_color=Color::DarkGrey title="Sidebar">
                    <TextSpan>"Files"</TextSpan>
                    <TextSpan>"Search"</TextSpan>
                </View>

                <View flex_grow=1.0 border=BorderStyle::Single border_color=Color::DarkGrey
                    title="Content" padding={(1, 0)} gap=1>
                    // Grows with the panel, but never past 30 or under 12.
                    <View min_width=12 max_width=30 height=1 background_color=Color::DarkGrey>
                        <TextSpan>"min 12, max 30"</TextSpan>
                    </View>

                    // Width over height. Cells are about twice as tall as
                    // they're wide, so 2.0 looks roughly square.
                    <View flex_direction=FlexDirection::Row gap=2 align_items=AlignItems::Start>
                        <View width=8 aspect_ratio=2.0 border=BorderStyle::Single title="2:1" />
                        <View width=12 aspect_ratio=4.0 border=BorderStyle::Single title="4:1" />
                    </View>
                </View>
            </View>

            <TextSpan foreground_color=Color::DarkGrey>"Tab: focus · Enter: toggle · Esc: quit"</TextSpan>
        </View>
    }
}

fn main() -> std::io::Result<()> {
    App::new(|| view! { <SizingDemo /> })
        .on_key(sveltui::app::exit_shortcuts)
        .run()
}
