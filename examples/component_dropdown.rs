use sveltui::prelude::*;
use sveltui_components::*;

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
        <FlexBox width=Size::Fill height=Size::Fill align_items=Align::Center justify_content=Justify::Center>
            <FlexBox
                width=68
                border=BorderStyle::Single
                border_color=Color::DarkGrey
                padding=Padding::all(1)
                gap=1
            >
                <TextSpan color=Color::White bold=true>"component_dropdown: Menus & Hierarchical Submenus"</TextSpan>
                <Divider />

                <TextSpan color=Color::DarkGrey>
                    "Use ↑/↓ to navigate menu items, → to open submenu, ← to go back, Enter to select."
                </TextSpan>

                <FlexBox height=1 />

                <FlexBox flex_direction=FlexDirection::Row gap=2 align_items=Align::Center>
                    <DropdownMenu
                        open=is_open
                        placement=Placement::BottomStart
                        width=24
                        on_close={move || menu_open.set(false)}
                    >
                        {#snippet trigger()}
                            <Button on_click={move || menu_open.update(|v| *v = !*v)}>
                                "File Menu ▾"
                            </Button>
                        {/snippet}
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
                </FlexBox>

                <FlexBox height=1 />
                <Divider />

                <FlexBox border=BorderStyle::Single border_color=Color::DarkGrey padding=Padding::all(1)>
                    <TextSpan color=Color::White>{format!("Selected Option: {}", selected_action.get())}</TextSpan>
                    <TextSpan color=Color::DarkGrey>"Esc / Tab: Close dropdown | Arrow keys: Navigate items"</TextSpan>
                </FlexBox>
            </FlexBox>
        </FlexBox>
    }
}

fn main() -> std::io::Result<()> {
    App::new(|| view! { <DropdownDemo /> })
        .on_key(sveltui::app::exit_shortcuts)
        .run()
}
