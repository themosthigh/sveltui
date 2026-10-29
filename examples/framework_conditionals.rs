use svtui::prelude::*;
use svtui_components::*;

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
        <Box width=Size::Fill height=Size::Fill align_items=Align::Center justify_content=Justify::Center>
            <Box
                width=64
                border=BorderStyle::Single
                border_color=Color::DarkGrey
                padding=Padding::all(1)
                gap=1
            >
                <Text color=Color::White bold=true>"framework_conditionals: {#if} Branches"</Text>
                <Divider />

                <Box flex_direction=FlexDirection::Row gap=1 align_items=Align::Center>
                    <Text color=Color::DarkGrey>"Switch Role:"</Text>
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
                </Box>

                <Box height=1 />
                <Checkbox checked=show_details label="Show permission details" />
                <Divider />

                // Conditional rendering with {#if} ... {:else if} ... {:else}
                {#if role.get() == AuthRole::Admin}
                    <Box border=BorderStyle::Single border_color=Color::White padding=Padding::all(1)>
                        <Text color=Color::White bold=true>"[Admin Access Granted]"</Text>
                        <Text color=Color::DarkGrey>"You have read, write, and execute permissions across all clusters."</Text>
                    </Box>
                {:else if role.get() == AuthRole::Guest}
                    <Box border=BorderStyle::Single border_color=Color::DarkGrey padding=Padding::all(1)>
                        <Text color=Color::White>"[Guest Session]"</Text>
                        <Text color=Color::DarkGrey>"Read-only public workspace access."</Text>
                    </Box>
                {:else}
                    <Box border=BorderStyle::Single border_color=Color::DarkGrey padding=Padding::all(1)>
                        <Text color=Color::DarkGrey>"Session locked. Please authenticate to view resources."</Text>
                    </Box>
                {/if}

                {#if show_details.get()}
                    <Box padding=Padding::symmetric(1, 0)>
                        <Text color=Color::DarkGrey>{format!("Current status: {:?}", match role.get() {
                            AuthRole::LoggedOut => "Anonymous visitor",
                            AuthRole::Guest => "Authenticated guest #408",
                            AuthRole::Admin => "Root administrator (Superuser)",
                        })}</Text>
                    </Box>
                {/if}
            </Box>
        </Box>
    }
}

fn main() -> std::io::Result<()> {
    App::new(|| view! { <ConditionalsDemo /> })
        .on_key(svtui::app::exit_shortcuts)
        .run()
}
