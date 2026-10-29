### Application Setup

```rust
use sveltui::prelude::*;

fn main() -> std::io::Result<()> {
    App::new(|| view! { <Application /> })
        .on_key(sveltui::app::exit_shortcuts)
        .run()
}

#[component]
fn Application() -> impl IntoNode {
    view! {
        <FlexBox border=BorderStyle::Single padding=Padding::all(1)>
            <TextSpan>"Hello, sveltui"</TextSpan>
        </FlexBox>
    }
}
```

### State

```rust
#[component]
fn Counter() -> impl IntoNode {
    let count = state(0);

    view! {
        <FlexBox>
            <TextSpan>{count.get().to_string()}</TextSpan>
            <Button on_click={move || count.update(|value| *value += 1)}>
                "Increment"
            </Button>
            <Button on_click={move || count.set(0)}>"Reset"</Button>
        </FlexBox>
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
        <FlexBox>
            <TextSpan>{format!("Total: {}", total.get())}</TextSpan>
            <Button on_click={move || quantity.update(|value| *value += 1)}>
                "Add one"
            </Button>
        </FlexBox>
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

    view! { <TextSpan>"Session active"</TextSpan> }
}
```

### Batching

```rust
#[component]
fn Profile() -> impl IntoNode {
    let name = state(String::new());
    let email = state(String::new());

    view! {
        <FlexBox>
            <TextInput value=name placeholder="Name" />
            <TextInput value=email placeholder="Email" />
            <Button on_click={move || batch(|| {
                name.set(String::new());
                email.set(String::new());
            })}>
                "Clear"
            </Button>
        </FlexBox>
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
    view! { <TextSpan>{props.count.get().to_string()}</TextSpan> }
}

#[component]
fn CounterPanel() -> impl IntoNode {
    let count = state(0);

    view! {
        <FlexBox>
            <CounterValue count=count />
            <Button on_click={move || count.update(|value| *value += 1)}>
                "Increment"
            </Button>
        </FlexBox>
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
        <FlexBox gap={if props.compact { 0 } else { 1 }}>
            <TextSpan>{format!("Hello, {}", props.name.get())}</TextSpan>
            {#if let Some(subtitle) = &props.subtitle}
                <TextSpan>{subtitle.clone()}</TextSpan>
            {/if}
        </FlexBox>
    }
}

#[component]
fn Greetings() -> impl IntoNode {
    let name = state(String::from("Ada"));

    view! {
        <FlexBox>
            <Greeting name=name />
            <Greeting name=name subtitle=Some(String::from("Welcome")) compact=true />
        </FlexBox>
    }
}
```

### Children & Snippets

Component children and snippets share a unified syntax via [`Snippet`](file:///Users/themosthigh/Projects/experimental/sveltui/crates/sveltui-element/src/snippet.rs), governed by the **One-Way Rule**:
- **Primary Children (1 Way)**: Must be placed directly inside the element body (`<Component> ... </Component>`). Passing `children=...` as a prop attribute or declaring `{#snippet children}` is prohibited and rejected at compile time.
- **Custom Named Slots (1 Way)**: Must be declared via block macro syntax inside the element body (`{#snippet slot_name} ... {/snippet}`). Passing snippets as regular prop attributes is prohibited.
- **Explicit Snippet Attributes**:
  - Required slots are declared with `#[snippet]`, e.g. `#[snippet] pub children: Snippet` or `#[snippet] header: Snippet<T>`.
  - Optional slots are declared with `#[snippet(optional)]`, e.g. `#[snippet(optional)] pub children: Option<Snippet>` or `#[snippet(optional)] footer: Option<Snippet<T>>`.
- Use `.is_snippet_present()` to check whether an optional snippet contains renderable content (safely accounting for empty snippets).
- Use `{@render props.children}` or `{@render props.slot(arg)}` to render directly.

```rust
#[derive(Props)]
struct DialogProps {
    #[snippet]
    header: Snippet,
    #[snippet(optional)]
    footer: Option<Snippet>,
    #[snippet]
    pub children: Snippet,
}

#[component]
fn Dialog(props: DialogProps) -> impl IntoNode {
    view! {
        <FlexBox border=BorderStyle::Double padding=Padding::all(1)>
            <FlexBox>{@render props.header}</FlexBox>
            <Divider />
            <FlexBox>{@render props.children}</FlexBox>
            {#if let Some(footer) = &props.footer}
                <Divider />
                <FlexBox>{@render footer}</FlexBox>
            {/if}
        </FlexBox>
    }
}

#[component]
fn ConfirmDelete() -> impl IntoNode {
    view! {
        <Dialog>
            {#snippet header}
                <TextSpan>"Confirm Delete"</TextSpan>
            {/snippet}

            <TextSpan>"Are you sure you want to delete this item?"</TextSpan>

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
    #[snippet]
    row: Snippet<T>,
}

#[component]
fn ListView<T: Clone + 'static>(props: ListViewProps<T>) -> impl IntoNode {
    view! {
        <FlexBox>
            {#each props.items.get() as item}
                {@render props.row(item)}
            {/each}
        </FlexBox>
    }
}

#[component]
fn UserDirectory() -> impl IntoNode {
    let users = state(vec![String::from("Ada"), String::from("Grace")]);

    view! {
        <ListView items=users>
            {#snippet row(user)}
                <TextSpan>{user}</TextSpan>
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
        <FlexBox>
            {#if props.connected.get()}
                <Badge label="Online" />
            {:else if props.retries.get() > 3}
                <TextSpan>"Connection failed"</TextSpan>
            {:else}
                <TextSpan>{format!("Retrying ({})...", props.retries.get())}</TextSpan>
            {/if}
        </FlexBox>
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
        <FlexBox>
            {#each props.tasks.get() as task (task.id)}
                <FlexBox flex_direction=FlexDirection::Row gap=1>
                    <Checkbox checked=task.done />
                    <TextSpan>{task.title.clone()}</TextSpan>
                </FlexBox>
            {:else}
                <TextSpan>"No tasks remaining"</TextSpan>
            {/each}
        </FlexBox>
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
        <FlexBox>
            <TextSpan>{props.value.get().to_string()}</TextSpan>
            <Button on_click={move || props.value.update(|current| *current += 1)}>
                "+"
            </Button>
        </FlexBox>
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
    FlexBoxElement::new()
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
        <ContextProvider value=session>
            <SessionName />
        </ContextProvider>
    }
}

#[component]
fn SessionName() -> impl IntoNode {
    let session = use_context::<SessionContext>().expect("Session provider required");

    view! {
        <FlexBox>
            <TextSpan>{session.username.get()}</TextSpan>
            <TextInput value=session.username />
        </FlexBox>
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
        <TextSpan>
            {props
                .input
                .get()
                .parse::<i32>()
                .expect("Invalid number")
                .to_string()}
        </TextSpan>
    }
}

#[component]
fn NumberEditor() -> impl IntoNode {
    let input = state(String::from("42"));

    view! {
        <FlexBox>
            <TextInput value=input />
            <ErrorBoundary>
                {#snippet fallback(error)}
                    <TextSpan>{format!("Component crashed: {error}")}</TextSpan>
                {/snippet}

                <ParsedNumber input=input />
            </ErrorBoundary>
        </FlexBox>
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
        <FlexBox>
            <TextInput value=path />
            {#await load_document(path.get())}
                <TextSpan>"Loading..."</TextSpan>
            {:then text}
                <TextSpan>{text}</TextSpan>
            {:catch error}
                <TextSpan>{format!("Could not load document: {error}")}</TextSpan>
            {/await}
        </FlexBox>
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

    view! { <TextSpan>{status.get()}</TextSpan> }
}
```
