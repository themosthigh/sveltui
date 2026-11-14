use crate::element::dynamic::DynamicSlot;
use crate::element::node::VNode;
use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;

type RenderRow<K, T> = Rc<dyn Fn(&K, &T) -> VNode>;

struct Row<T> {
    item: Rc<RefCell<T>>,
    slot: DynamicSlot,
}

/// A list that keeps one retained `DynamicSlot` per key. Each `update`
/// reconciles against the items it is given:
///
/// - a key we have not seen gets a new row;
/// - a key we have gets its row back, and the row builds again only if its
///   item changed or a signal it read was written, so its state and the
///   work of building it carry over untouched;
/// - a key that is gone has its row dropped, which destroys the scope the
///   row built in and runs its cleanups.
///
/// The rows come back in the order of the items, so moving an item moves
/// its row and nothing else.
pub struct KeyedList<K, T> {
    render: RenderRow<K, T>,
    rows: RefCell<HashMap<K, Row<T>>>,
}

impl<K: Hash + Eq + Clone + 'static, T: Clone + PartialEq + 'static> KeyedList<K, T> {
    pub fn new(render: impl Fn(&K, &T) -> VNode + 'static) -> Self {
        Self {
            render: Rc::new(render),
            rows: RefCell::new(HashMap::new()),
        }
    }

    /// Reconciles the rows against `items` and returns them in order.
    pub fn update(&self, items: impl IntoIterator<Item = (K, T)>) -> VNode {
        let mut rows = self.rows.borrow_mut();
        let mut kept = HashMap::new();
        let mut order = Vec::new();
        for (key, item) in items {
            let row = match rows.remove(&key) {
                Some(row) => {
                    if *row.item.borrow() != item {
                        *row.item.borrow_mut() = item;
                        row.slot.invalidate();
                    }
                    row
                }
                None => {
                    let item = Rc::new(RefCell::new(item));
                    let render = self.render.clone();
                    let (row_key, row_item) = (key.clone(), item.clone());
                    Row {
                        item,
                        slot: DynamicSlot::new(move || render(&row_key, &row_item.borrow())),
                    }
                }
            };
            order.push(VNode::Dynamic(row.slot.clone()));
            // A repeated key keeps its first row, as keyed scopes do.
            kept.entry(key).or_insert(row);
        }
        // Whatever is left in `rows` was not asked for, and is dropped here.
        *rows = kept;
        VNode::Fragment(order)
    }

    /// How many rows are alive.
    pub fn len(&self) -> usize {
        self.rows.borrow().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
