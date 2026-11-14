use sveltui::prelude::*;
use sveltui_components::*;

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
            title: String::from("Initialize sveltui application"),
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
        <View width={Dimension::Percent(1.0)} height={Dimension::Percent(1.0)} align_items=AlignItems::Center justify_content=JustifyContent::Center>
            <View
                width=68
                border=BorderStyle::Single
                border_color=Color::DarkGrey
                padding={Padding::all(1)}
                gap=1
            >
                <View flex_direction=FlexDirection::Row justify_content=JustifyContent::SpaceBetween>
                    <TextSpan foreground_color=Color::White bold=true>"framework_loops: {#each} List Rendering"</TextSpan>
                    <TextSpan foreground_color=Color::DarkGrey>{format!("Items: {}", tasks.get().len())}</TextSpan>
                </View>
                <Divider />

                <View flex_direction=FlexDirection::Row gap=1>
                    <Button on_click={add_task}>
                        "+ Add Task"
                    </Button>
                    <Button on_click={remove_last}>
                        "- Remove Last"
                    </Button>
                    <Button on_click={clear_all}>
                        "Clear All"
                    </Button>
                </View>

                <View height=1 />

                // Render tasks or show empty list fallback
                <View gap=1 border=BorderStyle::Single border_color=Color::DarkGrey padding={Padding::all(1)}>
                    {#each tasks.get() as task (task.id)}
                        <View flex_direction=FlexDirection::Row justify_content=JustifyContent::SpaceBetween align_items=AlignItems::Center>
                            <View flex_direction=FlexDirection::Row gap=1>
                                <TextSpan foreground_color={if task.done { Color::DarkGrey } else { Color::White }}>
                                    {if task.done { "[x]" } else { "[ ]" }}
                                </TextSpan>
                                <TextSpan foreground_color={if task.done { Color::DarkGrey } else { Color::White }}>
                                    {task.title.clone()}
                                </TextSpan>
                            </View>
                            <TextSpan foreground_color=Color::DarkGrey>{format!("id:{}", task.id)}</TextSpan>
                        </View>
                    {:else}
                        <View align_items=AlignItems::Center padding={Padding::symmetric(0, 1)}>
                            <TextSpan foreground_color=Color::DarkGrey>"No tasks remaining. Click '+ Add Task' above!"</TextSpan>
                        </View>
                    {/each}
                </View>
            </View>
        </View>
    }
}

fn main() -> std::io::Result<()> {
    App::new(|| view! { <LoopsDemo /> })
        .on_key(sveltui::app::exit_shortcuts)
        .run()
}
