use sveltui::prelude::*;
use sveltui_components::*;

use sveltui_examples::fixtures::THE_RAVEN;

#[cfg(test)]
mod paragraph_tests;

#[component]
fn ParagraphDemo() -> impl IntoNode {
    let scroll_id = use_hook(FocusId::new);
    let poem = THE_RAVEN.join("\n\n");
    view! {
        <View width={Dimension::Percent(1.0)} height={Dimension::Percent(1.0)}
            align_items=AlignItems::Center justify_content=JustifyContent::Center>
            <Pressable focus_id=scroll_id tab_stop=true>
                <View width={Dimension::Percent(1.0)} max_width=68 height={Dimension::Percent(1.0)} max_height=40
                    scroll_id=scroll_id
                    border=BorderStyle::Single
                    border_color=Color::DarkGrey padding={Padding::all(1)} gap=1
                    overflow_y=Overflow::Scroll>
                    <TextSpan foreground_color=Color::White bold=true>"The Raven"</TextSpan>
                    <TextSpan foreground_color=Color::DarkGrey>"Edgar Allan Poe"</TextSpan>
                    <Divider />

                    <Paragraph foreground_color=Color::DarkGrey max_lines=2>
                        {poem.clone()}
                    </Paragraph>

                    <Divider title="Full poem" />
                    <Paragraph foreground_color=Color::White content=poem />

                    <Divider />
                    <Paragraph foreground_color=Color::DarkGrey>"Tab: Focus | Up / Down / Wheel: Scroll | Q / Esc: Quit"</Paragraph>
                </View>
            </Pressable>
        </View>
    }
}

fn main() -> std::io::Result<()> {
    App::new(|| view! { <ParagraphDemo /> })
        .on_key(sveltui::app::exit_shortcuts)
        .run()
}
