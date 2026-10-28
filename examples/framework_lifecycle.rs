use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use svtui::prelude::*;

#[derive(Props)]
struct ActiveSessionProps {
    session_id: State<usize>,
}

#[component]
fn ActiveSession(props: ActiveSessionProps) -> impl IntoNode {
    let session_id = props.session_id;
    let logs = state(Vec::<String>::new());

    on_mount(move || {
        logs.update(|l| l.push(String::from("[on_mount] Session initialized")));

        move || {
            // Teardown logic
        }
    });

    on_destroy(|| {
        // Scope destruction logic
    });

    effect(move || {
        let current = session_id.get();
        logs.update(|l| {
            l.push(format!("[effect] Tracked signal changed: id = {}", current));
        });
    });

    view! {
        <Box border=BorderStyle::Single border_color=Color::DarkGrey padding=Padding::all(1) gap=1>
            <Text color=Color::White bold=true>"Active Session Component"</Text>
            <Divider />
            <Text color=Color::DarkGrey>"Recent Lifecycle Activity:"</Text>
            {#each logs.get() as log}
                <Text color=Color::White>{log}</Text>
            {:else}
                <Text color=Color::DarkGrey>"No events recorded yet"</Text>
            {/each}
        </Box>
    }
}

#[component]
fn LifecycleDemo() -> impl IntoNode {
    let session_id = state(1usize);
    let session_visible = state(true);
    let mounts_counter = Arc::new(AtomicUsize::new(0));

    let m_clone = mounts_counter.clone();
    on_mount(move || {
        m_clone.fetch_add(1, Ordering::SeqCst);
    });

    view! {
        <Box width=Size::Fill height=Size::Fill align_items=Align::Center justify_content=Justify::Center>
            <Box
                width=68
                border=BorderStyle::Double
                border_color=Color::DarkGrey
                padding=Padding::all(1)
                gap=1
            >
                <Text color=Color::White bold=true>"framework_lifecycle: on_mount, on_destroy & effect"</Text>
                <Divider />

                <Box flex_direction=FlexDirection::Row gap=1>
                    <Button
                        variant=ButtonVariant::Primary
                        on_click={move || session_id.update(|id| *id += 1)}
                    >
                        "Next Session ID"
                    </Button>
                    <Button
                        variant=ButtonVariant::Secondary
                        on_click={move || session_visible.update(|v| *v = !*v)}
                    >
                        {if session_visible.get() { "Unmount Component" } else { "Mount Component" }}
                    </Button>
                </Box>

                <Box height=1 />

                {#if session_visible.get()}
                    <ActiveSession session_id=session_id />
                {:else}
                    <Box
                        border=BorderStyle::Single
                        border_color=Color::DarkGrey
                        padding=Padding::all(2)
                        align_items=Align::Center
                    >
                        <Text color=Color::DarkGrey>"Component unmounted. Cleanups executed."</Text>
                    </Box>
                {/if}
            </Box>
        </Box>
    }
}

fn main() -> std::io::Result<()> {
    App::new(|| view! { <LifecycleDemo /> })
        .on_key(svtui::app::exit_shortcuts)
        .run()
}
