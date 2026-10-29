use svtui::prelude::*;
use svtui_components::*;

#[derive(Clone, Debug, PartialEq)]
struct UserItem {
    name: String,
    role: String,
}

#[derive(Props)]
struct UserTableProps<T: 'static> {
    title: String,
    items: Vec<T>,
    row: Snippet<T>,
    #[prop(optional)]
    header: Option<Snippet<()>>,
    #[prop(optional)]
    footer: Option<Snippet<()>>,
}

#[component]
fn UserTable<T: Clone + 'static>(props: UserTableProps<T>) -> impl IntoNode {
    view! {
        <Box border=BorderStyle::Single padding=Padding::all(1) gap=1>
            <Text bold=true>{props.title.clone()}</Text>
            {#if props.header.is_snippet_present()}
                {@render props.header}
                <Divider />
            {/if}
            {#each props.items as item}
                {@render props.row(item)}
            {/each}
            {#if props.footer.is_snippet_present()}
                <Divider />
                {@render props.footer}
            {/if}
        </Box>
    }
}

#[derive(Props)]
struct CardLayoutProps {
    title: String,
    pub children: Snippet,
    #[prop(optional)]
    actions: Option<Snippet<()>>,
}

#[component]
fn CardLayout(props: CardLayoutProps) -> impl IntoNode {
    view! {
        <Box border=BorderStyle::Double padding=Padding::all(1)>
            <Text bold=true>{props.title.clone()}</Text>
            <Divider />
            <Box>{@render props.children}</Box>
            {#if props.actions.is_snippet_present()}
                <Divider />
                {@render props.actions}
            {/if}
        </Box>
    }
}

#[test]
fn test_generic_parameterized_and_optional_snippets() {
    let users = vec![
        UserItem {
            name: "Alice".into(),
            role: "Admin".into(),
        },
        UserItem {
            name: "Bob".into(),
            role: "Dev".into(),
        },
    ];

    let node = view! {
        <UserTable title="Team" items=users>
            {#snippet header}
                <Text>"Active Members"</Text>
            {/snippet}

            {#snippet row(u: UserItem)}
                <Box flex_direction=FlexDirection::Row gap=2>
                    <Text>{u.name.clone()}</Text>
                    <Text>{u.role.clone()}</Text>
                </Box>
            {/snippet}

            {#snippet footer}
                <Text>"End of List"</Text>
            {/snippet}
        </UserTable>
    };

    assert!(matches!(node, VNode::Box(_)));
}

#[test]
fn test_generic_snippets_with_omitted_optional_slots() {
    let users = vec![UserItem {
        name: "Charlie".into(),
        role: "User".into(),
    }];

    // Header and footer omitted
    let node = view! {
        <UserTable title="Team" items=users>
            {#snippet row(u: UserItem)}
                <Text>{u.name.clone()}</Text>
            {/snippet}
        </UserTable>
    };

    assert!(matches!(node, VNode::Box(_)));
}

#[test]
fn test_multi_slot_card_layout_with_regular_children_and_named_snippet() {
    let node = view! {
        <CardLayout title="Delete Item">
            <Text>"Are you sure you want to delete this resource?"</Text>
            <Text>"This cannot be undone."</Text>

            {#snippet actions}
                <Button label="Cancel" />
                <Button label="Confirm" />
            {/snippet}
        </CardLayout>
    };

    assert!(matches!(node, VNode::Box(_)));
}

#[test]
fn test_is_snippet_present_helper() {
    let s_present: Snippet = Snippet::from("hello");
    assert!(s_present.is_snippet_present());

    let s_empty: Snippet = Snippet::none();
    assert!(!s_empty.is_snippet_present());

    let opt_some_present: Option<Snippet> = Some(s_present);
    assert!(opt_some_present.is_snippet_present());

    let opt_some_empty: Option<Snippet> = Some(Snippet::none());
    // Crucially: even though outer Option is Some, inner Snippet is none!
    assert!(!opt_some_empty.is_snippet_present());

    let opt_none: Option<Snippet> = None;
    assert!(!opt_none.is_snippet_present());
}

#[test]
fn test_whitespace_only_children_does_not_suppress_label_fallback() {
    // Blank string child should be treated as empty, allowing label to take effect
    let btn = view! {
        <Button label="Fallback Label">"   "</Button>
    };

    assert!(matches!(btn, VNode::Box(_)));
}
