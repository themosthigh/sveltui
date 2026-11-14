use sveltui::prelude::*;

thread_local! {
    static ROWS: std::cell::RefCell<Vec<u32>> = const { std::cell::RefCell::new(Vec::new()) };
    static LOG: std::cell::RefCell<Vec<(u32, u32)>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// A row that counts how many frames it has been drawn for, in state of
/// its own, and reports `(id, count)`.
#[derive(Props)]
struct RowProps {
    id: u32,
}

#[component]
fn Row(props: RowProps) -> impl IntoNode {
    let id = props.id;
    let frames = state(0u32);
    frames.update(|n| *n += 1);
    LOG.with(|log| log.borrow_mut().push((id, frames.get())));
    view! { <TextSpan>{format!("row {id}")}</TextSpan> }
}

#[component]
fn List() -> impl IntoNode {
    let ids = ROWS.with(|rows| rows.borrow().clone());
    view! {
        <View>
            {#each ids as id (id)}
                <Row id={id} />
            {/each}
        </View>
    }
}

#[component]
fn PositionalList() -> impl IntoNode {
    let ids = ROWS.with(|rows| rows.borrow().clone());
    view! {
        <View>
            {#each ids as id}
                <Row id={id} />
            {/each}
        </View>
    }
}

fn frame(root: fn() -> VNode) -> Vec<(u32, u32)> {
    LOG.with(|log| log.borrow_mut().clear());
    begin_frame();
    let _ = root();
    end_frame();
    LOG.with(|log| log.borrow().clone())
}

#[test]
fn removing_the_first_keyed_row_keeps_the_others_state() {
    ROWS.with(|rows| *rows.borrow_mut() = vec![1, 2, 3]);
    assert_eq!(
        frame(|| List(ListProps::builder().build()).into_node()),
        vec![(1, 1), (2, 1), (3, 1)]
    );
    assert_eq!(
        frame(|| List(ListProps::builder().build()).into_node()),
        vec![(1, 2), (2, 2), (3, 2)]
    );
    ROWS.with(|rows| rows.borrow_mut().remove(0));
    assert_eq!(
        frame(|| List(ListProps::builder().build()).into_node()),
        vec![(2, 3), (3, 3)]
    );
}

#[test]
fn a_removed_key_comes_back_with_fresh_state() {
    ROWS.with(|rows| *rows.borrow_mut() = vec![1, 2]);
    frame(|| List(ListProps::builder().build()).into_node());
    ROWS.with(|rows| *rows.borrow_mut() = vec![2]);
    frame(|| List(ListProps::builder().build()).into_node());
    ROWS.with(|rows| *rows.borrow_mut() = vec![1, 2]);
    assert_eq!(
        frame(|| List(ListProps::builder().build()).into_node()),
        vec![(1, 1), (2, 3)]
    );
}

#[test]
fn unkeyed_rows_follow_position() {
    ROWS.with(|rows| *rows.borrow_mut() = vec![1, 2, 3]);
    frame(|| PositionalList(PositionalListProps::builder().build()).into_node());
    ROWS.with(|rows| rows.borrow_mut().remove(0));
    // Row 2 now sits where row 1 was, so it inherits row 1's count.
    assert_eq!(
        frame(|| PositionalList(PositionalListProps::builder().build()).into_node()),
        vec![(2, 2), (3, 2)]
    );
}
