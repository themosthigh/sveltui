use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use sveltui::prelude::*;
use sveltui_components::*;

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
        <View border=BorderStyle::Single border_color=Color::DarkGrey padding={Padding::all(1)} gap=1>
            <TextSpan foreground_color=Color::White bold=true>"Active Session Component"</TextSpan>
            <Divider />
            <TextSpan foreground_color=Color::DarkGrey>"Recent Lifecycle Activity:"</TextSpan>
            {#each logs.get() as log}
                <TextSpan foreground_color=Color::White>{log}</TextSpan>
            {:else}
                <TextSpan foreground_color=Color::DarkGrey>"No events recorded yet"</TextSpan>
            {/each}
        </View>
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
        <View width={Dimension::Percent(1.0)} height={Dimension::Percent(1.0)} align_items=AlignItems::Center justify_content=JustifyContent::Center>
            <View
                width=68
                border=BorderStyle::Double
                border_color=Color::DarkGrey
                padding={Padding::all(1)}
                gap=1
            >
                <TextSpan foreground_color=Color::White bold=true>"framework_lifecycle: on_mount, on_destroy & effect"</TextSpan>
                <Divider />

                <View flex_direction=FlexDirection::Row gap=1>
                    <Button
                        on_click={move || session_id.update(|id| *id += 1)}
                    >
                        "Next Session ID"
                    </Button>
                    <Button
                        on_click={move || session_visible.update(|v| *v = !*v)}
                    >
                        {if session_visible.get() { "Unmount Component" } else { "Mount Component" }}
                    </Button>
                </View>

                <View height=1 />

                {#if session_visible.get()}
                    <ActiveSession session_id=session_id />
                {:else}
                    <View
                        border=BorderStyle::Single
                        border_color=Color::DarkGrey
                        padding={Padding::all(2)}
                        align_items=AlignItems::Center
                    >
                        <TextSpan foreground_color=Color::DarkGrey>"Component unmounted. Cleanups executed."</TextSpan>
                    </View>
                {/if}
            </View>
        </View>
    }
}

fn main() -> std::io::Result<()> {
    App::new(|| view! { <LifecycleDemo /> })
        .on_key(sveltui::app::exit_shortcuts)
        .run()
}
