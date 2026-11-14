use sveltui::prelude::*;

#[derive(Props)]
struct FieldProps {
    #[prop(bindable)]
    value: Bindable<String>,
}

fn passed_on(props: FieldProps) -> Bindable<String> {
    FieldProps::builder().bind_value(props.value).build().value
}

#[test]
fn a_state_binds() {
    let name = state(String::from("a"));
    let field = FieldProps::builder().bind_value(name).build();
    assert_eq!(field.value.bound(), Some(name));
    assert!(field.value.value().is_none());
}

#[test]
fn an_optional_state_binds_or_leaves_it_unbound() {
    let none: Option<State<String>> = None;
    let field = FieldProps::builder().bind_value(none).build();
    assert!(field.value.bound().is_none());
}

#[test]
fn a_bindable_passes_its_state_on() {
    let name = state(String::from("a"));
    let parent = FieldProps::builder().bind_value(name).build();
    assert_eq!(passed_on(parent).bound(), Some(name));
}

#[test]
fn a_bindable_passes_its_read_only_value_on() {
    let parent = FieldProps::builder().value("fixed").build();
    let child = passed_on(parent);
    assert_eq!(child.value().as_deref(), Some("fixed"));
    assert!(child.bound().is_none());
}

#[test]
fn a_borrowed_bindable_is_cloned() {
    let parent = FieldProps::builder().value("fixed").build();
    let child = FieldProps::builder().bind_value(&parent.value).build();
    assert_eq!(child.value.value().as_deref(), Some("fixed"));
    assert_eq!(parent.value.value().as_deref(), Some("fixed"));
}

#[derive(Props)]
struct EchoProps {
    #[prop(bindable)]
    value: Bindable<String>,
    #[prop(optional)]
    on_value_change: Option<std::rc::Rc<dyn Fn(String)>>,
}

#[component]
fn Echo(props: EchoProps) -> impl IntoNode {
    if let Some(on_change) = &props.on_value_change {
        on_change(format!("{}!", props.value.value().unwrap_or_default()));
    }
    view! { <TextSpan>"echo"</TextSpan> }
}

#[component]
fn FunctionBound() -> impl IntoNode {
    let name = state(String::from("hi"));
    let seen = state(String::new());
    let node = view! {
        <Echo bind:value={(move || name.get(), move |next: String| seen.set(next))} />
    };
    assert_eq!(seen.get(), "hi!");
    node
}

#[test]
fn function_bindings_read_and_report_through_callbacks() {
    begin_frame();
    let _ = FunctionBound(FunctionBoundProps::builder().build());
    end_frame();
}

#[derive(Props, Clone)]
struct BadgeProps {
    label: String,
    #[prop(default = 1)]
    size: u16,
    #[prop(optional)]
    hint: Option<String>,
    #[prop(bindable)]
    value: Bindable<String>,
}

#[component]
fn Badge(props: BadgeProps) -> impl IntoNode {
    view! { <TextSpan>{format!("{}:{}:{:?}", props.label, props.size, props.hint)}</TextSpan> }
}

#[component]
fn Wrapper(props: BadgeProps) -> impl IntoNode {
    view! { <Badge {..props} size=5 /> }
}

#[test]
fn spread_forwards_props_and_later_ones_win() {
    let props = BadgeProps::builder().label("a").hint("h").build();
    let built = BadgeProps::builder().spread(props).size(7).build();
    assert_eq!(built.label, "a");
    assert_eq!(built.size, 7);
    assert_eq!(built.hint.as_deref(), Some("h"));
    // An unset optional prop in the spread keeps an earlier value.
    let plain = BadgeProps::builder().label("b").build();
    let merged = BadgeProps::builder()
        .label("x")
        .hint("keep")
        .spread(plain)
        .build();
    assert_eq!(merged.label, "b");
    assert_eq!(merged.hint.as_deref(), Some("keep"));
}

#[test]
fn spread_works_inside_a_view() {
    begin_frame();
    let props = BadgeProps::builder().label("a").build();
    let _ = Wrapper(props);
    end_frame();
}
