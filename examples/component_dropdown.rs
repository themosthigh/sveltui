use svtui::prelude::*;

#[component]
fn DropdownDemo() -> impl IntoNode {
    let menu_open = state(false);
    let selected_action = state(String::from("None"));

    let is_open = menu_open.get();

    let select = |name: &'static str| {
        move || {
            selected_action.set(String::from(name));
            menu_open.set(false);
        }
    };

    view! {
        <Box width=Size::Fill height=Size::Fill align_items=Align::Center justify_content=Justify::Center>
            <Box
                width=68
                border=BorderStyle::Single
                border_color=Color::DarkGrey
                padding=Padding::all(1)
                gap=1
            >
                <Text color=Color::White bold=true>"component_dropdown: Menus & Hierarchical Submenus"</Text>
                <Divider />

                <Text color=Color::DarkGrey>
                    "Use ↑/↓ to navigate menu items, → to open submenu, ← to go back, Enter to select."
                </Text>

                <Box height=1 />

                <Box flex_direction=FlexDirection::Row gap=2 align_items=Align::Center>
                    <DropdownMenu
                        open=is_open
                        placement=Placement::BottomStart
                        width=24
                        on_close={move || menu_open.set(false)}
                        trigger={
                            Button::new("File Menu ▾")
                                .variant(ButtonVariant::Primary)
                                .on_click(move || menu_open.update(|v| *v = !*v))
                        }
                    >
                        <DropdownItem label="New File" on_select={select("New File")} />
                        <DropdownItem label="Open Project..." on_select={select("Open Project")} />
                        <DropdownSubmenu label="Export" width=22>
                            <DropdownItem label="Export to PDF" on_select={select("Export PDF")} />
                            <DropdownItem label="Export to JSON" on_select={select("Export JSON")} />
                            <DropdownSubmenu label="Archive Format" width=22>
                                <DropdownItem label="Tarball (.tar.gz)" on_select={select("Archive .tar.gz")} />
                                <DropdownItem label="Zip Archive (.zip)" on_select={select("Archive .zip")} />
                            </DropdownSubmenu>
                        </DropdownSubmenu>
                        <DropdownItem label="Settings" on_select={select("Settings")} />
                    </DropdownMenu>
                </Box>

                <Box height=1 />
                <Divider />

                <Box border=BorderStyle::Single border_color=Color::DarkGrey padding=Padding::all(1)>
                    <Text color=Color::White>{format!("Selected Option: {}", selected_action.get())}</Text>
                    <Text color=Color::DarkGrey>"Esc / Tab: Close dropdown | Arrow keys: Navigate items"</Text>
                </Box>
            </Box>
        </Box>
    }
}

fn main() -> std::io::Result<()> {
    App::new(|| view! { <DropdownDemo /> })
        .on_key(svtui::app::exit_shortcuts)
        .run()
}
