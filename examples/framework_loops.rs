use svtui::prelude::*;
use svtui_components::*;

#[derive(Clone, PartialEq, Eq)]
struct Task {
    id: u64,
    title: String,
    done: bool,
}

#[component]
fn LoopsDemo() -> impl IntoNode {
    let tasks = state(vec![
        Task {
            id: 1,
            title: String::from("Initialize svtui application"),
            done: true,
        },
        Task {
            id: 2,
            title: String::from("Try out {#each} and {#if} runes"),
            done: true,
        },
        Task {
            id: 3,
            title: String::from("Build interactive TUI screens"),
            done: false,
        },
    ]);
    let next_id = state(4u64);

    let add_task = move || {
        let id = next_id.get();
        next_id.update(|v| *v += 1);
        tasks.update(|list| {
            list.push(Task {
                id,
                title: format!("Task number #{}", id),
                done: false,
            });
        });
    };

    let remove_last = move || {
        tasks.update(|list| {
            let _ = list.pop();
        });
    };

    let clear_all = move || {
        tasks.update(|list| list.clear());
    };

    view! {
        <FlexBox width=Size::Fill height=Size::Fill align_items=Align::Center justify_content=Justify::Center>
            <FlexBox
                width=68
                border=BorderStyle::Single
                border_color=Color::DarkGrey
                padding=Padding::all(1)
                gap=1
            >
                <FlexBox flex_direction=FlexDirection::Row justify_content=Justify::SpaceBetween>
                    <TextSpan color=Color::White bold=true>"framework_loops: {#each} List Rendering"</TextSpan>
                    <TextSpan color=Color::DarkGrey>{format!("Items: {}", tasks.get().len())}</TextSpan>
                </FlexBox>
                <Divider />

                <FlexBox flex_direction=FlexDirection::Row gap=1>
                    <Button on_click={add_task}>
                        "+ Add Task"
                    </Button>
                    <Button on_click={remove_last}>
                        "- Remove Last"
                    </Button>
                    <Button on_click={clear_all}>
                        "Clear All"
                    </Button>
                </FlexBox>

                <FlexBox height=1 />

                // Render tasks or show empty list fallback
                <FlexBox gap=1 border=BorderStyle::Single border_color=Color::DarkGrey padding=Padding::all(1)>
                    {#each tasks.get() as task (task.id)}
                        <FlexBox flex_direction=FlexDirection::Row justify_content=Justify::SpaceBetween align_items=Align::Center>
                            <FlexBox flex_direction=FlexDirection::Row gap=1>
                                <TextSpan color=if task.done { Color::DarkGrey } else { Color::White }>
                                    {if task.done { "[x]" } else { "[ ]" }}
                                </TextSpan>
                                <TextSpan color=if task.done { Color::DarkGrey } else { Color::White }>
                                    {task.title.clone()}
                                </TextSpan>
                            </FlexBox>
                            <TextSpan color=Color::DarkGrey>{format!("id:{}", task.id)}</TextSpan>
                        </FlexBox>
                    {:else}
                        <FlexBox align_items=Align::Center padding=Padding::symmetric(0, 1)>
                            <TextSpan color=Color::DarkGrey>"No tasks remaining. Click '+ Add Task' above!"</TextSpan>
                        </FlexBox>
                    {/each}
                </FlexBox>
            </FlexBox>
        </FlexBox>
    }
}

fn main() -> std::io::Result<()> {
    App::new(|| view! { <LoopsDemo /> })
        .on_key(svtui::app::exit_shortcuts)
        .run()
}
