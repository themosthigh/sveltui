use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use svtui::prelude::*;
use svtui_components::*;

// Section 1: Application Setup
#[component]
fn Application() -> impl IntoNode {
    view! {
        <FlexBox border=BorderStyle::Single padding=Padding::all(1)>
            <TextSpan>"Hello, svtui"</TextSpan>
        </FlexBox>
    }
}

#[test]
fn test_section_1_application_setup() {
    let node = view! { <Application /> };
    assert!(matches!(node, VNode::FlexBox(_)));
}

// Section 2: State
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

#[test]
fn test_section_2_state() {
    let count = state(0);
    assert_eq!(count.get(), 0);
    count.update(|value| *value += 1);
    assert_eq!(count.get(), 1);
    count.set(0);
    assert_eq!(count.get(), 0);

    let node = view! { <Counter /> };
    assert!(matches!(node, VNode::FlexBox(_)));
}

// Section 3: Derived State
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

#[test]
fn test_section_3_derived_state() {
    let quantity = state(1);
    let unit_price = state(25);
    let total = derived(move || quantity.get() * unit_price.get());
    assert_eq!(total.get(), 25);
    quantity.update(|v| *v += 1);
    assert_eq!(total.get(), 50);

    let node = view! { <Total /> };
    assert!(matches!(node, VNode::FlexBox(_)));
}

// Section 4: Effects
#[test]
fn test_section_4_effects() {
    let text = state(String::from("init"));
    let recorded = Arc::new(std::sync::Mutex::new(String::new()));
    let rec = recorded.clone();

    effect(move || {
        let value = text.get();
        *rec.lock().unwrap() = value;
    });

    assert_eq!(*recorded.lock().unwrap(), "init");
    text.set("updated".into());
    assert_eq!(*recorded.lock().unwrap(), "updated");
}

// Section 5: Mount & Cleanup
#[test]
fn test_section_5_mount_and_cleanup() {
    let mounted = Arc::new(AtomicUsize::new(0));
    let cleaned = Arc::new(AtomicUsize::new(0));

    let m = mounted.clone();
    let c = cleaned.clone();

    let ((), scope) = ComponentScope::run(move || {
        on_mount(move || {
            m.fetch_add(1, Ordering::SeqCst);
            move || {
                c.fetch_add(1, Ordering::SeqCst);
            }
        });

        on_destroy(|| {
            // Clean up resources
        });
    });

    assert_eq!(mounted.load(Ordering::SeqCst), 1);
    assert_eq!(cleaned.load(Ordering::SeqCst), 0);
    scope.run_cleanups();
    assert_eq!(cleaned.load(Ordering::SeqCst), 1);
}

// Section 6: Batching
#[test]
fn test_section_6_batching() {
    let name = state(String::new());
    let email = state(String::new());
    let runs = Arc::new(AtomicUsize::new(0));

    let r = runs.clone();
    effect(move || {
        let _ = (name.get(), email.get());
        r.fetch_add(1, Ordering::SeqCst);
    });

    assert_eq!(runs.load(Ordering::SeqCst), 1);

    batch(|| {
        name.set("Ada".into());
        email.set("ada@lovelace.org".into());
    });

    // Batched updates only execute the effect once
    assert_eq!(runs.load(Ordering::SeqCst), 2);
    assert_eq!(name.get(), "Ada");
    assert_eq!(email.get(), "ada@lovelace.org");
}

// Section 7: Props
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

#[test]
fn test_section_7_props() {
    let count = state(42);
    let node = view! { <CounterValue count=count /> };
    if let VNode::Text(t) = node {
        assert_eq!(t.content, "42");
    } else {
        panic!("Expected Text node");
    }

    let panel = view! { <CounterPanel /> };
    assert!(matches!(panel, VNode::FlexBox(_)));
}

// Section 8: Optional & Default Props
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

#[test]
fn test_section_8_optional_default_props() {
    let name = state(String::from("Ada"));

    let g1 = view! { <Greeting name=name /> };
    assert!(matches!(g1, VNode::FlexBox(_)));

    let g2 = view! {
        <Greeting name=name subtitle=Some(String::from("Welcome")) compact=true />
    };
    assert!(matches!(g2, VNode::FlexBox(_)));

    let all = view! { <Greetings /> };
    assert!(matches!(all, VNode::FlexBox(_)));
}

// Section 9: Children & Snippets
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

#[test]
fn test_section_9_children_and_snippets() {
    let node = view! { <ConfirmDelete /> };
    assert!(matches!(node, VNode::FlexBox(_)));
}

// Section 10: Parameterized Snippets
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

#[test]
fn test_section_10_parameterized_snippets() {
    let node = view! { <UserDirectory /> };
    assert!(matches!(node, VNode::FlexBox(_)));
}

// Section 11: Conditionals
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

#[test]
fn test_section_11_conditionals() {
    let connected = state(true);
    let retries = state(0);
    let node = view! { <ConnectionStatus connected=connected retries=retries /> };
    assert!(matches!(node, VNode::FlexBox(_)));

    connected.set(false);
    retries.set(5);
    let node2 = view! { <ConnectionStatus connected=connected retries=retries /> };
    assert!(matches!(node2, VNode::FlexBox(_)));

    retries.set(1);
    let node3 = view! { <ConnectionStatus connected=connected retries=retries /> };
    assert!(matches!(node3, VNode::FlexBox(_)));
}

// Section 12: Loops
#[derive(Clone)]
#[allow(dead_code)]
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

#[test]
fn test_section_12_loops() {
    let tasks = state(vec![Task {
        id: 1,
        title: "Test".into(),
        done: state(false),
    }]);

    let node = view! { <TaskList tasks=tasks /> };
    assert!(matches!(node, VNode::FlexBox(_)));

    tasks.set(vec![]);
    let empty_node = view! { <TaskList tasks=tasks /> };
    assert!(matches!(empty_node, VNode::FlexBox(_)));
}

// Section 13: Components With Macros vs Without Macros
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

fn stepper(value: State<i32>) -> impl IntoNode {
    FlexBoxElement::new()
        .child(TextElement::new(value.get().to_string()))
        .child(Button(
            ButtonProps::builder()
                .on_click(move || value.update(|current| *current += 1))
                .children("+")
                .build(),
        ))
}

fn stepper_panel() -> impl IntoNode {
    let value = state(0);
    stepper(value)
}

#[test]
fn test_section_13_components_with_and_without_macros() {
    let with_macro = view! { <StepperPanel /> };
    assert!(matches!(with_macro, VNode::FlexBox(_)));

    let without_macro = stepper_panel().into_node();
    assert!(matches!(without_macro, VNode::FlexBox(_)));
}

// Section 14: Context Providers
#[derive(Clone, Copy, PartialEq, Debug)]
struct SessionContext {
    username: State<String>,
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

#[test]
fn test_section_14_context_providers() {
    let node = view! { <SessionProvider /> };
    assert!(matches!(node, VNode::FlexBox(_)));
}

// Section 15: Error Boundaries
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

#[test]
fn test_section_15_error_boundaries() {
    let valid_editor = view! { <NumberEditor /> };
    assert!(matches!(valid_editor, VNode::FlexBox(_)));

    // Test panic catching with bad input
    let bad_input = state(String::from("not_a_number"));
    let caught = view! {
        <ErrorBoundary>
            {#snippet fallback(error)}
                <TextSpan>{format!("Caught error: {error}")}</TextSpan>
            {/snippet}

            <ParsedNumber input=bad_input />
        </ErrorBoundary>
    };

    if let VNode::Text(t) = caught {
        assert!(t.content.contains("Caught error:"));
        assert!(t.content.contains("Invalid number"));
    } else {
        panic!("Expected fallback text node");
    }
}

// Section 16: Async & Await
async fn load_document(path: String) -> std::io::Result<String> {
    spawn_blocking(move || {
        if path == "valid.txt" {
            Ok(String::from("Document content"))
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "File not found",
            ))
        }
    })
    .await
}

#[component]
fn Document() -> impl IntoNode {
    let path = state(String::from("valid.txt"));

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

#[test]
fn test_section_16_async_and_await() {
    let node = view! { <Document /> };
    assert!(matches!(node, VNode::FlexBox(_)));
}

// Section 17: Background Tasks
#[component]
fn BackgroundScan() -> impl IntoNode {
    let status = state(String::from("Scanning..."));

    on_mount(move || {
        let task = spawn_task(async move {
            let result = spawn_blocking(|| 42).await;
            status.set(format!("Found {result} entries"));
        });

        move || task.abort()
    });

    view! { <TextSpan>{status.get()}</TextSpan> }
}

#[test]
fn test_section_17_background_tasks() {
    let node = view! { <BackgroundScan /> };
    assert!(matches!(node, VNode::Text(_)));
}

// Section 18: State Persistence Across Render Passes
static PERSISTENT_SHARED_BUTTON_CLICK: std::sync::Mutex<Option<Box<dyn Fn() + Send + Sync>>> =
    std::sync::Mutex::new(None);

#[component]
fn InteractiveCounter() -> impl IntoNode {
    let count = state(0);
    let mut click_guard = PERSISTENT_SHARED_BUTTON_CLICK.lock().unwrap();
    *click_guard = Some(Box::new(move || {
        count.update(|v| *v += 1);
    }));

    view! {
        <FlexBox>
            <TextSpan>{format!("Count is: {}", count.get())}</TextSpan>
        </FlexBox>
    }
}

#[test]
fn test_state_persistence_across_render_passes() {
    begin_frame();
    let node1 = view! { <InteractiveCounter /> };
    end_frame();

    if let VNode::FlexBox(b) = node1 {
        if let VNode::Text(t) = &b.children[0] {
            assert_eq!(t.content, "Count is: 0");
        } else {
            panic!("Expected text node");
        }
    }

    // Trigger state update from the button click closure
    {
        let click_fn = PERSISTENT_SHARED_BUTTON_CLICK.lock().unwrap();
        (click_fn.as_ref().unwrap())();
    }

    // Render next frame
    begin_frame();
    let node2 = view! { <InteractiveCounter /> };
    end_frame();

    if let VNode::FlexBox(b) = node2 {
        if let VNode::Text(t) = &b.children[0] {
            assert_eq!(t.content, "Count is: 1");
        } else {
            panic!("Expected text node");
        }
    }

    // Trigger another update
    {
        let click_fn = PERSISTENT_SHARED_BUTTON_CLICK.lock().unwrap();
        (click_fn.as_ref().unwrap())();
        (click_fn.as_ref().unwrap())();
    }

    // Render third frame
    begin_frame();
    let node3 = view! { <InteractiveCounter /> };
    end_frame();

    if let VNode::FlexBox(b) = node3 {
        if let VNode::Text(t) = &b.children[0] {
            assert_eq!(t.content, "Count is: 3");
        } else {
            panic!("Expected text node");
        }
    }
}

#[derive(Props)]
struct InstanceCounterProps {
    initial: i32,
}

#[component]
fn InstanceCounter(props: InstanceCounterProps) -> (State<i32>, VNode) {
    let count = state(props.initial);
    let vnode = view! { <TextSpan>{format!("Val: {}", count.get())}</TextSpan> };
    (count, vnode)
}

#[test]
fn test_multiple_component_instances_have_independent_state() {
    begin_frame();
    let (s1, n1) = InstanceCounter(InstanceCounterProps { initial: 10 });
    let (_s2, n2) = InstanceCounter(InstanceCounterProps { initial: 20 });
    end_frame();

    if let VNode::Text(t1) = n1 {
        assert_eq!(t1.content, "Val: 10");
    }
    if let VNode::Text(t2) = n2 {
        assert_eq!(t2.content, "Val: 20");
    }

    // Mutate instance 1
    s1.update(|v| *v += 5);

    // Frame 2
    begin_frame();
    let (_s1_f2, n1_f2) = InstanceCounter(InstanceCounterProps { initial: 10 });
    let (_s2_f2, n2_f2) = InstanceCounter(InstanceCounterProps { initial: 20 });
    end_frame();

    if let VNode::Text(t1) = n1_f2 {
        assert_eq!(t1.content, "Val: 15");
    }
    if let VNode::Text(t2) = n2_f2 {
        assert_eq!(t2.content, "Val: 20");
    }
}

#[test]
fn test_unmount_and_remount_lifecycle() {
    let mounted_count = Arc::new(AtomicUsize::new(0));
    let cleaned_count = Arc::new(AtomicUsize::new(0));

    let m = mounted_count.clone();
    let c = cleaned_count.clone();

    // Frame 1: render child
    begin_frame();
    let s = ComponentScope::run_scoped("LifecycleItem", || {
        let st = state(100);
        let m_inner = m.clone();
        let c_inner = c.clone();
        on_mount(move || {
            m_inner.fetch_add(1, Ordering::SeqCst);
            move || {
                c_inner.fetch_add(1, Ordering::SeqCst);
            }
        });
        st
    });
    s.set(999);
    end_frame();

    assert_eq!(mounted_count.load(Ordering::SeqCst), 1);
    assert_eq!(cleaned_count.load(Ordering::SeqCst), 0);

    // Frame 2: omit child -> unmounts
    begin_frame();
    // LifecycleItem not called
    end_frame();

    assert_eq!(cleaned_count.load(Ordering::SeqCst), 1);

    // Frame 3: remount child
    begin_frame();
    let s_remount = ComponentScope::run_scoped("LifecycleItem", || state(100));
    end_frame();

    // Fresh state!
    assert_eq!(s_remount.get(), 100);
}
