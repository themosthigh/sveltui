use std::time::Duration;
use sveltui::prelude::*;
use sveltui_components::*;

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
        <View width={Dimension::Percent(1.0)} height={Dimension::Percent(1.0)} align_items=AlignItems::Center justify_content=JustifyContent::Center>
            <View
                width=72
                border=BorderStyle::Single
                border_color=Color::DarkGrey
                padding={Padding::all(1)}
                gap=1
            >
                <TextSpan foreground_color=Color::White bold=true>"framework_async: {#await} Asynchronous Runes"</TextSpan>
                <Divider />

                <View flex_direction=FlexDirection::Row gap=2 align_items=AlignItems::Center>
                    <TextSpan foreground_color=Color::DarkGrey>"Server Endpoint:"</TextSpan>
                    <TextInput bind:value=endpoint placeholder="Region (try 'fail' to test catch)..." />
                </View>

                <View flex_direction=FlexDirection::Row gap=1>
                    <Button on_click={move || endpoint.set(String::from("eu-west-1"))}>
                        "EU West"
                    </Button>
                    <Button on_click={move || endpoint.set(String::from("us-west-2"))}>
                        "US West"
                    </Button>
                    <Button on_click={move || endpoint.set(String::from("fail-region"))}>
                        "Simulate Error"
                    </Button>
                </View>

                <Divider />

                <View border=BorderStyle::Single border_color=Color::DarkGrey padding={Padding::all(1)} min_height=4>
                    {#await fetch_server_stats(endpoint.get())}
                        <TextSpan foreground_color=Color::DarkGrey>"[Pending] Contacting cluster API..."</TextSpan>
                    {:then stats}
                        <TextSpan foreground_color=Color::White bold=true>{format!("[Success] {stats}")}</TextSpan>
                    {:catch error}
                        <TextSpan foreground_color=Color::White>{format!("[Error] {error}")}</TextSpan>
                    {/await}
                </View>
            </View>
        </View>
    }
}

fn main() -> std::io::Result<()> {
    App::new(|| view! { <AsyncDemo /> })
        .on_key(sveltui::app::exit_shortcuts)
        .run()
}
