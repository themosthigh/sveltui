use sveltui::prelude::*;
use sveltui_components::ContextMenu;

#[component]
fn ContextMenuDemo() -> impl IntoNode {
    let selected = state(String::from("No action yet"));
    view! {
        <FlexBox width=Size::Fill height=Size::Fill padding=2 gap=1>
            <TextSpan>"Right-click the area below, or focus it and press Shift+F10."</TextSpan>
            <ContextMenu::Root>
                <ContextMenu::Trigger>"Document actions"</ContextMenu::Trigger>
                <ContextMenu::Content>
                    <ContextMenu::Group>
                        <ContextMenu::GroupHeading>"Document"</ContextMenu::GroupHeading>
                        <ContextMenu::Item on_select={move || selected.set("Copied".into())}>
                            "Copy"
                        </ContextMenu::Item>
                        <ContextMenu::Sub>
                            <ContextMenu::SubTrigger>"Export"</ContextMenu::SubTrigger>
                            <ContextMenu::SubContent>
                                <ContextMenu::Item on_select={move || selected.set("Exported JSON".into())}>
                                    "JSON"
                                </ContextMenu::Item>
                            </ContextMenu::SubContent>
                        </ContextMenu::Sub>
                    </ContextMenu::Group>
                    <ContextMenu::Separator />
                    <ContextMenu::Item disabled=true>"Delete (unavailable)"</ContextMenu::Item>
                </ContextMenu::Content>
            </ContextMenu::Root>
            <TextSpan>{selected.get()}</TextSpan>
        </FlexBox>
    }
}

fn main() -> std::io::Result<()> {
    App::new(|| view! { <ContextMenuDemo /> })
        .on_key(sveltui::app::exit_shortcuts)
        .run()
}
