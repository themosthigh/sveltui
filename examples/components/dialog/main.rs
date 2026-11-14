use sveltui::prelude::*;
use sveltui_components::*;

#[component]
fn DialogDemo() -> impl IntoNode {
    let dialog_open = state(false);
    let popover_open = state(false);
    let action_log = state(String::from("Ready. Click buttons below or press Tab."));

    let is_dlg = dialog_open.get();
    let is_pop = popover_open.get();

    view! {
        <View width={Dimension::Percent(1.0)} height={Dimension::Percent(1.0)} align_items=AlignItems::Center justify_content=JustifyContent::Center>
            <View
                width=68
                border=BorderStyle::Double
                border_color=Color::DarkGrey
                padding={Padding::all(1)}
            >
                <Paragraph foreground_color=Color::White bold=true>"component_dialog: Modals, Popovers & Scoped Traps"</Paragraph>
                <Divider />

                <TextSpan foreground_color=Color::DarkGrey>
                    "Dialogs contain keyboard navigation until dismissed with Escape or Action buttons."
                </TextSpan>

                <View height=1 />

                <View flex_direction=FlexDirection::Row gap=2>
                    <Button
                        on_click={move || dialog_open.set(true)}
                    >
                        "Open Confirm Modal"
                    </Button>

                    <Popover
                        open=is_pop
                        placement=Placement::BottomStart
                        on_close={move || popover_open.set(false)}
                    >
                        {#snippet trigger()}
                            <Button on_click={move || popover_open.update(|v| *v = !*v)}>
                                "Anchored Info Popover"
                            </Button>
                        {/snippet}
                        <View border=BorderStyle::Single border_color=Color::White padding={Padding::all(1)} gap=1>
                            <TextSpan foreground_color=Color::White bold=true>"Popover Details"</TextSpan>
                            <TextSpan foreground_color=Color::DarkGrey>"Anchored to trigger element."</TextSpan>
                            <Button
                                on_click={move || {
                                    action_log.set(String::from("Popover action confirmed."));
                                    popover_open.set(false);
                                }}
                            >
                                "Dismiss Popover"
                            </Button>
                        </View>
                    </Popover>
                </View>

                <View height=1 />
                <Divider />

                <View border=BorderStyle::Single border_color=Color::DarkGrey padding={Padding::all(1)}>
                    <TextSpan foreground_color=Color::White>{format!("Status: {}", action_log.get())}</TextSpan>
                    <TextSpan foreground_color=Color::DarkGrey>"Escape: Dismiss top-layer | Tab: Navigate inside trap"</TextSpan>
                </View>

                // Modal Dialog Overlay
                <Dialog
                    open=is_dlg
                    title=" Confirm Operation "
                    border=BorderStyle::Double
                    border_color=Color::White
                    width=48
                    on_close={move || dialog_open.set(false)}
                >
                    <View padding={Padding::all(1)} gap=1>
                        <TextSpan foreground_color=Color::White>"Proceed with irreversible cluster deployment?"</TextSpan>
                        <TextSpan foreground_color=Color::DarkGrey>"This will overwrite current staging replicas."</TextSpan>
                        <View height=1 />
                        <View flex_direction=FlexDirection::Row gap=1 justify_content=JustifyContent::FlexEnd>
                            <Button
                                on_click={move || {
                                    action_log.set(String::from("Deployment cancelled."));
                                    dialog_open.set(false);
                                }}
                            >
                                "Cancel"
                            </Button>
                            <Button
                                on_click={move || {
                                    action_log.set(String::from("Cluster deployment started successfully."));
                                    dialog_open.set(false);
                                }}
                            >
                                "Deploy Now"
                            </Button>
                        </View>
                    </View>
                </Dialog>
            </View>
        </View>
    }
}

fn main() -> std::io::Result<()> {
    App::new(|| view! { <DialogDemo /> })
        .on_key(sveltui::app::exit_shortcuts)
        .run()
}
