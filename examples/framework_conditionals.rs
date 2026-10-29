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
        <FlexBox width=Size::Fill height=Size::Fill align_items=Align::Center justify_content=Justify::Center>
            <FlexBox
                width=64
                border=BorderStyle::Single
                border_color=Color::DarkGrey
                padding=Padding::all(1)
                gap=1
            >
                <TextSpan color=Color::White bold=true>"framework_conditionals: {#if} Branches"</TextSpan>
                <Divider />

                <FlexBox flex_direction=FlexDirection::Row gap=1 align_items=Align::Center>
                    <TextSpan color=Color::DarkGrey>"Switch Role:"</TextSpan>
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
                </FlexBox>

                <FlexBox height=1 />
                <Checkbox checked=show_details label="Show permission details" />
                <Divider />

                // Conditional rendering with {#if} ... {:else if} ... {:else}
                {#if role.get() == AuthRole::Admin}
                    <FlexBox border=BorderStyle::Single border_color=Color::White padding=Padding::all(1)>
                        <TextSpan color=Color::White bold=true>"[Admin Access Granted]"</TextSpan>
                        <TextSpan color=Color::DarkGrey>"You have read, write, and execute permissions across all clusters."</TextSpan>
                    </FlexBox>
                {:else if role.get() == AuthRole::Guest}
                    <FlexBox border=BorderStyle::Single border_color=Color::DarkGrey padding=Padding::all(1)>
                        <TextSpan color=Color::White>"[Guest Session]"</TextSpan>
                        <TextSpan color=Color::DarkGrey>"Read-only public workspace access."</TextSpan>
                    </FlexBox>
                {:else}
                    <FlexBox border=BorderStyle::Single border_color=Color::DarkGrey padding=Padding::all(1)>
                        <TextSpan color=Color::DarkGrey>"Session locked. Please authenticate to view resources."</TextSpan>
                    </FlexBox>
                {/if}

                {#if show_details.get()}
                    <FlexBox padding=Padding::symmetric(1, 0)>
                        <TextSpan color=Color::DarkGrey>{format!("Current status: {:?}", match role.get() {
                            AuthRole::LoggedOut => "Anonymous visitor",
                            AuthRole::Guest => "Authenticated guest #408",
                            AuthRole::Admin => "Root administrator (Superuser)",
                        })}</TextSpan>
                    </FlexBox>
                {/if}
            </FlexBox>
        </FlexBox>
    }
}

fn main() -> std::io::Result<()> {
    App::new(|| view! { <ConditionalsDemo /> })
        .on_key(sveltui::app::exit_shortcuts)
        .run()
}
