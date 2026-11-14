use sveltui::prelude::*;

#[cfg(test)]
mod centering_tests;

#[derive(Props)]
struct FrameProps {
    title: String,
    #[snippet]
    children: Snippet,
}

/// A titled box that gives its child room to move around in.
#[component]
fn Frame(props: FrameProps) -> impl IntoNode {
    view! {
        <View flex_grow=1.0 flex_basis=0 border=BorderStyle::Single border_color=Color::DarkGrey
            title=props.title>
            {@render props.children}
        </View>
    }
}

#[component]
fn CenteringDemo() -> impl IntoNode {
    view! {
        <View width=Dimension::Percent(1.0) height=Dimension::Percent(1.0) padding=1 gap=1>
            <TextSpan foreground_color=Color::DarkGrey>
                "An auto margin takes up all the free space on its side."
            </TextSpan>

            <View flex_direction=FlexDirection::Row gap=1 flex_grow=1.0>
                <Frame title="Margin::auto_x()">
                    <View margin=Margin::auto_x() width=12 height=3 background_color=Color::White />
                </Frame>
                <Frame title="Margin::auto_y()">
                    <View margin=Margin::auto_y() width=12 height=3 background_color=Color::White />
                </Frame>
                <Frame title="Margin::auto()">
                    <View margin=Margin::auto() width=12 height=3 background_color=Color::White />
                </Frame>
            </View>

            // One auto edge pushes everything after it to the far end.
            <View flex_direction=FlexDirection::Row gap=2 border=BorderStyle::Single
                border_color=Color::DarkGrey padding=(1, 0) title="One auto edge">
                <TextSpan>"File"</TextSpan>
                <TextSpan>"Edit"</TextSpan>
                <TextSpan>"View"</TextSpan>
                <View margin={Margin { left: MarginEdge::Auto, ..Margin::zero() }}>
                    <TextSpan foreground_color=Color::DarkGrey>"Esc: quit"</TextSpan>
                </View>
            </View>
        </View>
    }
}

fn main() -> std::io::Result<()> {
    App::new(|| view! { <CenteringDemo /> })
        .on_key(sveltui::app::exit_shortcuts)
        .run()
}
