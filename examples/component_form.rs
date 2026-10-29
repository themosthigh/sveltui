use sveltui::prelude::*;
use sveltui_components::*;

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
        <FlexBox
            width=Size::Fill
            height=Size::Fill
            align_items=Align::Center
            justify_content=Justify::Center
        >
            <FlexBox
                border=BorderStyle::Single
                border_color=Color::DarkGrey
                title=" Registration Form "
                padding=Padding::all(1)
                width=54
                gap=1
            >
                <TextSpan color=Color::White bold=true>"Create Account"</TextSpan>
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

                <FlexBox flex_direction=FlexDirection::Row gap=1>
                    <Button on_click={on_submit}>
                        "Submit Registration"
                    </Button>
                    <Button on_click={on_clear}>
                        "Clear"
                    </Button>
                </FlexBox>

                {#if sub_opt.is_some()}
                    <FlexBox
                        border=BorderStyle::Single
                        border_color=Color::White
                        padding=Padding::all(1)
                    >
                        <TextSpan color=Color::White bold=true>"[Registration Confirmed]"</TextSpan>
                        <TextSpan>{format!("Name: {}", sub_opt.as_ref().unwrap().name)}</TextSpan>
                        <TextSpan color=Color::DarkGrey>{format!("Email: {}", sub_opt.as_ref().unwrap().email)}</TextSpan>
                        <TextSpan color=Color::DarkGrey>{format!("Newsletter: {}", sub_opt.as_ref().unwrap().newsletter)}</TextSpan>
                    </FlexBox>
                {/if}
            </FlexBox>
        </FlexBox>
    }
}

fn main() -> std::io::Result<()> {
    App::new(|| view! { <FormDemo /> })
        .on_key(sveltui::app::exit_shortcuts)
        .run()
}
