use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    static CONTEXT_STACK: RefCell<HashMap<TypeId, Vec<Box<dyn Any>>>> = RefCell::new(HashMap::new());
}

pub fn provide_context<T: 'static, R>(value: T, f: impl FnOnce() -> R) -> R {
    let type_id = TypeId::of::<T>();
    CONTEXT_STACK.with(|stack| {
        stack
            .borrow_mut()
            .entry(type_id)
            .or_default()
            .push(Box::new(value));
    });
    let result = f();
    CONTEXT_STACK.with(|stack| {
        if let Some(list) = stack.borrow_mut().get_mut(&type_id) {
            list.pop();
        }
    });
    result
}

pub fn use_context<T: Clone + 'static>() -> Option<T> {
    let type_id = TypeId::of::<T>();
    CONTEXT_STACK.with(|stack| {
        stack.borrow().get(&type_id).and_then(|list| {
            list.last()
                .and_then(|boxed| boxed.downcast_ref::<T>())
                .cloned()
        })
    })
}
