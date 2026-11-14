use sveltui::prelude::*;

#[derive(Props)]
#[allow(dead_code)]
struct Card {
    title: String,
    #[prop(default = 3)]
    width: u16,
    #[prop(optional)]
    subtitle: Option<String>,
    #[snippet]
    children: Snippet,
}

#[derive(Props)]
struct Labeled<T: Clone + 'static> {
    value: T,
}

#[test]
fn build_exists_once_every_required_prop_is_set() {
    let card = Card::builder()
        .title("Hi")
        .children(Snippet::new(|_| VNode::Empty))
        .build();
    assert_eq!(card.title, "Hi");
    assert_eq!(card.width, 3);
    assert!(card.subtitle.is_none());
}

#[test]
fn required_props_can_be_set_in_any_order_and_more_than_once() {
    let card = Card::builder()
        .children(Snippet::new(|_| VNode::Empty))
        .subtitle("sub")
        .title("a")
        .title("b")
        .width(9)
        .build();
    assert_eq!(card.title, "b");
    assert_eq!(card.width, 9);
    assert_eq!(card.subtitle.as_deref(), Some("sub"));
}

#[test]
fn generic_props_keep_their_parameters() {
    let labeled = Labeled::<u8>::builder().value(4u8).build();
    assert_eq!(labeled.value, 4);
}
