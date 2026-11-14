use sveltui::prelude::*;
use sveltui_components::*;

#[derive(Clone, Debug, PartialEq)]
struct UserItem {
    name: String,
    role: String,
}

#[derive(Props)]
struct UserTableProps<T: 'static> {
    title: String,
    items: Vec<T>,
    #[snippet]
    row: Snippet<T>,
    #[snippet(optional)]
    header: Option<Snippet<()>>,
    #[snippet(optional)]
    footer: Option<Snippet<()>>,
}

#[component]
fn UserTable<T: Clone + 'static>(props: UserTableProps<T>) -> impl IntoNode {
    view! {
        <View border=BorderStyle::Single padding={Padding::all(1)} gap=1>
            <TextSpan bold=true>{props.title.clone()}</TextSpan>
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
        </View>
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
                <TextSpan>"Active Members"</TextSpan>
            {/snippet}

            {#snippet row(u: UserItem)}
                <View flex_direction=FlexDirection::Row gap=2>
                    <TextSpan>{u.name.clone()}</TextSpan>
                    <TextSpan>{u.role.clone()}</TextSpan>
                </View>
            {/snippet}

            {#snippet footer}
                <TextSpan>"End of List"</TextSpan>
            {/snippet}
        </UserTable>
    };

    assert!(matches!(node, VNode::View(_)));
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
                <TextSpan>{u.name.clone()}</TextSpan>
            {/snippet}
        </UserTable>
    };

    assert!(matches!(node, VNode::View(_)));
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
