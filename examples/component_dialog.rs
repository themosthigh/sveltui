use svtui::prelude::*;
use svtui_components::*;

#[component]
fn DialogDemo() -> impl IntoNode {
    let dialog_open = state(false);
    let popover_open = state(false);
    let action_log = state(String::from("Ready. Click buttons below or press Tab."));

    let is_dlg = dialog_open.get();
    let is_pop = popover_open.get();

    view! {
        <Box width=Size::Fill height=Size::Fill align_items=Align::Center justify_content=Justify::Center>
            <Box
                width=68
                border=BorderStyle::Double
                border_color=Color::DarkGrey
                padding=Padding::all(1)
                gap=1
            >
                <Text color=Color::White bold=true>"component_dialog: Modals, Popovers & Scoped Traps"</Text>
                <Divider />

                <Text color=Color::DarkGrey>
                    "Dialogs contain keyboard navigation until dismissed with Escape or Action buttons."
                </Text>

                <Box height=1 />

                <Box flex_direction=FlexDirection::Row gap=2>
                    <Button
                        on_click={move || dialog_open.set(true)}
                    >
                        "Open Confirm Modal"
                    </Button>

                    <Popover
                        open=is_pop
                        placement=Placement::BottomStart
                        on_close={move || popover_open.set(false)}
                        trigger={
                            Button(
                                ButtonProps::builder()
                                    .children("Anchored Info Popover")
                                    .on_click(move || popover_open.update(|v| *v = !*v))
                                    .build(),
                            )
                        }
                    >
                        <Box border=BorderStyle::Single border_color=Color::White padding=Padding::all(1) gap=1>
                            <Text color=Color::White bold=true>"Popover Details"</Text>
                            <Text color=Color::DarkGrey>"Anchored to trigger element."</Text>
                            <Button
                                on_click={move || {
                                    action_log.set(String::from("Popover action confirmed."));
                                    popover_open.set(false);
                                }}
                            >
                                "Dismiss Popover"
                            </Button>
                        </Box>
                    </Popover>
                </Box>

                <Box height=1 />
                <Divider />

                <Box border=BorderStyle::Single border_color=Color::DarkGrey padding=Padding::all(1)>
                    <Text color=Color::White>{format!("Status: {}", action_log.get())}</Text>
                    <Text color=Color::DarkGrey>"Escape: Dismiss top-layer | Tab: Navigate inside trap"</Text>
                </Box>

                // Modal Dialog Overlay
                <Dialog
                    open=is_dlg
                    title=" Confirm Operation "
                    border=BorderStyle::Double
                    border_color=Color::White
                    width=48
                    on_close={move || dialog_open.set(false)}
                >
                    <Box padding=Padding::all(1) gap=1>
                        <Text color=Color::White>"Proceed with irreversible cluster deployment?"</Text>
                        <Text color=Color::DarkGrey>"This will overwrite current staging replicas."</Text>
                        <Box height=1 />
                        <Box flex_direction=FlexDirection::Row gap=1 justify_content=Justify::End>
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
                        </Box>
                    </Box>
                </Dialog>
            </Box>
        </Box>
    }
}

fn main() -> std::io::Result<()> {
    App::new(|| view! { <DialogDemo /> })
        .on_key(svtui::app::exit_shortcuts)
        .run()
}
