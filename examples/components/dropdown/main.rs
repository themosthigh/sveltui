use sveltui::prelude::*;
use sveltui_components::*;

#[component]
fn DropdownDemo() -> impl IntoNode {
    let menu_open = state(false);
    let selected_action = state(String::from("None"));

    let is_open = menu_open.get();

    let select = move |name: &'static str| {
        move || {
            selected_action.set(String::from(name));
            menu_open.set(false);
        }
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
                <TextSpan foreground_color=Color::White bold=true>"component_dropdown: Menus & Hierarchical Submenus"</TextSpan>
                <Divider />

                <TextSpan foreground_color=Color::DarkGrey>
                    "Use ↑/↓ to navigate menu items, → to open submenu, ← to go back, Enter to select."
                </TextSpan>

                <View height=1 />

                <View flex_direction=FlexDirection::Row gap=2 align_items=AlignItems::Center>
                    <DropdownMenu::Root open=is_open on_open_change={move |next| menu_open.set(next)}>
                        <DropdownMenu::Trigger>
                            "File Menu ▾"
                        </DropdownMenu::Trigger>
                        <DropdownMenu::Content placement=Placement::BottomStart width=24>

                            <DropdownMenu::Item on_select={select("New File")}>"New File"</DropdownMenu::Item>
                            <DropdownMenu::Item on_select={select("Open Project")}>"Open Project..."</DropdownMenu::Item>
                            <DropdownMenu::Sub>
                                <DropdownMenu::SubTrigger>"Export"</DropdownMenu::SubTrigger>
                                <DropdownMenu::SubContent width=22>
                                    <DropdownMenu::Item on_select={select("Export PDF")}>"Export to PDF"</DropdownMenu::Item>
                                    <DropdownMenu::Item on_select={select("Export JSON")}>"Export to JSON"</DropdownMenu::Item>
                                    <DropdownMenu::Sub>
                                        <DropdownMenu::SubTrigger>"Archive Format"</DropdownMenu::SubTrigger>
                                        <DropdownMenu::SubContent width=22>
                                            <DropdownMenu::Item on_select={select("Archive .tar.gz")}>"Tarball (.tar.gz)"</DropdownMenu::Item>
                                            <DropdownMenu::Item on_select={select("Archive .zip")}>"Zip Archive (.zip)"</DropdownMenu::Item>
                                        </DropdownMenu::SubContent>
                                    </DropdownMenu::Sub>
                                </DropdownMenu::SubContent>
                            </DropdownMenu::Sub>
                            <DropdownMenu::Item on_select={select("Settings")}>"Settings"</DropdownMenu::Item>
                        </DropdownMenu::Content>
                    </DropdownMenu::Root>
                </View>

                <View height=1 />
                <Divider />

                <View border=BorderStyle::Single border_color=Color::DarkGrey padding={Padding::all(1)}>
                    <TextSpan foreground_color=Color::White>{format!("Selected Option: {}", selected_action.get())}</TextSpan>
                    <TextSpan foreground_color=Color::DarkGrey>"Esc / Tab: Close dropdown | Arrow keys: Navigate items"</TextSpan>
                </View>
            </View>
        </View>
    }
}

fn main() -> std::io::Result<()> {
    App::new(|| view! { <DropdownDemo /> })
        .on_key(sveltui::app::exit_shortcuts)
        .run()
}
