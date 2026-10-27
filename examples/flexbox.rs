use svtui::prelude::*;

fn main() -> std::io::Result<()> {
    let (direction, set_direction) = create_signal(FlexDirection::Row);
    App::new(move || {
        let axis = direction.get();
        view! {
            <Box width=Size::Fill height=Size::Fill align_items=Align::Center justify_content=Justify::Center>
                <Box width=Size::Fill max_width=78 height=Size::Fill max_height=28
                    border=BorderStyle::Single title=" Flexbox " padding=1 gap=1 overflow=Overflow::Hidden>
                    <Box width=Size::Fill flex_direction=FlexDirection::Row
                        align_items=Align::Center justify_content=Justify::SpaceBetween>
                        <Text bold=true>{format!("Direction: {axis:?}")}</Text>
                        <Button on_click={move || set_direction.set(match axis {
                            FlexDirection::Row => FlexDirection::Column,
                            FlexDirection::Column => FlexDirection::Row,
                        })}>"Switch direction"</Button>
                    </Box>
                    <Box width=Size::Fill height=Size::Fill flex_direction=axis gap=1 overflow=Overflow::Auto>
                        <Box width=Size::Fill height=Size::Fill min_width=12 max_width=26 min_height=3
                            border=BorderStyle::Single title=" Capped " overflow=Overflow::Hidden>
                            <Text>"Fill, max width 26"</Text>
                        </Box>
                        <Box width=Size::Fill height=Size::Fill min_width=12 min_height=3
                            border=BorderStyle::Single title=" Flexible " overflow=Overflow::Hidden>
                            <Text>"Shares remaining space"</Text>
                        </Box>
                        <Box width=Size::Fill height=Size::Fill min_width=12 min_height=3
                            border=BorderStyle::Single title=" Flexible " overflow=Overflow::Hidden>
                            <Text>"Same Fill allocation"</Text>
                        </Box>
                    </Box>
                    <Box width=Size::Fill height=4 flex_direction=FlexDirection::Row gap=1 overflow_x=Overflow::Scroll>
                        <Box width=30 height=3 border=BorderStyle::Single><Text>"Scroll horizontally"</Text></Box>
                        <Box width=30 height=3 border=BorderStyle::Single><Text>"Fixed widths stay fixed"</Text></Box>
                        <Box width=30 height=3 border=BorderStyle::Single><Text>"Wheel or Shift+wheel"</Text></Box>
                    </Box>
                    <Text color=Color::DarkGrey>"R: row | C: column | Tab/Enter: switch | Q: quit"</Text>
                </Box>
            </Box>
        }
    })
    .on_key(move |key| match key.code {
        crossterm::event::KeyCode::Char('r') => set_direction.set(FlexDirection::Row),
        crossterm::event::KeyCode::Char('c') => set_direction.set(FlexDirection::Column),
        _ => {}
    })
    .run()
}
