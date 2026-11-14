use sveltui::prelude::*;
use sveltui_components::*;

#[cfg(test)]
mod binding_tests;

#[derive(Props)]
struct NameFieldProps {
    #[prop(bindable)]
    value: Bindable<String>,
    #[prop(default = String::from("Name"))]
    label: String,
}

/// A labelled text input that can itself be bound. It hands its prop on as
/// it is, so the `TextInput` gets the bound state or the read-only value,
/// whichever the parent gave.
#[component]
fn NameField(props: NameFieldProps) -> impl IntoNode {
    view! {
        <View flex_direction=FlexDirection::Row align_items=AlignItems::Center gap=1>
            <TextSpan foreground_color=Color::DarkGrey>{format!("{:>8}", props.label)}</TextSpan>
            <TextInput bind:value={props.value}
                placeholder="Type here..." width=28 />
        </View>
    }
}

#[component]
fn BindingDemo() -> impl IntoNode {
    let name = state(String::new());
    let nickname = state(String::new());
    let shout = state(String::new());

    view! {
        <View width=Dimension::Percent(1.0) height=Dimension::Percent(1.0)
            align_items=AlignItems::Center justify_content=JustifyContent::Center>
            <View width=56 border=BorderStyle::Single border_color=Color::DarkGrey
                title=" Two-way binding " padding=1 gap=1>

                <TextSpan foreground_color=Color::DarkGrey>"Two inputs bound to one state"</TextSpan>
                <TextInput bind:value={name} placeholder="First name" width=40 />
                <TextInput bind:value={name} placeholder="Same first name" width=40 />

                <TextSpan foreground_color=Color::DarkGrey>"A component that passes its binding on"</TextSpan>
                <NameField label="Nickname" bind:value={nickname} />

                <TextSpan foreground_color=Color::DarkGrey>"Read-only, upper-cased by the parent"</TextSpan>
                <TextInput value={shout.get()} placeholder="Type quietly"
                    on_value_change={move |next: String| shout.set(next.to_uppercase())} width=40 />

                <Divider />
                <TextSpan>{format!("name: {:?}", name.get())}</TextSpan>
                <TextSpan>{format!("nickname: {:?}", nickname.get())}</TextSpan>
                <TextSpan>{format!("shout: {:?}", shout.get())}</TextSpan>
                <Paragraph foreground_color=Color::DarkGrey>
                    "Tab: next input · type to edit · Esc: quit"
                </Paragraph>
            </View>
        </View>
    }
}

fn main() -> std::io::Result<()> {
    App::new(|| view! { <BindingDemo /> })
        .on_key(sveltui::app::exit_shortcuts)
        .run()
}
