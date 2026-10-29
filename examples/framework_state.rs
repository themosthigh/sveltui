use sveltui::prelude::*;
use sveltui_components::*;

#[component]
fn StateDemo() -> impl IntoNode {
    let name = state(String::from(""));
    let greeting = derived(move || format!("Hi there, {}", name.get()));

    view! {
        <FlexBox width=Size::Fill height=Size::Fill align_items=Align::Center justify_content=Justify::Center>
            <TextInput value={name}  title="Name" placeholder="Enter name here" width=Size::Fill/>

            {#if !name.get().is_empty()}
                <Divider />
                <TextSpan>{greeting.get()}</TextSpan>
            {/if}
        </FlexBox>
    }
}

fn main() -> std::io::Result<()> {
    App::new(|| view! { <StateDemo /> })
        .on_key(sveltui::app::exit_shortcuts)
        .run()
}
