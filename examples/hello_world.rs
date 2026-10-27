use svtui::prelude::*;

fn main() -> std::io::Result<()> {
    let (count, set_count) = create_signal(0);
    let (save_info, set_save_info) = create_signal(String::from("No save yet (edit & save a file or press Ctrl+S)"));
    let (save_count, set_save_count) = create_signal(0);

    App::new(move || {
        let counter_text = format!("Count (Space): {}", count.get());
        let saves_text = format!("Repaint on save count: {}", save_count.get());
        let status_text = format!("Save status: {}", save_info.get());

        view! {
            <Box
                width=Size::Fill
                height=Size::Fill
                align_items=Align::Center
                justify_content=Justify::Center
            >
                <Box
                    border=BorderStyle::Rounded
                    border_color=Color::Cyan
                    title=" svtui "
                    padding=1
                    align_items=Align::Center
                    justify_content=Justify::Center
                >
                    <Text color=Color::Green bold=true>
                        "Hello, world!"
                    </Text>
                    <Text color=Color::Yellow>
                        { counter_text }
                    </Text>
                    <Text color=Color::Magenta>
                        { saves_text }
                    </Text>
                    <Text color=Color::DarkGrey>
                        { status_text }
                    </Text>
                    <Text color=Color::Blue>
                        "Press Space to count | Ctrl+S to save | 'q' to quit"
                    </Text>
                </Box>
            </Box>
        }
    })
    .watch_path(".")
    .on_save(move |path| {
        set_save_count.update(|c| *c += 1);
        let filename = path
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());
        set_save_info.set(format!("Saved & repainted: {}", filename));
    })
    .on_key(move |key| {
        if key.code == crossterm::event::KeyCode::Char(' ') {
            set_count.update(|c| *c += 1);
        }
    })
    .run()
}
