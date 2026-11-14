use sveltui::prelude::*;
use sveltui_components::*;

#[cfg(test)]
mod resizable_tests;

#[derive(Props)]
struct PaneProps {
    title: &'static str,
    body: &'static str,
}

#[component]
fn Pane(props: PaneProps) -> impl IntoNode {
    view! {
        <View padding=1 gap=1>
            <TextSpan foreground_color=Color::White bold=true>{props.title}</TextSpan>
            <Paragraph foreground_color=Color::DarkGrey>{props.body}</Paragraph>
        </View>
    }
}

#[component]
fn Workbench() -> impl IntoNode {
    // `bind:layout` keeps the outer split's sizes in `sizes`, so the footer
    // can show them. Leave it out and the group keeps its own.
    let sizes = state(Vec::<f32>::new());
    // The layout is only written once a handle moves, so start from the
    // panels' own defaults.
    let current = sizes.get();
    let shown = if current.is_empty() {
        String::from("25% / 75%")
    } else {
        current
            .iter()
            .map(|s| format!("{s:.0}%"))
            .collect::<Vec<_>>()
            .join(" / ")
    };

    view! {
        <View width={Dimension::Percent(1.0)} height={Dimension::Percent(1.0)} gap=1>
            <View flex_grow=1.0 border=BorderStyle::Single border_color=Color::DarkGrey>
                <Resizable::PanelGroup bind:layout={sizes}>
                    <Resizable::Panel default_size=25.0 min_size=15.0 max_size=50.0
                        collapsible=true>
                        <Pane title="Sidebar" body="Enter on a handle collapses this panel." />
                    </Resizable::Panel>

                    <Resizable::Handle with_handle=true label="Sidebar width" />

                    <Resizable::Panel default_size=75.0>
                        <Resizable::PanelGroup direction=Orientation::Vertical>
                            <Resizable::Panel default_size=65.0 min_size=20.0>
                                <Pane title="Editor" body="Drag a handle, or focus it and use the arrow keys." />
                            </Resizable::Panel>

                            <Resizable::Handle label="Terminal height" />

                            <Resizable::Panel default_size=35.0 min_size=10.0>
                                <Pane title="Terminal" body="Vertical groups resize with Up and Down." />
                            </Resizable::Panel>
                        </Resizable::PanelGroup>
                    </Resizable::Panel>
                </Resizable::PanelGroup>
            </View>
            <TextSpan foreground_color=Color::DarkGrey>
                {format!("Sidebar / main: {shown} | Tab: next handle | Arrows: resize (Shift: 5x) | Home/End | Enter: collapse | Q/Esc: quit")}
            </TextSpan>
        </View>
    }
}

fn main() -> std::io::Result<()> {
    App::new(|| view! { <Workbench /> })
        .on_key(sveltui::app::exit_shortcuts)
        .run()
}
