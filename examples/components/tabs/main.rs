use std::ops::Range;
use sveltui::prelude::*;
use sveltui_components::*;
use sveltui_examples::fixtures::THE_RAVEN;

#[cfg(test)]
mod tabs_tests;

/// A scrollable run of stanzas. Mounted only while its tab is selected.
#[derive(Props)]
struct StanzasProps {
    stanzas: Range<usize>,
}

#[component]
fn Stanzas(props: StanzasProps) -> impl IntoNode {
    let scroll = use_hook(FocusId::new);
    let text = THE_RAVEN[props.stanzas].join("\n\n");
    view! {
        <Pressable focus_id=scroll tab_stop=true>
            <View height=14 scroll_id=scroll overflow_y=Overflow::Scroll
                border=BorderStyle::Single border_color=Color::DarkGrey padding={Padding::all(1)}>
                <Paragraph foreground_color=Color::White content=text />
            </View>
        </Pressable>
    }
}

#[component]
fn RavenTabs() -> impl IntoNode {
    // `bind:value` keeps `part` in sync with the selected tab. For full
    // control, pass `value` and `on_value_change` instead; leave both out and
    // `default_value` (or the first enabled tab) is selected.
    let part = state(String::from("part-1"));

    view! {
        <View width={Dimension::Percent(1.0)} height={Dimension::Percent(1.0)}
            align_items=AlignItems::Center justify_content=JustifyContent::Center>
            <View width={Dimension::Percent(1.0)} max_width=76 border=BorderStyle::Single
                title=" The Raven — Edgar Allan Poe " padding=1 gap=1>

                // Automatic: arrows switch the panel immediately.
                <Tabs::Root
                    bind:value={part}
                    activation_mode=TabsActivation::Automatic
                >
                    <View gap=1>
                        <Tabs::List>
                            <Tabs::Trigger value="part-1">"I – VI"</Tabs::Trigger>
                            <Tabs::Trigger value="part-2">"VII – XII"</Tabs::Trigger>
                            <Tabs::Trigger value="part-3">"XIII – XVIII"</Tabs::Trigger>
                            <Tabs::Trigger value="notes" disabled=true>"Notes"</Tabs::Trigger>
                        </Tabs::List>
                        <Tabs::Content value="part-1"><Stanzas stanzas={0..6} /></Tabs::Content>
                        <Tabs::Content value="part-2"><Stanzas stanzas={6..12} /></Tabs::Content>
                        <Tabs::Content value="part-3"><Stanzas stanzas={12..THE_RAVEN.len()} /></Tabs::Content>
                        <Tabs::Content value="notes"><Paragraph>"Unavailable."</Paragraph></Tabs::Content>
                    </View>
                </Tabs::Root>

                <Divider title="Single stanzas" />

                // Vertical + manual: arrows move focus, Enter/Space selects.
                <Tabs::Root
                    default_value="midnight"
                    orientation=Orientation::Vertical
                    activation_mode=TabsActivation::Manual
                >
                    <View flex_direction=FlexDirection::Row gap=2>
                        <Tabs::List>
                            <Tabs::Trigger value="midnight">"Midnight"</Tabs::Trigger>
                            <Tabs::Trigger value="lenore">"Lenore"</Tabs::Trigger>
                            <Tabs::Trigger value="shadow">"Shadow"</Tabs::Trigger>
                        </Tabs::List>
                        <Tabs::Content value="midnight">
                            <Paragraph foreground_color=Color::White>{THE_RAVEN[0]}</Paragraph>
                        </Tabs::Content>
                        <Tabs::Content value="lenore">
                            <Paragraph foreground_color=Color::White>{THE_RAVEN[1]}</Paragraph>
                        </Tabs::Content>
                        <Tabs::Content value="shadow">
                            <Paragraph foreground_color=Color::White>{THE_RAVEN[THE_RAVEN.len() - 1]}</Paragraph>
                        </Tabs::Content>
                    </View>
                </Tabs::Root>

                <TextSpan foreground_color=Color::DarkGrey>
                    "Tab: next stop | Arrows: switch tabs / scroll | Enter/Space: select | Q/Esc: quit"
                </TextSpan>
            </View>
        </View>
    }
}

fn main() -> std::io::Result<()> {
    App::new(|| view! { <RavenTabs /> })
        .on_key(sveltui::app::exit_shortcuts)
        .run()
}
