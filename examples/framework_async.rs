use std::time::Duration;
use svtui::prelude::*;
use svtui_components::*;

async fn fetch_server_stats(endpoint: String) -> Result<String, String> {
    spawn_blocking(move || {
        std::thread::sleep(Duration::from_millis(50));
        if endpoint.contains("fail") {
            Err(String::from(
                "Endpoint unreachable: HTTP 503 Service Unavailable",
            ))
        } else {
            Ok(format!(
                "Online: 1,420 nodes active | Region: {endpoint} | Latency: 12ms"
            ))
        }
    })
    .await
}

#[component]
fn AsyncDemo() -> impl IntoNode {
    let endpoint = state(String::from("us-east-1"));

    view! {
        <Box width=Size::Fill height=Size::Fill align_items=Align::Center justify_content=Justify::Center>
            <Box
                width=72
                border=BorderStyle::Single
                border_color=Color::DarkGrey
                padding=Padding::all(1)
                gap=1
            >
                <Text color=Color::White bold=true>"framework_async: {#await} Asynchronous Runes"</Text>
                <Divider />

                <Box flex_direction=FlexDirection::Row gap=2 align_items=Align::Center>
                    <Text color=Color::DarkGrey>"Server Endpoint:"</Text>
                    <TextInput value=endpoint placeholder="Region (try 'fail' to test catch)..." />
                </Box>

                <Box flex_direction=FlexDirection::Row gap=1>
                    <Button on_click={move || endpoint.set(String::from("eu-west-1"))}>
                        "EU West"
                    </Button>
                    <Button on_click={move || endpoint.set(String::from("us-west-2"))}>
                        "US West"
                    </Button>
                    <Button on_click={move || endpoint.set(String::from("fail-region"))}>
                        "Simulate Error"
                    </Button>
                </Box>

                <Divider />

                <Box border=BorderStyle::Single border_color=Color::DarkGrey padding=Padding::all(1) min_height=4>
                    {#await fetch_server_stats(endpoint.get())}
                        <Text color=Color::DarkGrey>"[Pending] Contacting cluster API..."</Text>
                    {:then stats}
                        <Text color=Color::White bold=true>{format!("[Success] {stats}")}</Text>
                    {:catch error}
                        <Text color=Color::White>{format!("[Error] {error}")}</Text>
                    {/await}
                </Box>
            </Box>
        </Box>
    }
}

fn main() -> std::io::Result<()> {
    App::new(|| view! { <AsyncDemo /> })
        .on_key(svtui::app::exit_shortcuts)
        .run()
}
