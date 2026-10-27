use svtui::prelude::*;

#[derive(Clone, Default)]
struct SubmittedData {
    name: String,
    email: String,
    bio: String,
    terms: bool,
    newsletter: bool,
}

fn main() -> std::io::Result<()> {
    let (name, set_name) = create_signal(String::new());
    let (email, set_email) = create_signal(String::new());
    let (bio, set_bio) = create_signal(String::from(
        "Software engineer & TUI enthusiast.\nEnjoys building reactive apps.",
    ));
    let (terms, set_terms) = create_signal(false);
    let (newsletter, set_newsletter) = create_signal(false);
    let (submitted, set_submitted) = create_signal(Option::<SubmittedData>::None);

    App::new(move || {
        let current_sub = submitted.get();

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
                    title=" Form "
                    padding=1
                    width=46
                    align_items=Align::Center
                    justify_content=Justify::Start
                >
                    <Text color=Color::White bold=true>
                        "User Registration"
                    </Text>

                    <TextInput
                        value=name
                        set_value=set_name
                        placeholder="Enter full name..."
                        title="Name"
                        width=38
                    />

                    <TextInput
                        value=email
                        set_value=set_email
                        placeholder="Enter email address..."
                        title="Email"
                        width=38
                    />

                    <TextArea
                        value=bio
                        set_value=set_bio
                        placeholder="Enter your biography..."
                        title="Bio"
                        width=38
                        height=6
                        show_line_numbers=true
                    />

                    <Checkbox
                        checked=terms
                        set_checked=set_terms
                        label="I accept terms and conditions"
                    />

                    <Switch
                        checked=newsletter
                        set_checked=set_newsletter
                        label="Subscribe to newsletter"
                    />

                    <Box
                        align_items=Align::Center
                        justify_content=Justify::Center
                    >
                        <Button
                            variant=ButtonVariant::Primary
                            on_click={move || {
                                set_submitted.set(Some(SubmittedData {
                                    name: name.get(),
                                    email: email.get(),
                                    bio: bio.get(),
                                    terms: terms.get(),
                                    newsletter: newsletter.get(),
                                }));
                            }}
                        >
                            "Submit Form"
                        </Button>

                        <Button
                            variant=ButtonVariant::Secondary
                            on_click={move || {
                                set_name.set(String::new());
                                set_email.set(String::new());
                                set_bio.set(String::new());
                                set_terms.set(false);
                                set_newsletter.set(false);
                                set_submitted.set(None);
                            }}
                        >
                            "Clear"
                        </Button>
                    </Box>

                    {
                        match current_sub {
                            Some(data) => {
                                let bio_preview = if data.bio.is_empty() {
                                    "(Empty)".to_string()
                                } else {
                                    data.bio.lines().next().unwrap_or("").to_string()
                                };
                                view! {
                                    <Box
                                        border=BorderStyle::Single
                                        border_color=Color::DarkGrey
                                        title=" Submitted Summary "
                                        padding=1
                                        width=38
                                        align_items=Align::Center
                                        justify_content=Justify::Start
                                    >
                                        <Text color=Color::White>{ format!("Name:       {}", if data.name.is_empty() { "(Empty)" } else { &data.name }) }</Text>
                                        <Text color=Color::White>{ format!("Email:      {}", if data.email.is_empty() { "(Empty)" } else { &data.email }) }</Text>
                                        <Text color=Color::White>{ format!("Bio:        {}", bio_preview) }</Text>
                                        <Text color=Color::White>{ format!("Terms:      {}", if data.terms { "Accepted" } else { "Declined" }) }</Text>
                                        <Text color=Color::White>{ format!("Newsletter: {}", if data.newsletter { "Subscribed" } else { "No" }) }</Text>
                                    </Box>
                                }
                            }
                            None => {
                                view! {
                                    <Text color=Color::DarkGrey>
                                        "Tab: Navigate | Space/Enter: Toggle | Esc: Exit"
                                    </Text>
                                }
                            }
                        }
                    }
                </Box>
            </Box>
        }
    })
    .run()
}
