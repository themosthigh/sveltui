use svtui::prelude::*;

/// Example of a custom reusable container using `use_focus` to implement DOM-like `:focus-within`.
fn focus_card(title: &str, children: impl IntoIterator<Item = impl IntoNode>) -> VNode {
    let card_id = FocusId::new();
    let focus = use_focus(card_id);

    // Completely opt-in: highlights container border when any child contains focus!
    let (border_color, border_style) = if focus.has_focus_within() {
        (Color::Cyan, BorderStyle::Double)
    } else {
        (Color::DarkGrey, BorderStyle::Single)
    };

    let title_text = if focus.has_focus_within() {
        format!(" {} [focus-within] ", title)
    } else {
        format!(" {} ", title)
    };

    BoxElement::new()
        .border(border_style)
        .border_color(border_color)
        .title(title_text)
        .padding(Padding::symmetric(1, 0))
        .focus_id(card_id)
        .width(50)
        .children(children)
        .into_node()
}

fn main() -> std::io::Result<()> {
    let (name, set_name) = create_signal(String::new());
    let (dark_mode, set_dark_mode) = create_signal(true);
    let (notifications, set_notifications) = create_signal(false);
    let (click_count, set_click_count) = create_signal(0);
    let (status, set_status) = create_signal(String::from("Ready"));

    App::new(move || {
        let name_val = name.get();
        let count_val = click_count.get();
        let progress = (name_val.len() as f32 / 10.0).clamp(0.0, 1.0);

        let status_text = format!("Status: {} | Clicks: {}", status.get(), count_val);

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
                    title=" Interactive Components Showcase "
                    padding=1
                    width=54
                    justify_content=Justify::Center
                    align_items=Align::Start
                >
                    <Text color=Color::Green bold=true>
                        "Tab: Navigate Focus | Mouse: Click | Esc: Exit"
                    </Text>

                    <Divider title="Text Input Pattern" />

                    <TextInput
                        value=name
                        set_value=set_name
                        placeholder="Enter your name..."
                        title="User Name"
                        width=46
                    />

                    <ProgressBar
                        ratio=progress
                        width=28
                        title="Name Length Meter"
                    />

                    <Divider title="Custom Element (:focus-within)" />

                    {
                        focus_card("Settings Card", vec![
                            view! { <Switch checked=dark_mode set_checked=set_dark_mode label="Enable Dark Mode" /> },
                            view! { <Checkbox checked=notifications set_checked=set_notifications label="Subscribe to Updates" /> },
                        ])
                    }

                    <Divider title="Actions & Badges" />

                    <Box
                        align_items=Align::Center
                        justify_content=Justify::Center
                    >
                        <Badge color=Color::Green>"ONLINE"</Badge>
                        <Button
                            variant=ButtonVariant::Primary
                            on_click={move || {
                                set_click_count.update(|c| *c += 1);
                                set_status.set(String::from("Button clicked!"));
                            }}
                        >
                            "Click Me (+1)"
                        </Button>
                        <Button
                            variant=ButtonVariant::Danger
                            on_click={move || {
                                set_click_count.set(0);
                                set_name.set(String::new());
                                set_notifications.set(false);
                                set_status.set(String::from("Form reset"));
                            }}
                        >
                            "Reset"
                        </Button>
                    </Box>

                    <Divider />

                    <Text color=Color::Yellow>
                        { status_text }
                    </Text>
                </Box>
            </Box>
        }
    })
    .run()
}
