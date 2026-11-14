use sveltui::prelude::*;
use sveltui_components::*;

#[derive(Clone, Copy, PartialEq, Eq)]
enum AuthRole {
    LoggedOut,
    Guest,
    Admin,
}

#[component]
fn ConditionalsDemo() -> impl IntoNode {
    let role = state(AuthRole::Guest);
    let show_details = state(true);

    view! {
        <View width={Dimension::Percent(1.0)} height={Dimension::Percent(1.0)} align_items=AlignItems::Center justify_content=JustifyContent::Center>
            <View
                width=64
                border=BorderStyle::Single
                border_color=Color::DarkGrey
                padding={Padding::all(1)}
                gap=1
            >
                <TextSpan foreground_color=Color::White bold=true>"framework_conditionals: {#if} Branches"</TextSpan>
                <Divider />

                <View flex_direction=FlexDirection::Row gap=1 align_items=AlignItems::Center>
                    <TextSpan foreground_color=Color::DarkGrey>"Switch Role:"</TextSpan>
                    <Button
                        on_click={move || role.set(AuthRole::LoggedOut)}
                    >
                        "Logged Out"
                    </Button>
                    <Button
                        on_click={move || role.set(AuthRole::Guest)}
                    >
                        "Guest"
                    </Button>
                    <Button
                        on_click={move || role.set(AuthRole::Admin)}
                    >
                        "Admin"
                    </Button>
                </View>

                <View height=1 />
                <Checkbox bind:checked=show_details label="Show permission details" />
                <Divider />

                // Conditional rendering with {#if} ... {:else if} ... {:else}
                {#if role.get() == AuthRole::Admin}
                    <View border=BorderStyle::Single border_color=Color::White padding={Padding::all(1)}>
                        <TextSpan foreground_color=Color::White bold=true>"[Admin Access Granted]"</TextSpan>
                        <TextSpan foreground_color=Color::DarkGrey>"You have read, write, and execute permissions across all clusters."</TextSpan>
                    </View>
                {:else if role.get() == AuthRole::Guest}
                    <View border=BorderStyle::Single border_color=Color::DarkGrey padding={Padding::all(1)}>
                        <TextSpan foreground_color=Color::White>"[Guest Session]"</TextSpan>
                        <TextSpan foreground_color=Color::DarkGrey>"Read-only public workspace access."</TextSpan>
                    </View>
                {:else}
                    <View border=BorderStyle::Single border_color=Color::DarkGrey padding={Padding::all(1)}>
                        <TextSpan foreground_color=Color::DarkGrey>"Session locked. Please authenticate to view resources."</TextSpan>
                    </View>
                {/if}

                {#if show_details.get()}
                    <View padding={Padding::symmetric(1, 0)}>
                        <TextSpan foreground_color=Color::DarkGrey>{format!("Current status: {:?}", match role.get() {
                            AuthRole::LoggedOut => "Anonymous visitor",
                            AuthRole::Guest => "Authenticated guest #408",
                            AuthRole::Admin => "Root administrator (Superuser)",
                        })}</TextSpan>
                    </View>
                {/if}
            </View>
        </View>
    }
}

fn main() -> std::io::Result<()> {
    App::new(|| view! { <ConditionalsDemo /> })
        .on_key(sveltui::app::exit_shortcuts)
        .run()
}
