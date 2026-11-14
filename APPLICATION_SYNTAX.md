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
        <View border=BorderStyle::Single padding=Padding::all(1)>
            <TextSpan>"Hello, sveltui"</TextSpan>
        </View>
    }
}
```

### State

```rust
#[component]
fn Counter() -> impl IntoNode {
    let count = state(0);

    view! {
        <View>
            <TextSpan>{count.get().to_string()}</TextSpan>
            <Button on_click={move || count.update(|value| *value += 1)}>
                "Increment"
            </Button>
            <Button on_click={move || count.set(0)}>"Reset"</Button>
        </View>
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
        <View>
            <TextSpan>{format!("Total: {}", total.get())}</TextSpan>
            <Button on_click={move || quantity.update(|value| *value += 1)}>
                "Add one"
            </Button>
        </View>
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

    view! { <TextInput bind:value=text placeholder="Draft..." /> }
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
        <View>
            <TextInput bind:value=name placeholder="Name" />
            <TextInput bind:value=email placeholder="Email" />
            <Button on_click={move || batch(|| {
                name.set(String::new());
                email.set(String::new());
            })}>
                "Clear"
            </Button>
        </View>
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
        <View>
            <CounterValue count=count />
            <Button on_click={move || count.update(|value| *value += 1)}>
                "Increment"
            </Button>
        </View>
    }
}
```

A value can go without braces only when it is a literal (`width=40`,
`gap=-1`, `bold=true`, `title="Name"`) or a plain or qualified path
(`count=count`, `border=BorderStyle::Single`). Anything else, such as a
call, a field, a method, a tuple, an array, a closure or an operator, needs
braces: `padding={Padding::all(1)}`, `width={props.width}`,
`disabled={a > b}`. The compiler message shows the braced form.

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
        <View gap={if props.compact { 0 } else { 1 }}>
            <TextSpan>{format!("Hello, {}", props.name.get())}</TextSpan>
            {#if let Some(subtitle) = &props.subtitle}
                <TextSpan>{subtitle.clone()}</TextSpan>
            {/if}
        </View>
    }
}

#[component]
fn Greetings() -> impl IntoNode {
    let name = state(String::from("Ada"));

    view! {
        <View>
            <Greeting name=name />
            <Greeting name=name subtitle=Some(String::from("Welcome")) compact=true />
        </View>
    }
}
```

### Two-way Props (`bind:`)

Put `bind:` in front of a prop to let the component write back to your state:

```rust
let email = state(String::new());
let accepted = state(false);

view! {
    <TextInput bind:value={email} placeholder="Email" />
    <Checkbox bind:checked={accepted} label="I accept the terms" />
}
```

Your own components can offer the same thing. Mark the prop `#[prop(bindable)]`
and give it the type `Bindable<T>`; the component reads it with `.value()`
(passed as `count={..}`) or `.bound()` (passed as `bind:count={..}`):

```rust
#[derive(Props)]
struct CounterProps {
    #[prop(bindable)]
    count: Bindable<i32>,
}

#[component]
fn Counter(props: CounterProps) -> impl IntoNode {
    // Count in the parent's state when it's bound; otherwise count locally,
    // starting from `count={..}` if one was passed.
    let local = state(props.count.value().unwrap_or(0));
    let count = props.count.bound().unwrap_or(local);

    view! {
        <View flex_direction=FlexDirection::Row gap=1>
            <Button on_click={move || count.update(|n| *n -= 1)}>"-"</Button>
            <TextSpan>{count.get().to_string()}</TextSpan>
            <Button on_click={move || count.update(|n| *n += 1)}>"+"</Button>
        </View>
    }
}

#[component]
fn Cart() -> impl IntoNode {
    let items = state(1);

    view! {
        <View>
            <Counter bind:count={items} />
            <TextSpan>{format!("{} items in the cart", items.get())}</TextSpan>
        </View>
    }
}
```

### Colors

Color props take crossterm's named colors, or a hex string through `hex_color`
(`#rgb` or `#rrggbb`). A bad hex string panics, and inside a `const` it fails
the build.

```rust
const ACCENT: Color = hex_color("#7aa2f7");

#[component]
fn Banner() -> impl IntoNode {
    view! {
        <View border=BorderStyle::Single border_color=ACCENT>
            <TextSpan foreground_color={hex_color("#f80")}>"Warm"</TextSpan>
            <TextSpan foreground_color=Color::DarkGrey>"Muted"</TextSpan>
        </View>
    }
}
```

### Grid Layout

`display=Display::Grid` lays a view's children out in columns and rows.
Tracks are sized with `length(cells)`, `fr(share)`, `auto()`,
`min_content()`, `max_content()`, `fit_content(cells)`, `minmax(min, max)`
and `repeat(count, [..])`. Children are placed with `grid_column` and
`grid_row`, using `span(n)`, `line(n)` or `lines(start, end)`; without
them they fill the next free cell.

```rust
#[component]
fn Dashboard() -> impl IntoNode {
    view! {
        <View display=Display::Grid width=Dimension::Percent(1.0) height=Dimension::Percent(1.0)
            grid_template_columns={[length(22), fr(1.0)]}
            grid_template_rows={[length(3), fr(1.0)]}
            column_gap=1>
            <View grid_column={span(2)}><TextSpan>"Header"</TextSpan></View>
            <View><TextSpan>"Sidebar"</TextSpan></View>
            // As many 18-cell columns as fit, sharing what's left.
            <View display=Display::Grid row_gap=1
                grid_template_columns={[repeat(Repeat::AutoFill, [minmax(length(18), fr(1.0))])]}>
                <TextSpan>"api"</TextSpan>
                <TextSpan>"worker"</TextSpan>
            </View>
        </View>
    }
}
```

`grid_rules` draws lines in a grid's gaps, joined to each other and to the
border, and broken around cells that span them. `grid_rule_axes` picks rows,
columns or both, and a child with `separator=true` draws a single line across
the grid, such as under a header.

```rust
view! {
    <View display=Display::Grid border=BorderStyle::Rounded grid_rules=BorderStyle::Single
        grid_template_columns={[auto(), fr(1.0)]}>
        <TextSpan>"Name"</TextSpan><TextSpan>"Ada"</TextSpan>
        <TextSpan>"Language"</TextSpan><TextSpan>"Rust"</TextSpan>
    </View>
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
        <View border=BorderStyle::Double padding=Padding::all(1)>
            <View>{@render props.header}</View>
            <Divider />
            <View>{@render props.children}</View>
            {#if let Some(footer) = &props.footer}
                <Divider />
                <View>{@render footer}</View>
            {/if}
        </View>
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
        <View>
            {#each props.items.get() as item}
                {@render props.row(item)}
            {/each}
        </View>
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
        <View>
            {#if props.connected.get()}
                <Badge label="Online" />
            {:else if props.retries.get() > 3}
                <TextSpan>"Connection failed"</TextSpan>
            {:else}
                <TextSpan>{format!("Retrying ({})...", props.retries.get())}</TextSpan>
            {/if}
        </View>
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
        <View>
            {#each props.tasks.get() as task (task.id)}
                <View flex_direction=FlexDirection::Row gap=1>
                    <Checkbox bind:checked={task.done} />
                    <TextSpan>{task.title.clone()}</TextSpan>
                </View>
            {:else}
                <TextSpan>"No tasks remaining"</TextSpan>
            {/each}
        </View>
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
        <View>
            <TextSpan>{props.value.get().to_string()}</TextSpan>
            <Button on_click={move || props.value.update(|current| *current += 1)}>
                "+"
            </Button>
        </View>
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
    ViewElement::new()
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
        <View>
            <TextSpan>{session.username.get()}</TextSpan>
            <TextInput bind:value={session.username} />
        </View>
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
        <View>
            <TextInput bind:value=input />
            <ErrorBoundary>
                {#snippet fallback(error)}
                    <TextSpan>{format!("Component crashed: {error}")}</TextSpan>
                {/snippet}

                <ParsedNumber input=input />
            </ErrorBoundary>
        </View>
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
        <View>
            <TextInput bind:value=path />
            {#await load_document(path.get())}
                <TextSpan>"Loading..."</TextSpan>
            {:then text}
                <TextSpan>{text}</TextSpan>
            {:catch error}
                <TextSpan>{format!("Could not load document: {error}")}</TextSpan>
            {/await}
        </View>
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
