use svtui::prelude::*;

#[derive(Clone, Default)]
#[allow(dead_code)]
struct SubmittedData {
    name: String,
    email: String,
    bio: String,
    terms: bool,
    newsletter: bool,
}

#[component]
fn FormDemo() -> impl IntoNode {
    let name = state(String::new());
    let email = state(String::new());
    let bio = state(String::from(
        "Software engineer & TUI enthusiast.\nEnjoys declarative syntax.",
    ));
    let terms = state(false);
    let newsletter = state(false);
    let submitted = state(Option::<SubmittedData>::None);

    let on_submit = move || {
        let name_val = name.get();
        let email_val = email.get();
        let bio_val = bio.get();
        let terms_val = terms.get();
        let newsletter_val = newsletter.get();

        if !name_val.trim().is_empty() && terms_val {
            submitted.set(Some(SubmittedData {
                name: name_val,
                email: email_val,
                bio: bio_val,
                terms: terms_val,
                newsletter: newsletter_val,
            }));
        }
    };

    let on_clear = move || {
        name.set(String::new());
        email.set(String::new());
        bio.set(String::new());
        terms.set(false);
        newsletter.set(false);
        submitted.set(None);
    };

    let sub_opt = submitted.get();

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
                title=" Registration Form "
                padding=Padding::all(1)
                width=54
                gap=1
            >
                <Text color=Color::White bold=true>"Create Account"</Text>
                <Divider />

                <TextInput
                    value=name
                    placeholder="Enter full name..."
                    title="Name"
                    width=48
                />

                <TextInput
                    value=email
                    placeholder="Enter email address..."
                    title="Email"
                    width=48
                />

                <TextArea
                    value=bio.read()
                    title="Biography"
                    width=48
                    height=4
                />

                <Checkbox checked=terms label="I agree to Terms & Conditions" />
                <Switch checked=newsletter label="Subscribe to release announcements" />

                <Divider />

                <Box flex_direction=FlexDirection::Row gap=1>
                    <Button variant=ButtonVariant::Primary on_click={on_submit}>
                        "Submit Registration"
                    </Button>
                    <Button variant=ButtonVariant::Outline on_click={on_clear}>
                        "Clear"
                    </Button>
                </Box>

                {#if sub_opt.is_some()}
                    <Box
                        border=BorderStyle::Single
                        border_color=Color::White
                        padding=Padding::all(1)
                    >
                        <Text color=Color::White bold=true>"[Registration Confirmed]"</Text>
                        <Text>{format!("Name: {}", sub_opt.as_ref().unwrap().name)}</Text>
                        <Text color=Color::DarkGrey>{format!("Email: {}", sub_opt.as_ref().unwrap().email)}</Text>
                        <Text color=Color::DarkGrey>{format!("Newsletter: {}", sub_opt.as_ref().unwrap().newsletter)}</Text>
                    </Box>
                {/if}
            </Box>
        </Box>
    }
}

fn main() -> std::io::Result<()> {
    App::new(|| view! { <FormDemo /> })
        .on_key(svtui::app::exit_shortcuts)
        .run()
}
