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
