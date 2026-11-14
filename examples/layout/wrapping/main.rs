use sveltui::prelude::*;
use sveltui_components::*;

#[cfg(test)]
mod wrapping_tests;

const TAGS: [&str; 12] = [
    "rust", "terminal", "layout", "flexbox", "taffy", "reactive", "signals", "focus", "keyboard",
    "mouse", "scroll", "unicode",
];

const WIDTHS: [u16; 3] = [24, 40, 60];
const ALIGNMENTS: [(&str, AlignContent); 3] = [
    ("Start", AlignContent::Start),
    ("Center", AlignContent::Center),
    ("SpaceBetween", AlignContent::SpaceBetween),
];

#[component]
fn WrappingDemo() -> impl IntoNode {
    let width = state(1usize);
    let align = state(0usize);
    let (align_name, align_content) = ALIGNMENTS[align.get()];

    view! {
        <View width={Dimension::Percent(1.0)} height={Dimension::Percent(1.0)} padding=1 gap=1>
            <TextSpan foreground_color=Color::DarkGrey>
                "flex_wrap moves items to a new line; row_gap and column_gap space them separately."
            </TextSpan>

            <View flex_direction=FlexDirection::Row gap=1>
                <Button on_click={move || width.update(|w| *w = (*w + 1) % WIDTHS.len())}>
                    {format!("Width: {}", WIDTHS[width.get()])}
                </Button>
                <Button on_click={move || align.update(|a| *a = (*a + 1) % ALIGNMENTS.len())}>
                    {format!("align_content: {align_name}")}
                </Button>
            </View>

            <View width={WIDTHS[width.get()]} height=12 border=BorderStyle::Single
                border_color=Color::DarkGrey padding={(1, 0)}
                flex_direction=FlexDirection::Row flex_wrap=FlexWrap::Wrap
                column_gap=2 row_gap=1 align_content=align_content>
                {TAGS.iter().map(|tag| view! {
                    <View background_color=Color::DarkGrey padding={(1, 0)}>
                        <TextSpan>{*tag}</TextSpan>
                    </View>
                }).collect::<Vec<_>>()}
            </View>

            <TextSpan foreground_color=Color::DarkGrey>"Tab: focus · Enter: change · Esc: quit"</TextSpan>
        </View>
    }
}

fn main() -> std::io::Result<()> {
    App::new(|| view! { <WrappingDemo /> })
        .on_key(sveltui::app::exit_shortcuts)
        .run()
}
