use sveltui::prelude::*;
use sveltui_components::{Button, ButtonProps, MenuBar};

#[component]
fn MenuBarDemo() -> impl IntoNode {
    let selected = state(String::from("No action yet"));
    view! {
        <FlexBox width=Size::Fill height=Size::Fill padding=1 gap=1>
            <MenuBar::Root>
                <MenuBar::Menu value="file">
                    <MenuBar::Trigger>"File"</MenuBar::Trigger>
                    <MenuBar::Content>
                        <MenuBar::Item on_select={move || selected.set("New file".into())}>
                            "New"
                        </MenuBar::Item>
                        <MenuBar::Sub>
                            <MenuBar::SubTrigger>"Export"</MenuBar::SubTrigger>
                            <MenuBar::SubContent>
                                <MenuBar::Item on_select={move || selected.set("Exported JSON".into())}>
                                    "JSON"
                                </MenuBar::Item>
                            </MenuBar::SubContent>
                        </MenuBar::Sub>
                    </MenuBar::Content>
                </MenuBar::Menu>
                <MenuBar::Menu value="edit">
                    <MenuBar::Trigger>"Edit"</MenuBar::Trigger>
                    <MenuBar::Content>
                        <MenuBar::Item on_select={move || selected.set("Copied".into())}>
                            "Copy"
                        </MenuBar::Item>
                        <MenuBar::Item on_select={move || selected.set("Pasted".into())}>
                            "Paste"
                        </MenuBar::Item>
                    </MenuBar::Content>
                </MenuBar::Menu>
            </MenuBar::Root>
            <TextSpan>"Tab enters the bar; Left/Right selects a menu; Down opens it."</TextSpan>
            <TextSpan>"Hover switches open menus. Escape closes; Tab exits."</TextSpan>
            <Button>"After menu bar"</Button>
            <TextSpan>{selected.get()}</TextSpan>
        </FlexBox>
    }
}

fn main() -> std::io::Result<()> {
    App::new(|| view! { <MenuBarDemo /> })
        .on_key(sveltui::app::exit_shortcuts)
        .run()
}
