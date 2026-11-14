use sveltui::prelude::*;
use sveltui_components::date::{self, Date, Time, Weekday};
use sveltui_components::*;

#[cfg(test)]
mod form_tests;

const FORM_WIDTH: u16 = 54;
const PREVIEW_WIDTH: u16 = 40;
const FIELD_WIDTH: u16 = 46;

fn roles() -> Vec<ListboxOption> {
    vec![
        ListboxOption::new("engineer", "Engineer"),
        ListboxOption::new("designer", "Designer"),
        ListboxOption::new("manager", "Manager"),
        ListboxOption::new("other", "Other"),
    ]
}

fn countries() -> Vec<ListboxOption> {
    [
        ("ke", "Kenya"),
        ("ng", "Nigeria"),
        ("za", "South Africa"),
        ("de", "Germany"),
        ("jp", "Japan"),
        ("br", "Brazil"),
        ("ca", "Canada"),
    ]
    .into_iter()
    .map(|(value, label)| ListboxOption::new(value, label))
    .collect()
}

/// The label an option's value stands for, or a dash.
fn label_of(options: &[ListboxOption], value: &str) -> String {
    options
        .iter()
        .find(|o| o.value == value)
        .map_or_else(|| "—".to_owned(), |o| o.label.clone())
}

fn or_dash(text: &str) -> String {
    if text.trim().is_empty() {
        "—".to_owned()
    } else {
        text.to_owned()
    }
}

fn date_text(day: Option<Date>) -> String {
    day.map_or_else(|| "—".to_owned(), date::format_date)
}

fn time_text(time: Option<Time>) -> String {
    time.map_or_else(|| "—".to_owned(), |t| date::format_time(t, false))
}

fn yes_no(on: bool) -> &'static str {
    if on { "yes" } else { "no" }
}

/// A field that doesn't draw its own label, with one beside it.
fn labelled(label: &'static str, field: VNode) -> VNode {
    view! {
        <View flex_direction=FlexDirection::Row gap=1>
            <View width=14><TextSpan foreground_color=Color::DarkGrey>{label}</TextSpan></View>
            {field}
        </View>
    }
    .into_node()
}

/// One line of the preview: a grey name and its value.
fn row(name: &'static str, value: String) -> VNode {
    view! {
        <View flex_direction=FlexDirection::Row gap=1>
            <View width=12 flex_shrink=0.0><TextSpan foreground_color=Color::DarkGrey>{name}</TextSpan></View>
            <TextSpan>{value}</TextSpan>
        </View>
    }
    .into_node()
}

#[component]
fn FormDemo() -> impl IntoNode {
    let name = state(String::new());
    let email = state(String::new());
    let bio = state(String::from(
        "Software engineer & TUI enthusiast.\nEnjoys declarative syntax.",
    ));
    let role = state(String::new());
    let country = state(String::new());
    let birthday = state(None::<Date>);
    let start = state(None::<Date>);
    let call_time = state(None::<Time>);
    let terms = state(false);
    let newsletter = state(false);
    let submitted = state(false);
    let form_scroll = use_hook(FocusId::new);
    let preview_scroll = use_hook(FocusId::new);

    let ready = !name.get().trim().is_empty() && terms.get();
    let on_submit = move || submitted.set(ready);
    let on_clear = move || {
        name.set(String::new());
        email.set(String::new());
        bio.set(String::new());
        role.set(String::new());
        country.set(String::new());
        birthday.set(None);
        start.set(None);
        call_time.set(None);
        terms.set(false);
        newsletter.set(false);
        submitted.set(false);
    };

    let fields = vec![
        labelled(
            "Birthday",
            view! {
                <DateField bind:value={birthday} label="Birthday" order=DateOrder::Dmy
                    max={Some(date::today())} />
            }
            .into_node(),
        ),
        labelled(
            "Start date",
            view! {
                <DatePicker bind:value={start} label="Start date" min={Some(date::today())}
                    is_date_unavailable={|d: Date| matches!(d.weekday(), Weekday::Saturday | Weekday::Sunday)} />
            }
            .into_node(),
        ),
        labelled(
            "Call time",
            view! {
                <TimeField bind:value={call_time} label="Call time" hour_cycle=HourCycle::H12
                    minute_step=15 />
            }
            .into_node(),
        ),
    ];

    let status = if submitted.get() {
        ("Submitted", Color::White)
    } else if ready {
        ("Ready to submit", Color::DarkGrey)
    } else {
        ("Needs a name and the terms", Color::DarkGrey)
    };
    let preview = vec![
        row("Name", or_dash(&name.get())),
        row("Email", or_dash(&email.get())),
        row("Role", label_of(&roles(), &role.get())),
        row("Country", label_of(&countries(), &country.get())),
        row("Birthday", date_text(birthday.get())),
        row("Start date", date_text(start.get())),
        row("Call time", time_text(call_time.get())),
        row("Terms", yes_no(terms.get()).to_owned()),
        row("Newsletter", yes_no(newsletter.get()).to_owned()),
    ];
    let preview_focus = use_focus(preview_scroll).is_active();

    view! {
        <View
            width={Dimension::Percent(1.0)}
            height={Dimension::Percent(1.0)}
            flex_direction=FlexDirection::Row
            align_items=AlignItems::Center
            justify_content=JustifyContent::Center
            gap=2
        >
            // The form scrolls on its own; focus moving down it scrolls it too.
            <View
                scroll_id=form_scroll
                border=BorderStyle::Single
                border_color=Color::DarkGrey
                title=" Registration Form "
                padding={Padding::all(1)}
                gap=1
                width=FORM_WIDTH
                height={Dimension::Percent(0.9)}
                max_height=40
                overflow_y=Overflow::Scroll
                scrollbar_visibility=ScrollbarVisibility::Auto
            >
                <TextSpan foreground_color=Color::White bold=true>"Create Account"</TextSpan>
                <Divider title="About you" />
                <TextInput bind:value=name placeholder="Enter full name..." title="Name"
                    width=FIELD_WIDTH />
                <TextInput bind:value=email placeholder="Enter email address..." title="Email"
                    width=FIELD_WIDTH />
                <TextArea bind:value=bio title="Biography" width=FIELD_WIDTH height=4 />

                <Divider title="Work" />
                <Select bind:value=role options={roles()} placeholder="Pick a role..."
                    width=FIELD_WIDTH />
                <Combobox bind:value=country options={countries()}
                    placeholder="Search countries..." width={FIELD_WIDTH - 4} />

                <Divider title="Dates" />
                {fields}

                <Divider title="Consent" />
                // Left-aligned at their own width, rather than centred across the form.
                <View align_items=AlignItems::FlexStart gap=1>
                    <Checkbox bind:checked=terms label="I agree to Terms & Conditions" />
                    <Switch bind:checked=newsletter label="Subscribe to release announcements" />
                </View>

                <Divider />
                <View flex_direction=FlexDirection::Row gap=1>
                    <Button on_click={on_submit}>"Submit Registration"</Button>
                    <Button on_click={on_clear}>"Clear"</Button>
                </View>
            </View>

            // The preview follows every field as it changes. It's a tab stop,
            // so the keyboard can scroll it too.
            <Pressable focus_id=preview_scroll tab_stop=true>
                <View
                    scroll_id=preview_scroll
                    border=BorderStyle::Single
                    border_color={if preview_focus { Color::White } else { Color::DarkGrey }}
                    title=" Preview "
                    padding={Padding::all(1)}
                    gap=1
                    width=PREVIEW_WIDTH
                    height={Dimension::Percent(0.9)}
                    max_height=40
                    overflow_y=Overflow::Scroll
                    scrollbar_visibility=ScrollbarVisibility::Auto
                >
                    <TextSpan foreground_color={status.1} bold=true>{status.0}</TextSpan>
                    <Divider />
                    <View>{preview}</View>
                    <Divider title="Biography" />
                    <Paragraph>{bio.get()}</Paragraph>
                </View>
            </Pressable>
        </View>
    }
}

fn main() -> std::io::Result<()> {
    App::new(|| view! { <FormDemo /> })
        .on_key(sveltui::app::exit_shortcuts)
        .run()
}
