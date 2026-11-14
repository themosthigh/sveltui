use sveltui::prelude::*;
use sveltui_components::*;

#[cfg(test)]
mod combobox_tests;

#[component]
fn ComboboxDemo() -> impl IntoNode {
    let framework = state(String::new());
    let city = state(String::new());
    let cities = vec![
        ListboxOption::new("ams", "Amsterdam"),
        ListboxOption::new("ber", "Berlin"),
        ListboxOption::new("bru", "Brussels"),
        ListboxOption::new("lis", "Lisbon"),
        ListboxOption::new("nbo", "Nairobi"),
        ListboxOption::new("tyo", "Tokyo"),
    ];

    view! {
        <View width={Dimension::Percent(1.0)} height={Dimension::Percent(1.0)}
            align_items=AlignItems::Center justify_content=JustifyContent::Center>
            <View width=52 border=BorderStyle::Single border_color=Color::DarkGrey
                title=" Combobox " padding=1 gap=1>

                <TextSpan foreground_color=Color::DarkGrey>"Framework (composed from parts)"</TextSpan>
                <Combobox::Root bind:value={framework}>
                    <View>
                        <View flex_direction=FlexDirection::Row align_items=AlignItems::Center>
                            <Combobox::Input placeholder="Search frameworks" width=30 />
                            <Combobox::Trigger />
                        </View>
                        <Combobox::Content width=30>
                            <Combobox::Item value="svelte">"Svelte"</Combobox::Item>
                            <Combobox::Item value="solid">"Solid"</Combobox::Item>
                            <Combobox::Item value="sveltui">"Sveltui"</Combobox::Item>
                            <Combobox::Item value="react" disabled=true>"React"</Combobox::Item>
                            <Combobox::Item value="vue">"Vue"</Combobox::Item>
                            <Combobox::Empty>"No framework matches"</Combobox::Empty>
                        </Combobox::Content>
                    </View>
                </Combobox::Root>

                <TextSpan foreground_color=Color::DarkGrey>"City (built from an options list)"</TextSpan>
                <Combobox options=cities placeholder="Search cities" width=30
                    bind:value={city} />

                <Divider />
                <TextSpan>
                    {format!(
                        "Framework: {}  City: {}",
                        or_none(framework.get()),
                        or_none(city.get()),
                    )}
                </TextSpan>
                <Paragraph foreground_color=Color::DarkGrey>
                    "Type to filter · ↓/↑ move · Enter pick · Esc undo · Tab next"
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
    App::new(|| view! { <ComboboxDemo /> })
        .on_key(sveltui::app::exit_shortcuts)
        .run()
}
