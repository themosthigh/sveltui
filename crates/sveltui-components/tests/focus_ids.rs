use std::cell::RefCell;
use sveltui::prelude::*;

thread_local! {
    static ROWS: RefCell<Vec<u32>> = const { RefCell::new(Vec::new()) };
    static SEEN: RefCell<Vec<(u32, FocusId)>> = const { RefCell::new(Vec::new()) };
}

#[derive(Props)]
struct RowProps {
    n: u32,
}

/// Asks for an id the way a control that was not given one does.
#[component]
fn Row(props: RowProps) -> impl IntoNode {
    let id = FocusId::default();
    SEEN.with(|seen| seen.borrow_mut().push((props.n, id)));
    view! { <TextSpan>{props.n.to_string()}</TextSpan> }
}

#[component]
fn List() -> impl IntoNode {
    let rows = ROWS.with(|rows| rows.borrow().clone());
    view! {
        <View>
            {#each rows as n (n)}
                <Row n={n} />
            {/each}
        </View>
    }
}

fn frame() -> Vec<(u32, FocusId)> {
    SEEN.with(|seen| seen.borrow_mut().clear());
    begin_frame();
    let _ = List(ListProps::builder().build()).into_node();
    end_frame();
    SEEN.with(|seen| seen.borrow().clone())
}

#[test]
fn an_id_follows_its_scope_not_the_order_views_were_built_in() {
    ROWS.with(|rows| *rows.borrow_mut() = vec![1, 2, 3]);
    let before = frame();
    assert_eq!(frame(), before, "ids repeat from frame to frame");

    ROWS.with(|rows| rows.borrow_mut().remove(0));
    let after = frame();
    for (n, id) in &after {
        let was = before.iter().find(|(m, _)| m == n).unwrap().1;
        assert_eq!(*id, was, "row {n} keeps its id when row 1 goes");
    }
}

#[test]
fn rows_never_share_an_id() {
    ROWS.with(|rows| *rows.borrow_mut() = vec![1, 2, 3, 4]);
    let seen = frame();
    let mut ids: Vec<_> = seen.iter().map(|(_, id)| *id).collect();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), seen.len());
}
