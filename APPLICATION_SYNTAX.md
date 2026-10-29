### Application Setup

```rust
use svtui::prelude::*;

fn main() -> std::io::Result<()> {
    App::new(|| view! { <Application /> })
        .on_key(svtui::app::exit_shortcuts)
        .run()
}

#[component]
fn Application() -> impl IntoNode {
    view! {
        <Box border=BorderStyle::Single padding=Padding::all(1)>
            <Text>"Hello, svtui"</Text>
        </Box>
    }
}
```

### State

```rust
#[component]
fn Counter() -> impl IntoNode {
    let count = state(0);

    view! {
        <Box>
            <Text>{count.get().to_string()}</Text>
            <Button on_click={move || count.update(|value| *value += 1)}>
                "Increment"
            </Button>
            <Button on_click={move || count.set(0)}>"Reset"</Button>
        </Box>
    }
}
```

### Derived State

```rust
#[component]
fn Total() -> impl IntoNode {
    let quantity = state(1);
    let unit_price = state(25);
    let total = derived(move || quantity.get() * unit_price.get());

    view! {
        <Box>
            <Text>{format!("Total: {}", total.get())}</Text>
            <Button on_click={move || quantity.update(|value| *value += 1)}>
                "Add one"
            </Button>
        </Box>
    }
}
```

### Effects

```rust
#[component]
fn Draft() -> impl IntoNode {
    let text = state(String::new());

    effect(move || {
        let value = text.get();
        let _ = std::fs::write("draft.txt", value);
    });

    view! { <TextInput value=text placeholder="Draft..." /> }
}
```

### Mount & Cleanup

```rust
#[component]
fn Session() -> impl IntoNode {
    on_mount(|| {
        let _ = std::fs::write("session.lock", "active");

        || {
            let _ = std::fs::remove_file("session.lock");
        }
    });

    on_destroy(|| {
        let _ = std::fs::remove_file("session.tmp");
    });

    view! { <Text>"Session active"</Text> }
}
```

### Batching

```rust
#[component]
fn Profile() -> impl IntoNode {
    let name = state(String::new());
    let email = state(String::new());

    view! {
        <Box>
            <TextInput value=name placeholder="Name" />
            <TextInput value=email placeholder="Email" />
            <Button on_click={move || batch(|| {
                name.set(String::new());
                email.set(String::new());
            })}>
                "Clear"
            </Button>
        </Box>
    }
}
```

### Props

```rust
#[derive(Props)]
struct CounterValueProps {
    count: State<i32>,
}

#[component]
fn CounterValue(props: CounterValueProps) -> impl IntoNode {
    view! { <Text>{props.count.get().to_string()}</Text> }
}

#[component]
fn CounterPanel() -> impl IntoNode {
    let count = state(0);

    view! {
        <Box>
            <CounterValue count=count />
            <Button on_click={move || count.update(|value| *value += 1)}>
                "Increment"
            </Button>
        </Box>
    }
}
```

### Optional & Default Props

```rust
#[derive(Props)]
struct GreetingProps {
    name: State<String>,
    #[prop(optional)]
    subtitle: Option<String>,
    #[prop(default = false)]
    compact: bool,
}

#[component]
fn Greeting(props: GreetingProps) -> impl IntoNode {
    view! {
        <Box gap={if props.compact { 0 } else { 1 }}>
            <Text>{format!("Hello, {}", props.name.get())}</Text>
            {#if let Some(subtitle) = &props.subtitle}
                <Text>{subtitle.clone()}</Text>
            {/if}
        </Box>
    }
}

#[component]
fn Greetings() -> impl IntoNode {
    let name = state(String::from("Ada"));

    view! {
        <Box>
            <Greeting name=name />
            <Greeting name=name subtitle=Some(String::from("Welcome")) compact=true />
        </Box>
    }
}
```

### Children & Snippets

Component children and snippets share a unified syntax via [`Snippet`](file:///Users/themosthigh/Projects/experimental/svtui/crates/svtui-element/src/snippet.rs), governed by the **One-Way Rule**:
- **Primary Children (1 Way)**: Must be placed directly inside the element body (`<Component> ... </Component>`). Passing `children=...` as a prop attribute or declaring `{#snippet children}` is prohibited and rejected at compile time.
- **Custom Named Slots (1 Way)**: Must be declared via block macro syntax inside the element body (`{#snippet slot_name} ... {/snippet}`). Passing snippets as regular prop attributes is prohibited.
- **Required vs Optional**:
  - Required slots are declared as `pub children: Snippet` or `header: Snippet<T>`.
  - Optional slots are declared as `#[prop(optional)] pub children: Option<Snippet>` or `#[prop(optional)] footer: Option<Snippet<T>>`.
- Use `.is_snippet_present()` to check whether an optional snippet contains renderable content (safely accounting for empty snippets).
- Use `{@render props.children}` or `{@render props.slot(arg)}` to render directly.

```rust
#[derive(Props)]
struct DialogProps {
    header: Snippet,
    #[prop(optional)]
    footer: Option<Snippet>,
    pub children: Snippet,
}

#[component]
fn Dialog(props: DialogProps) -> impl IntoNode {
    view! {
        <Box border=BorderStyle::Double padding=Padding::all(1)>
            <Box>{@render props.header}</Box>
            <Divider />
            <Box>{@render props.children}</Box>
            {#if let Some(footer) = &props.footer}
                <Divider />
                <Box>{@render footer}</Box>
            {/if}
        </Box>
    }
}

#[component]
fn ConfirmDelete() -> impl IntoNode {
    view! {
        <Dialog>
            {#snippet header}
                <Text>"Confirm Delete"</Text>
            {/snippet}

            <Text>"Are you sure you want to delete this item?"</Text>

            {#snippet footer}
                <Button>"Cancel"</Button>
                <Button>"Delete"</Button>
            {/snippet}
        </Dialog>
    }
}
```

### Parameterized Snippets

```rust
#[derive(Props)]
struct ListViewProps<T: 'static> {
    items: State<Vec<T>>,
    row: Snippet<T>,
}

#[component]
fn ListView<T: Clone + 'static>(props: ListViewProps<T>) -> impl IntoNode {
    view! {
        <Box>
            {#each props.items.get() as item}
                {@render props.row(item)}
            {/each}
        </Box>
    }
}

#[component]
fn UserDirectory() -> impl IntoNode {
    let users = state(vec![String::from("Ada"), String::from("Grace")]);

    view! {
        <ListView items=users>
            {#snippet row(user)}
                <Text>{user}</Text>
            {/snippet}
        </ListView>
    }
}
```

### Conditionals

```rust
#[derive(Props)]
struct ConnectionStatusProps {
    connected: State<bool>,
    retries: State<u32>,
}

#[component]
fn ConnectionStatus(props: ConnectionStatusProps) -> impl IntoNode {
    view! {
        <Box>
            {#if props.connected.get()}
                <Badge label="Online" />
            {:else if props.retries.get() > 3}
                <Text>"Connection failed"</Text>
            {:else}
                <Text>{format!("Retrying ({})...", props.retries.get())}</Text>
            {/if}
        </Box>
    }
}
```

### Loops

```rust
#[derive(Clone)]
struct Task {
    id: u64,
    title: String,
    done: State<bool>,
}

#[derive(Props)]
struct TaskListProps {
    tasks: State<Vec<Task>>,
}

#[component]
fn TaskList(props: TaskListProps) -> impl IntoNode {
    view! {
        <Box>
            {#each props.tasks.get() as task (task.id)}
                <Box flex_direction=FlexDirection::Row gap=1>
                    <Checkbox checked=task.done />
                    <Text>{task.title.clone()}</Text>
                </Box>
            {:else}
                <Text>"No tasks remaining"</Text>
            {/each}
        </Box>
    }
}
```

### Components With Macros

```rust
#[derive(Props)]
struct StepperProps {
    value: State<i32>,
}

#[component]
fn Stepper(props: StepperProps) -> impl IntoNode {
    view! {
        <Box>
            <Text>{props.value.get().to_string()}</Text>
            <Button on_click={move || props.value.update(|current| *current += 1)}>
                "+"
            </Button>
        </Box>
    }
}

#[component]
fn StepperPanel() -> impl IntoNode {
    let value = state(0);

    view! { <Stepper value=value /> }
}
```

### Components Without Macros

```rust
fn stepper(value: State<i32>) -> impl IntoNode {
    BoxElement::new()
        .child(move || TextElement::new(value.get().to_string()))
        .child(
            Button::new("+")
                .on_click(move || value.update(|current| *current += 1)),
        )
}

fn stepper_panel() -> impl IntoNode {
    let value = state(0);

    stepper(value)
}
```

### Context Providers

```rust
#[derive(Clone, Copy)]
struct SessionContext {
    username: State<String>,
}

#[component]
fn SessionProvider() -> impl IntoNode {
    let session = SessionContext {
        username: state(String::from("Ada")),
    };

    view! {
        <Provider value=session>
            <SessionName />
        </Provider>
    }
}

#[component]
fn SessionName() -> impl IntoNode {
    let session = use_context::<SessionContext>().expect("Session provider required");

    view! {
        <Box>
            <Text>{session.username.get()}</Text>
            <TextInput value=session.username />
        </Box>
    }
}
```

### Error Boundaries

```rust
#[derive(Props)]
struct ParsedNumberProps {
    input: State<String>,
}

#[component]
fn ParsedNumber(props: ParsedNumberProps) -> impl IntoNode {
    view! {
        <Text>
            {props
                .input
                .get()
                .parse::<i32>()
                .expect("Invalid number")
                .to_string()}
        </Text>
    }
}

#[component]
fn NumberEditor() -> impl IntoNode {
    let input = state(String::from("42"));

    view! {
        <Box>
            <TextInput value=input />
            <ErrorBoundary>
                {#snippet fallback(error)}
                    <Text>{format!("Component crashed: {error}")}</Text>
                {/snippet}

                <ParsedNumber input=input />
            </ErrorBoundary>
        </Box>
    }
}
```

### Async & Await

```rust
async fn load_document(path: String) -> std::io::Result<String> {
    spawn_blocking(move || std::fs::read_to_string(path)).await
}

#[component]
fn Document() -> impl IntoNode {
    let path = state(String::from("notes.txt"));

    view! {
        <Box>
            <TextInput value=path />
            {#await load_document(path.get())}
                <Text>"Loading..."</Text>
            {:then text}
                <Text>{text}</Text>
            {:catch error}
                <Text>{format!("Could not load document: {error}")}</Text>
            {/await}
        </Box>
    }
}
```

### Background Tasks

```rust
#[component]
fn BackgroundScan() -> impl IntoNode {
    let status = state(String::from("Scanning..."));

    on_mount(move || {
        let task = spawn_task(async move {
            let result = spawn_blocking(|| {
                std::fs::read_dir(".").map(|entries| entries.count())
            }).await;

            status.set(match result {
                Ok(count) => format!("Found {count} entries"),
                Err(error) => error.to_string(),
            });
        });

        move || task.abort()
    });

    view! { <Text>{status.get()}</Text> }
}
```
