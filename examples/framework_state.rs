use svtui::prelude::*;
use svtui_components::*;

#[component]
fn StateDemo() -> impl IntoNode {
    let name = state(String::from(""));
    let greeting = derived(move || format!("Hi there, {}", name.get()));

    view! {
        <Box width=Size::Fill height=Size::Fill align_items=Align::Center justify_content=Justify::Center>
            <TextInput value={name}  title="Name" placeholder="Enter name here" width=Size::Fill/>

            {#if !name.get().is_empty()}
                <Divider />
                <Text>{greeting.get()}</Text>
            {/if}
        </Box>
    }
}

fn main() -> std::io::Result<()> {
    App::new(|| view! { <StateDemo /> })
        .on_key(svtui::app::exit_shortcuts)
        .run()
}
