use sveltui::prelude::*;
use sveltui_components::*;

#[cfg(test)]
mod select_tests;

#[component]
fn SelectDemo() -> impl IntoNode {
    let fruit = state(String::new());
    let size = state(String::from("md"));
    let sizes = vec![
        ListboxOption::new("sm", "Small"),
        ListboxOption::new("md", "Medium"),
        ListboxOption::new("lg", "Large"),
        ListboxOption::new("xl", "Extra large").disabled(true),
    ];

    view! {
        <View width={Dimension::Percent(1.0)} height={Dimension::Percent(1.0)}
            align_items=AlignItems::Center justify_content=JustifyContent::Center>
            <View width=48 border=BorderStyle::Single border_color=Color::DarkGrey
                title=" Select " padding=1 gap=1>

                <TextSpan foreground_color=Color::DarkGrey>"Fruit (grouped, composed from parts)"</TextSpan>
                <Select::Root bind:value={fruit}>
                    <View>
                        <Select::Trigger width=30>
                            <Select::Value placeholder="Pick a fruit" />
                        </Select::Trigger>
                        <Select::Content width=30>
                            <Select::Group>
                                <Select::GroupHeading>"Berries"</Select::GroupHeading>
                                <Select::Item value="blueberry">"Blueberry"</Select::Item>
                                <Select::Item value="raspberry">"Raspberry"</Select::Item>
                            </Select::Group>
                            <Select::Separator />
                            <Select::Group>
                                <Select::GroupHeading>"Stone fruit"</Select::GroupHeading>
                                <Select::Item value="cherry">"Cherry"</Select::Item>
                                <Select::Item value="mango" disabled=true>"Mango"</Select::Item>
                                <Select::Item value="peach">"Peach"</Select::Item>
                            </Select::Group>
                        </Select::Content>
                    </View>
                </Select::Root>

                <TextSpan foreground_color=Color::DarkGrey>"Size (built from an options list)"</TextSpan>
                <Select options=sizes bind:value={size} width=30 />

                <Divider />
                <TextSpan>
                    {format!("Fruit: {}  Size: {}", or_none(fruit.get()), size.get())}
                </TextSpan>
                <Paragraph foreground_color=Color::DarkGrey>
                    "Enter/Space/↓ open · ↑↓ Home End move · type to jump · Enter pick · Esc close"
                </Paragraph>
            </View>
        </View>
    }
}

fn or_none(value: String) -> String {
    if value.is_empty() {
        String::from("none")
    } else {
        value
    }
}

fn main() -> std::io::Result<()> {
    App::new(|| view! { <SelectDemo /> })
        .on_key(sveltui::app::exit_shortcuts)
        .run()
}
