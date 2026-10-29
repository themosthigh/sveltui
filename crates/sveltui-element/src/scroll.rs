use crate::focus::FocusId;
use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    static SCROLL_MAP: RefCell<HashMap<FocusId, (u16, u16)>> = RefCell::new(HashMap::new());
}

pub fn get_scroll(id: FocusId) -> (u16, u16) {
    SCROLL_MAP.with(|m| *m.borrow().get(&id).unwrap_or(&(0, 0)))
}

pub fn set_scroll(id: FocusId, pos: (u16, u16)) {
    SCROLL_MAP.with(|m| {
        m.borrow_mut().insert(id, pos);
    });
}
