use crate::*;
use sveltui::element::*;
use sveltui::layout::*;
use sveltui::prelude::*;
use sveltui::terminal::Buffer;

#[derive(Props)]
struct CardLayoutProps {
    title: String,
    #[snippet]
    pub children: Snippet,
    #[snippet(optional)]
    actions: Option<Snippet<()>>,
}

#[component]
fn CardLayout(props: CardLayoutProps) -> impl IntoNode {
    view! {
        <View border=BorderStyle::Double padding={Padding::all(1)}>
            <TextSpan bold=true>{props.title.clone()}</TextSpan>
            <Divider />
            <View>{@render props.children}</View>
            {#if props.actions.is_snippet_present()}
                <Divider />
                {@render props.actions}
            {/if}
        </View>
    }
}

#[test]
fn test_multi_slot_card_layout_with_regular_children_and_named_snippet() {
    let node = view! {
        <CardLayout title="Delete Item">
            <TextSpan>"Are you sure you want to delete this resource?"</TextSpan>
            <TextSpan>"This cannot be undone."</TextSpan>

            {#snippet actions}
                <Button label="Cancel" />
                <Button label="Confirm" />
            {/snippet}
        </CardLayout>
    };

    assert!(matches!(node, VNode::View(_)));
}

#[test]
fn test_whitespace_only_children_does_not_suppress_label_fallback() {
    // Blank string child should be treated as empty, allowing label to take effect
    let btn = view! {
        <Button label="Fallback Label">"   "</Button>
    };

    assert!(matches!(btn, VNode::View(v) if !v.pressables.is_empty()));
}

#[test]
fn test_pressable_event_registration() {
    let id = FocusId::new();
    let node = view! {
        <Pressable focus_id=id tab_stop=true>
            <View width=15 height=3 border=BorderStyle::Single>
                <TextSpan>"Button"</TextSpan>
            </View>
        </Pressable>
    };

    let mut buffer = Buffer::new(30, 10);
    let mut hits = sveltui::element::HitMap::default();
    sveltui::element::render_node_interactive(
        &node,
        Rect::new(0, 0, 30, 10),
        &mut buffer,
        &mut hits,
    );

    assert!(hits.hit_test(2, 1).is_some());
    assert_eq!(hits.hit_test(2, 1).unwrap().focus_id, Some(id));
}

#[test]
fn test_button_measurement() {
    let btn = Button(
        ButtonProps::builder()
            .children(Snippet::from("Click Me"))
            .build(),
    );
    let node = btn.into_node();
    let (w, h) = measure_node(&node, 80, 24);
    assert!(w >= 10);
    assert_eq!(h, 3); // 1 row text + 2 rows border
}
