use sveltui::prelude::*;
use sveltui_components::*;

#[component]
fn StateDemo() -> impl IntoNode {
    let name = state(String::from(""));
    let greeting = derived(move || format!("Hi there, {}", name.get()));

    view! {
        <View width={Dimension::Percent(1.0)} height={Dimension::Percent(1.0)} align_items=AlignItems::Center justify_content=JustifyContent::Center>
            <TextInput bind:value={name}  title="Name" placeholder="Enter name here" width={Dimension::Percent(1.0)}/>

            {#if !name.get().is_empty()}
                <Divider />
                <TextSpan>{greeting.get()}</TextSpan>
            {/if}
        </View>
    }
}

fn main() -> std::io::Result<()> {
    App::new(|| view! { <StateDemo /> })
        .on_key(sveltui::app::exit_shortcuts)
        .run()
}
