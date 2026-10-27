use svtui::prelude::*;

fn main() -> std::io::Result<()> {
    let scroll_id = FocusId::new();

    App::new(move || {
        let focus = use_focus(scroll_id);
        let border_style = if focus.is_active() {
            BorderStyle::Double
        } else {
            BorderStyle::Single
        };

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
                    title=" Scrollable Container Showcase "
                    padding=1
                    width=52
                    align_items=Align::Center
                    justify_content=Justify::Start
                >
                    <Text color=Color::White bold=true>
                        "Scroll Container Demo"
                    </Text>
                    <Text color=Color::DarkGrey>
                        "Use Mouse Wheel or Up/Down/PgUp/PgDn to scroll"
                    </Text>

                    <Divider title="Log Stream" />

                    <Box
                        border=border_style
                        border_color=Color::White
                        focus_id=scroll_id
                        width=46
                        height=10
                        padding=Padding::symmetric(1, 0)
                        overflow=Overflow::Scroll
                        scrollbar_visibility=ScrollbarVisibility::Auto
                    >
                        <Text color=Color::White>"[00:01] System boot initialized"</Text>
                        <Text color=Color::DarkGrey>"[00:02] Loading kernel modules..."</Text>
                        <Text color=Color::DarkGrey>"[00:03] Mount root filesystem read-only"</Text>
                        <Text color=Color::White>"[00:04] Checking system integrity: OK"</Text>
                        <Text color=Color::DarkGrey>"[00:05] Starting network subsystem..."</Text>
                        <Text color=Color::DarkGrey>"[00:06] DHCP lease obtained: 192.168.1.105"</Text>
                        <Text color=Color::White>"[00:07] DNS service reachable: 1.1.1.1"</Text>
                        <Text color=Color::DarkGrey>"[00:08] Initializing terminal device: /dev/tty0"</Text>
                        <Text color=Color::DarkGrey>"[00:09] Entering raw mode & alternate screen buffer"</Text>
                        <Text color=Color::White>"[00:10] Double-buffering initialized (60 FPS)"</Text>
                        <Text color=Color::DarkGrey>"[00:11] Reactive runtime engine active"</Text>
                        <Text color=Color::DarkGrey>"[00:12] Hit-test map ready for mouse/pointer events"</Text>
                        <Text color=Color::White>"[00:13] Sizing engine: min_width/max_width active"</Text>
                        <Text color=Color::DarkGrey>"[00:14] Scrollbar styling: track [░] thumb [█]"</Text>
                        <Text color=Color::White>"[00:15] All services operational and ready"</Text>
                    </Box>

                    <Divider />

                    <Text color=Color::DarkGrey>
                        "Tab: Focus container | Esc: Exit"
                    </Text>
                </Box>
            </Box>
        }
    })
    .run()
}
