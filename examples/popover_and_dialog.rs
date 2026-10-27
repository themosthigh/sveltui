use svtui::prelude::*;

fn main() -> std::io::Result<()> {
    let (menu_open, set_menu_open) = create_signal(false);
    let (dialog_open, set_dialog_open) = create_signal(false);
    let (popover_open, set_popover_open) = create_signal(false);
    let (status, set_status) = create_signal(String::from(
        "Ready. Use ↑/↓ to navigate menu, → to enter submenu, ← to exit submenu, Enter to select.",
    ));

    App::new(move || {
        let is_menu_open = menu_open.get();
        let is_dialog_open = dialog_open.get();
        let is_popover_open = popover_open.get();
        let status_msg = status.get();

        view! {
            <Box
                width=Size::Fill
                height=Size::Fill
                align_items=Align::Center
                justify_content=Justify::Center
            >
                <Box
                    border=BorderStyle::Single
                    border_color=Color::White
                    title=" Popover, Dropdown & Modal Dialog Demo "
                    padding=1
                    width=68
                    justify_content=Justify::Center
                >
                    <Text color=Color::DarkGrey>
                        "Top-layer overlay with arrow navigation, focus highlights, and 3-level submenus."
                    </Text>

                    <Box height=1 />

                    // Top Action Bar with 3-Level Nested Dropdown Menu and Anchored Popover
                    <Box width=Size::Fill flex_direction=FlexDirection::Row align_items=Align::Center justify_content=Justify::SpaceBetween>
                        <DropdownMenu
                            open=is_menu_open
                            placement=Placement::BottomStart
                            width=24
                            on_close={move || set_menu_open.set(false)}
                            trigger={
                                Button::new("☰  Actions")
                                    .on_click(move || set_menu_open.update(|v| *v = !*v))
                            }
                        >
                            <DropdownItem
                                label="New Project"
                                focus_bg={Color::Green}
                                focus_fg={Color::Black}
                                on_select={move || {
                                    set_status.set("Created new project.".to_string());
                                    set_menu_open.set(false);
                                }}
                            />
                            // Level 2 Submenu: "Export As"
                            <DropdownSubmenu label="Export As" width=24>
                                <DropdownItem
                                    label="Image (PNG)"
                                    on_select={move || {
                                        set_status.set("Exported workspace as PNG Image.".to_string());
                                        set_menu_open.set(false);
                                    }}
                                />
                                // Level 3 Submenu: "Document"
                                <DropdownSubmenu label="Document" width=28>
                                    <DropdownItem
                                        label="PDF (*.pdf)"
                                        on_select={move || {
                                            set_status.set("Exported Document as PDF.".to_string());
                                            set_menu_open.set(false);
                                        }}
                                    />
                                    <DropdownItem
                                        label="Markdown (*.md)"
                                        on_select={move || {
                                            set_status.set("Exported Document as Markdown.".to_string());
                                            set_menu_open.set(false);
                                        }}
                                    />
                                    <DropdownItem
                                        label="Plain Text (*.txt)"
                                        on_select={move || {
                                            set_status.set("Exported Document as Plain Text.".to_string());
                                            set_menu_open.set(false);
                                        }}
                                    />
                                </DropdownSubmenu>
                                <DropdownItem
                                    label="Archive (*.tar.gz)"
                                    on_select={move || {
                                        set_status.set("Exported workspace as TAR archive.".to_string());
                                        set_menu_open.set(false);
                                    }}
                                />
                            </DropdownSubmenu>
                            <DropdownItem
                                label="Open Dialog..."
                                on_select={move || {
                                    set_menu_open.set(false);
                                    set_dialog_open.set(true);
                                }}
                            />
                        </DropdownMenu>

                        <Popover
                            open=is_popover_open
                            placement=Placement::BottomEnd
                            on_close={move || set_popover_open.set(false)}
                            trigger={
                                Button::new("ℹ  Details")
                                    .on_click(move || set_popover_open.update(|v| *v = !*v))
                            }
                        >
                            <Box
                                border=BorderStyle::Single
                                border_color=Color::White
                                padding=1
                                width=30
                            >
                                <Text color=Color::White bold=true>"Popover Details"</Text>
                                <Text color=Color::DarkGrey>
                                    "Anchored to trigger button."
                                </Text>
                                <Text color=Color::DarkGrey>
                                    "Auto-flips on collision."
                                </Text>
                                <Box height=1 />
                                <Button
                                    width=Size::Fill
                                    on_click={move || set_popover_open.set(false)}
                                >
                                    "Close Details"
                                </Button>
                            </Box>
                        </Popover>
                    </Box>

                    <Box height=1 />

                    // Main Interactive Section
                    <Box
                        border=BorderStyle::Single
                        border_color=Color::DarkGrey
                        padding=1
                        width=Size::Fill
                        justify_content=Justify::Center
                    >
                        <Text color=Color::White>
                            "Click the button below to trigger a modal confirmation dialog:"
                        </Text>
                        <Box height=1 />
                        <Button
                            variant=ButtonVariant::Primary
                            on_click={move || set_dialog_open.set(true)}
                        >
                            "Launch Modal Confirmation Dialog"
                        </Button>
                    </Box>

                    <Box height=1 />

                    // Status line
                    <Box
                        width=Size::Fill
                        align_items=Align::Start
                    >
                        <Text color=Color::White bold=true>"Status: "</Text>
                        <Text color=Color::DarkGrey>{status_msg}</Text>
                    </Box>

                    // Modal Dialog (Rendered in top layer when open)
                    <Dialog
                        open=is_dialog_open
                        title=" Confirm Action "
                        width=52
                        padding=1
                        on_close={move || set_dialog_open.set(false)}
                    >
                        <Text color=Color::White bold=true>
                            "Discard pending modifications?"
                        </Text>
                        <Box height=1 />
                        <Text color=Color::DarkGrey>
                            "This action will reset your workspace changes."
                        </Text>
                        <Text color=Color::DarkGrey>
                            "Focus is trapped inside this modal dialog."
                        </Text>
                        <Text color=Color::DarkGrey>
                            "Press Tab to cycle buttons or Esc to dismiss."
                        </Text>
                        <Box height=1 />
                        <Box width=Size::Fill flex_direction=FlexDirection::Row justify_content=Justify::End gap=1>
                            <Button
                                on_click={move || {
                                    set_status.set("Dialog cancelled by user.".to_string());
                                    set_dialog_open.set(false);
                                }}
                            >
                                "Cancel"
                            </Button>
                            <Button
                                variant=ButtonVariant::Danger
                                on_click={move || {
                                    set_status.set("Changes discarded successfully.".to_string());
                                    set_dialog_open.set(false);
                                }}
                            >
                                "Discard"
                            </Button>
                        </Box>
                    </Dialog>
                </Box>
            </Box>
        }
    })
    .run()
}
