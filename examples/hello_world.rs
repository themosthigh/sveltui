use svtui::prelude::*;

#[component]
fn App() -> impl IntoNode {
    view! {
        <Box
            width=Size::Fill
            height=Size::Fill
            align_items=Align::Center
            justify_content=Justify::Center
        >
            <Box
                border=BorderStyle::Single
                border_color=Color::DarkGrey
                padding=Padding::all(2)
                align_items=Align::Center
            >
                <Text color=Color::White bold=true>"Hello, svtui!"</Text>
                <Box height=1 />
                <Text color=Color::DarkGrey>"Declarative terminal UI framework for Rust"</Text>
            </Box>
        </Box>
    }
}

fn main() -> std::io::Result<()> {
    svtui::app::App::new(|| view! { <App /> })
        .on_key(svtui::app::exit_shortcuts)
        .run()
}
