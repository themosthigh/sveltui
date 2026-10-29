use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use svtui::prelude::*;
use svtui_components::*;

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

#[derive(Props)]
struct KeyboardInspectorProps {
    last_key: State<String>,
    last_mods: State<String>,
    event_count: State<usize>,
    history: State<Vec<String>>,
}

#[component]
fn KeyboardInspector(props: KeyboardInspectorProps) -> impl IntoNode {
    view! {
        <FlexBox width=Size::Fill height=Size::Fill align_items=Align::Center justify_content=Justify::Center>
            <FlexBox
                width=68
                border=BorderStyle::Double
                border_color=Color::DarkGrey
                padding=Padding::all(1)
                gap=1
            >
                <FlexBox flex_direction=FlexDirection::Row justify_content=Justify::SpaceBetween align_items=Align::Center>
                    <TextSpan color=Color::White bold=true>"component_keyboard: Key & Modifier Inspector"</TextSpan>
                    <TextSpan color=Color::DarkGrey>{format!("Events: {}", props.event_count.get())}</TextSpan>
                </FlexBox>
                <Divider />

                <TextSpan color=Color::DarkGrey>"Press any key combination to analyze code and active modifiers:"</TextSpan>
                <FlexBox height=1 />

                <FlexBox border=BorderStyle::Single border_color=Color::White padding=Padding::all(1) gap=1>
                    <TextSpan color=Color::White bold=true>{format!("Key Code:  {}", props.last_key.get())}</TextSpan>
                    <TextSpan color=Color::DarkGrey>{format!("Modifiers: {}", props.last_mods.get())}</TextSpan>
                </FlexBox>

                <FlexBox height=1 />
                <TextSpan color=Color::DarkGrey>"Recent Key History (last 5):"</TextSpan>

                <FlexBox border=BorderStyle::Single border_color=Color::DarkGrey padding=Padding::all(1) min_height=6>
                    {#each props.history.get() as item}
                        <TextSpan color=Color::White>{item}</TextSpan>
                    {:else}
                        <TextSpan color=Color::DarkGrey>"No keyboard input captured yet. Press keys!"</TextSpan>
                    {/each}
                </FlexBox>

                <Divider />
                <TextSpan color=Color::DarkGrey>"Press 'Ctrl+C' or 'Esc' to exit the application."</TextSpan>
            </FlexBox>
        </FlexBox>
    }
}

fn main() -> std::io::Result<()> {
    let last_key = state(String::from("None"));
    let last_mods = state(String::from("None"));
    let event_count = state(0usize);
    let history = state(Vec::<String>::new());

    App::new(move || {
        view! {
            <KeyboardInspector
                last_key=last_key
                last_mods=last_mods
                event_count=event_count
                history=history
            />
        }
    })
    .on_key(move |key, context| {
        if key.code == KeyCode::Esc
            || (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL))
        {
            return svtui::app::exit_shortcuts(key, context);
        }

        let (code, mods) = format_key(key);
        let summary = format!(
            "[#{:03}] Key: {:<16} | Modifiers: {}",
            event_count.get() + 1,
            code,
            mods
        );

        last_key.set(code);
        last_mods.set(mods);
        event_count.update(|c| *c += 1);
        history.update(|h| {
            if h.len() >= 5 {
                h.remove(0);
            }
            h.push(summary);
        });

        EventResult::Handled
    })
    .run()
}
