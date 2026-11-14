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

#[component]
fn IndexedList() -> impl IntoNode {
    let ids = ROWS.with(|rows| rows.borrow().clone());
    view! {
        <View>
            {#each ids as id, i (id)}
                <Row id={id * 10 + i as u32} />
            {/each}
        </View>
    }
}

#[component]
fn Keyed() -> impl IntoNode {
    // The first row's id doubles as the key.
    let page = ROWS.with(|rows| rows.borrow()[0]);
    view! {
        {#key page}
            {@const id = 7}
            {#snippet pair(a: u32, b: u32)}
                <Row id={a + b} />
            {/snippet}
            {@render pair(id, 1)}
            <TextSpan>$"page {page} of {id:>3}"</TextSpan>
        {/key}
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

#[test]
fn each_binds_the_row_index_next_to_a_key() {
    ROWS.with(|rows| *rows.borrow_mut() = vec![1, 2, 3]);
    let seen = frame(|| IndexedList(IndexedListProps::builder().build()).into_node());
    let ids: Vec<u32> = seen.iter().map(|(id, _)| *id).collect();
    assert_eq!(ids, vec![10, 21, 32]);
}

#[test]
fn key_blocks_reset_state_when_the_value_changes() {
    ROWS.with(|rows| *rows.borrow_mut() = vec![1]);
    let run = || frame(|| Keyed(KeyedProps::builder().build()).into_node());
    assert_eq!(run(), vec![(8, 1)]);
    assert_eq!(run(), vec![(8, 2)]);
    ROWS.with(|rows| *rows.borrow_mut() = vec![2]);
    assert_eq!(run(), vec![(8, 1)]);
}
