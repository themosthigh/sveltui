use svtui::prelude::*;

fn main() -> std::io::Result<()> {
    App::new(|| view! { <Application /> })
        .on_key(svtui::app::exit_shortcuts)
        .run()
}

#[derive(Clone)]
#[allow(dead_code)]
struct TodoItem {
    id: u64,
    title: String,
    done: State<bool>,
}

#[derive(Clone, Copy)]
struct AppConfig {
    version: &'static str,
}

#[component]
fn Application() -> impl IntoNode {
    let config = AppConfig { version: "0.1.0" };

    view! {
        <Provider value=config>
            <Box border=BorderStyle::Single padding=Padding::all(1) width=Size::Fill height=Size::Fill>
                <Header title="svtui Application Syntax Demo" />
                <Divider />
                <Box flex_direction=FlexDirection::Row gap=2>
                    <CounterPanel />
                    <TasksPanel />
                </Box>
                <Divider />
                <Footer />
            </Box>
        </Provider>
    }
}

#[derive(Props)]
struct HeaderProps {
    title: String,
}

#[component]
fn Header(props: HeaderProps) -> impl IntoNode {
    let config = use_context::<AppConfig>().unwrap_or(AppConfig { version: "unknown" });

    view! {
        <Box flex_direction=FlexDirection::Row justify_content=Justify::SpaceBetween>
            <Text color=Color::White bold=true>{props.title.clone()}</Text>
            <Text color=Color::DarkGrey>{format!("v{}", config.version)}</Text>
        </Box>
    }
}

#[component]
fn CounterPanel() -> impl IntoNode {
    let count = state(0);
    let doubled = derived(move || count.get() * 2);

    view! {
        <Box border=BorderStyle::Single padding=Padding::all(1) width=Size::Fixed(36)>
            <Text color=Color::White bold=true>"Reactive State & Derived"</Text>
            <Box height=1 />
            <Text>{format!("Current count: {}", count.get())}</Text>
            <Text color=Color::DarkGrey>{format!("Doubled: {}", doubled.get())}</Text>
            <Box height=1 />
            <Box flex_direction=FlexDirection::Row gap=1>
                <Button on_click={move || count.update(|v| *v += 1)}>
                    "+"
                </Button>
                <Button on_click={move || count.update(|v| if *v > 0 { *v -= 1 })}>
                    "-"
                </Button>
                <Button on_click={move || count.set(0)}>
                    "Reset"
                </Button>
            </Box>
        </Box>
    }
}

#[component]
fn TasksPanel() -> impl IntoNode {
    let tasks = state(vec![
        TodoItem {
            id: 1,
            title: String::from("Explore template syntax"),
            done: state(true),
        },
        TodoItem {
            id: 2,
            title: String::from("Try out {#each} and {#if}"),
            done: state(false),
        },
    ]);

    view! {
        <Box border=BorderStyle::Single padding=Padding::all(1) width=Size::Fixed(42)>
            <Text color=Color::White bold=true>"Loops & Conditionals"</Text>
            <Box height=1 />
            {#each tasks.get() as task (task.id)}
                <Box flex_direction=FlexDirection::Row gap=1>
                    <Checkbox checked=task.done />
                    {#if task.done.get()}
                        <Text color=Color::DarkGrey>{task.title.clone()}</Text>
                    {:else}
                        <Text color=Color::White>{task.title.clone()}</Text>
                    {/if}
                </Box>
            {:else}
                <Text color=Color::DarkGrey>"No tasks remaining"</Text>
            {/each}
        </Box>
    }
}

#[component]
fn Footer() -> impl IntoNode {
    let status = state(String::from("System ready"));

    on_mount(move || {
        let task = spawn_task(async move {
            let res = spawn_blocking(|| "All background workers operational").await;
            status.set(res.to_string());
        });

        move || task.abort()
    });

    view! {
        <Box flex_direction=FlexDirection::Row justify_content=Justify::SpaceBetween>
            <Text color=Color::DarkGrey>{status.get()}</Text>
            <Text color=Color::DarkGrey>"Press Ctrl+C or Esc to exit"</Text>
        </Box>
    }
}
