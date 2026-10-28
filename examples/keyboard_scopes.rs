use crossterm::event::{KeyCode, KeyModifiers};
use svtui::prelude::*;

fn main() -> std::io::Result<()> {
    let (dialog, set_dialog) = create_signal(false);
    let (count, set_count) = create_signal(0);
    let (status, set_status) = create_signal(String::from("Ready"));
    let custom_id = FocusId::new();
    let dialog_id = FocusId::new();
    let submenu_id = FocusId::new();
    let submenu_layer = FocusId::new();

    App::new(move || {
        let custom_border = if use_focus(custom_id).is_focused() { BorderStyle::Double } else { BorderStyle::Single };
        view! {
            <Box width=Size::Fill height=Size::Fill align_items=Align::Center justify_content=Justify::Center>
                <Box width=Size::Fill max_width=74 padding=1 border=BorderStyle::Single title=" Keyboard scopes " gap=1>
                    <Text>"Tab visits controls. Menus use arrows. Esc dismisses one layer."</Text>
                    <Box flex_direction=FlexDirection::Row gap=1>
                        <Button on_click={move || set_dialog.set(true)}>"Open dialog"</Button>
                        <DropdownMenu trigger={view! { <Button>"Menu"</Button> }}>
                            <DropdownItem label="First action" on_select={move || set_status.set("First selected".into())} />
                            <DropdownSubmenu id=submenu_id layer_id=submenu_layer label="More">
                                <DropdownItem label="Nested action" on_select={move || set_status.set("Nested selected".into())} />
                            </DropdownSubmenu>
                            <DropdownItem label="Unavailable" disabled=true />
                        </DropdownMenu>
                        <Popover trigger={view! { <Button>"Popover"</Button> }}>
                            <Box padding=1 border=BorderStyle::Single width=26>
                                <Text>"Tab stays here; Esc closes"</Text>
                                <Button on_click={move || set_status.set("Popover action".into())}>"Action"</Button>
                            </Box>
                        </Popover>
                    </Box>
                    <Divider title="Core-only control" />
                    <Box focus_id=custom_id tab_stop=true activation=Activation::Press
                        semantics={Semantics::new(Role::Button, "Core counter")}
                        border=custom_border padding=1 on_click={move || set_count.update(|n| *n += 1)}>
                        <Text>{format!("Core counter: {}", count.get())}</Text>
                    </Box>
                    <Text>{format!("Status: {}", status.get())}</Text>
                    <Text color=Color::DarkGrey>"Q exits outside editors. Ctrl+C is an explicit shortcut."</Text>
                </Box>
                <Dialog id=dialog_id open={dialog.get()} title=" Scoped dialog " width=48
                    on_close={move || set_dialog.set(false)}
                    on_key={|key, context| {
                        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                            context.exit(); EventResult::Handled
                        } else { EventResult::Ignored }
                    }}>
                    <Text>"Tab stays here. Esc closes this dialog."</Text>
                    <DropdownMenu trigger={view! { <Button>"Dialog menu"</Button> }}>
                        <DropdownItem label="Inside action" on_select={move || set_status.set("Dialog action".into())} />
                    </DropdownMenu>
                    <Button on_click={move || set_dialog.set(false)}>"Close dialog"</Button>
                </Dialog>
            </Box>
        }
    })
    .on_key(svtui::app::exit_shortcuts)
    .run()
}
