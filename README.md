# Sveltui (WIP)

Pronounced /svɛlt/; the 'u' and 'i' are silent.
This is a TUI framework that feels like writing Svelte.

> It takes a profoundly lazy person to decide they want to build their own framework.

I reject the TEA (Elm's model) for building applications. I build websites for a living, so why don't web developers have more say over what GUI and TUI application developer experience should be like.

I therefore believe that some amount of performance can be sacrificed for the _"Joy of code"_


## Installation

```sh
cargo add sveltui@0.1.0-alpha.5 sveltui-components@0.1.0-alpha.5
```

> Pre-release versions must be requested explicitly until a stable release exists.


## Syntax example

For a comprehensive guide, see the [syntax guide](./APPLICATION_SYNTAX.md).

```rs
use std::io::Result;

use sveltui::prelude::*;
use sveltui_components::*;


// Declare the main function with possible result
pub fn main () -> Result<()> {

    // Create a new app with text and a return
    App::new(|| view! {
        <TextSpan>"Hello darkness my old friend!"</TextSpan>
    })
    .on_key(sveltui::app::exit_shortcuts) // this is optional unless you want to leave the program.
    .run()
  
}

```


A counter component would look like this (using the internal component lib)

```rs

// You can use this as <Counter />

#[component]
fn Counter () -> impl IntoNode {

    let count = state(0); // uses signals

    view! {
      <View
        height={Dimension::Percent(1.0)}
        width={Dimension::Percent(1.0)}
        justify_content=JustifyContent::Center
        align_items=AlignItems::Center
        gap=4
      >

        <Button on_click={move || count.update(|v| *v -= 1)}>
          "Decrement"
        </Button>

        // Yes, we have comments in the macro.
        // Thank you for following along : )
        <TextSpan>{count.get().to_string()}</TextSpan>
        
        <Button on_click={move || count.update(|v| *v += 1)}>
          "Increment"
        </Button>

      </View>
    }
  
}
  
```


### Inspiration

I've got to hand it to the guys at [ttt](https://tttedit.dev/). They showed me that TUIs can be made accessible to muggles such as myself. And yes, I've used Neovim and Helix for years, get over it.
