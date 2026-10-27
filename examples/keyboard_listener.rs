use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use svtui::prelude::*;

fn format_key(key: KeyEvent) -> (String, String) {
    let code_str = match key.code {
        KeyCode::Char(c) => format!("Char('{}')", c),
        KeyCode::Enter => "Enter".to_string(),
        KeyCode::Backspace => "Backspace".to_string(),
        KeyCode::Left => "ArrowLeft".to_string(),
        KeyCode::Right => "ArrowRight".to_string(),
        KeyCode::Up => "ArrowUp".to_string(),
        KeyCode::Down => "ArrowDown".to_string(),
        KeyCode::Home => "Home".to_string(),
        KeyCode::End => "End".to_string(),
        KeyCode::PageUp => "PageUp".to_string(),
        KeyCode::PageDown => "PageDown".to_string(),
        KeyCode::Tab => "Tab".to_string(),
        KeyCode::BackTab => "BackTab (Shift+Tab)".to_string(),
        KeyCode::Delete => "Delete".to_string(),
        KeyCode::Esc => "Esc".to_string(),
        _ => format!("{:?}", key.code),
    };

    let mut mods = Vec::new();
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        mods.push("CTRL");
    }
    if key.modifiers.contains(KeyModifiers::SHIFT) {
        mods.push("SHIFT");
    }
    if key.modifiers.contains(KeyModifiers::ALT) {
        mods.push("ALT");
    }
    let mod_str = if mods.is_empty() {
        "None".to_string()
    } else {
        mods.join(" + ")
    };

    (code_str, mod_str)
}

fn main() -> std::io::Result<()> {
    let (last_key, set_last_key) = create_signal(String::from("None"));
    let (last_mods, set_last_mods) = create_signal(String::from("None"));
    let (event_count, set_event_count) = create_signal(0);
    let (history, set_history) = create_signal(Vec::<String>::new());

    App::new(move || {
        let key_str = last_key.get();
        let mod_str = last_mods.get();
        let count = event_count.get();
        let hist = history.get();

        view! {
            <Box
                width=Size::Fill
                height=Size::Fill
                align_items=Align::Center
                justify_content=Justify::Center
            >
                <Box
                    border=BorderStyle::Single
                    border_color=Color::White
                    title=" Keyboard Listener "
                    padding=1
                    width=46
                    align_items=Align::Center
                    justify_content=Justify::Start
                >
                    <Text color=Color::White bold=true>
                        { format!("Total Keystrokes: {}", count) }
                    </Text>

                    <Divider />

                    <Text color=Color::White>
                        { format!("Last Key:      {}", key_str) }
                    </Text>
                    <Text color=Color::White>
                        { format!("Modifiers:     {}", mod_str) }
                    </Text>

                    <Divider title="Event History" />

                    {
                        if hist.is_empty() {
                            view! { <Text color=Color::DarkGrey>"  (No keys pressed yet)"</Text> }
                        } else {
                            view! {
                                <Box align_items=Align::Center justify_content=Justify::Start>
                                    {
                                        hist.iter().map(|item| {
                                            view! { <Text color=Color::DarkGrey>{ format!("  {}", item) }</Text> }
                                        }).collect::<Vec<_>>()
                                    }
                                </Box>
                            }
                        }
                    }

                    { Divider::new().into_node() }

                    <Text color=Color::DarkGrey>
                        "Press any key to test | Press Esc or Ctrl+C to quit"
                    </Text>
                </Box>
            </Box>
        }
    })
    .on_key(move |key| {
        let (code_str, mod_str) = format_key(key);
        set_last_key.set(code_str.clone());
        set_last_mods.set(mod_str.clone());
        set_event_count.update(|c| *c += 1);

        let entry = format!("[{}] {}", mod_str, code_str);
        set_history.update(|h| {
            h.push(entry);
            if h.len() > 5 {
                h.remove(0);
            }
        });
    })
    .run()
}
