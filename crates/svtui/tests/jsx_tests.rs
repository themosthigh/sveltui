use svtui::prelude::*;

#[test]
fn test_jsx_custom_components_expansion() {
    let (val, set_val) = create_signal(String::from("Hello"));
    let (checked, set_checked) = create_signal(true);

    let _node: VNode = view! {
        <Box
            min_width=20
            max_width=80
            min_height=5
            max_height=30
            overflow=Overflow::Scroll
            scrollbar_visibility=ScrollbarVisibility::Auto
        >
            <Button variant=ButtonVariant::Primary>"Click Me"</Button>
            <Switch checked=checked set_checked=set_checked label="Toggle" />
            <Checkbox checked=checked set_checked=set_checked label="Accept" />
            <TextInput value=val set_value=set_val placeholder="Type..." title="Input" />
            <Divider title="Separator" />
            <ProgressBar ratio=0.75 width=20 />
            <Badge color=Color::Green>"Active"</Badge>
        </Box>
    };
}

#[test]
fn test_jsx_overlay_components_expansion() {
    let (is_open, set_open) = create_signal(true);
    let (modal_open, set_modal_open) = create_signal(false);

    let _node: VNode = view! {
        <Box width=80 height=24>
            <DropdownMenu
                open={is_open.get()}
                placement=Placement::BottomStart
                on_close={move || set_open.set(false)}
                trigger=Button::new("Actions")
            >
                <DropdownItem label="Cut" />
                <DropdownItem label="Copy" />
                <DropdownItem label="Paste" />
            </DropdownMenu>

            <Dialog
                open={modal_open.get()}
                title="Confirm Delete"
                on_close={move || set_modal_open.set(false)}
            >
                <Text>"Are you sure you want to delete this file?"</Text>
                <Box justify_content=Justify::End>
                    <Button on_click={move || set_modal_open.set(false)}>"Cancel"</Button>
                    <Button variant=ButtonVariant::Danger on_click={move || set_modal_open.set(false)}>"Delete"</Button>
                </Box>
            </Dialog>
        </Box>
    };
}
