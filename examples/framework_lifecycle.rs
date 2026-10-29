use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use svtui::prelude::*;
use svtui_components::*;

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
        <FlexBox border=BorderStyle::Single border_color=Color::DarkGrey padding=Padding::all(1) gap=1>
            <TextSpan color=Color::White bold=true>"Active Session Component"</TextSpan>
            <Divider />
            <TextSpan color=Color::DarkGrey>"Recent Lifecycle Activity:"</TextSpan>
            {#each logs.get() as log}
                <TextSpan color=Color::White>{log}</TextSpan>
            {:else}
                <TextSpan color=Color::DarkGrey>"No events recorded yet"</TextSpan>
            {/each}
        </FlexBox>
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
        <FlexBox width=Size::Fill height=Size::Fill align_items=Align::Center justify_content=Justify::Center>
            <FlexBox
                width=68
                border=BorderStyle::Double
                border_color=Color::DarkGrey
                padding=Padding::all(1)
                gap=1
            >
                <TextSpan color=Color::White bold=true>"framework_lifecycle: on_mount, on_destroy & effect"</TextSpan>
                <Divider />

                <FlexBox flex_direction=FlexDirection::Row gap=1>
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
                </FlexBox>

                <FlexBox height=1 />

                {#if session_visible.get()}
                    <ActiveSession session_id=session_id />
                {:else}
                    <FlexBox
                        border=BorderStyle::Single
                        border_color=Color::DarkGrey
                        padding=Padding::all(2)
                        align_items=Align::Center
                    >
                        <TextSpan color=Color::DarkGrey>"Component unmounted. Cleanups executed."</TextSpan>
                    </FlexBox>
                {/if}
            </FlexBox>
        </FlexBox>
    }
}

fn main() -> std::io::Result<()> {
    App::new(|| view! { <LifecycleDemo /> })
        .on_key(svtui::app::exit_shortcuts)
        .run()
}
