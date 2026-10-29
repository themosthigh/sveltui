use crate::interaction::InteractionTree;
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};

// Allocated identities use the upper half; frame-local and conventional
// explicit numeric IDs use the lower half. Components can retain allocated IDs
// across frames without colliding with either of those sources.
static USER_FOCUS_COUNTER: AtomicUsize = AtomicUsize::new(1 << (usize::BITS - 1));
thread_local! {
    static VIEW_CONTEXT: RefCell<Option<ViewContext>> = const { RefCell::new(None) };
    static RENDER_FOCUS_COUNTER: Cell<usize> = const { Cell::new(0) };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FocusId(pub usize);
impl FocusId {
    pub fn new() -> Self {
        Self(USER_FOCUS_COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}
impl Default for FocusId {
    fn default() -> Self {
        next_render_focus_id()
    }
}
impl From<usize> for FocusId {
    fn from(value: usize) -> Self {
        Self(value)
    }
}
pub fn reset_focus_counter() {
    RENDER_FOCUS_COUNTER.with(|counter| counter.set(0));
}
pub fn next_render_focus_id() -> FocusId {
    RENDER_FOCUS_COUNTER.with(|counter| {
        let id = counter.get();
        counter.set(id + 1);
        FocusId(id)
    })
}

struct ViewContext {
    focused: Option<FocusId>,
    ancestors: HashSet<FocusId>,
    open: HashSet<FocusId>,
    anchors: HashMap<FocusId, (u16, u16)>,
}
struct RestoreContext(Option<ViewContext>);
impl Drop for RestoreContext {
    fn drop(&mut self) {
        VIEW_CONTEXT.with(|context| *context.borrow_mut() = self.0.take());
    }
}

pub(crate) fn with_context<T>(
    focused: Option<FocusId>,
    tree: &InteractionTree,
    open: &HashSet<FocusId>,
    anchors: &HashMap<FocusId, (u16, u16)>,
    f: impl FnOnce() -> T,
) -> T {
    let ancestors = focused
        .and_then(|id| tree.node(id))
        .map(|node| node.ancestors.iter().copied().collect())
        .unwrap_or_default();
    let previous = VIEW_CONTEXT.with(|context| {
        context.replace(Some(ViewContext {
            focused,
            ancestors,
            open: open.clone(),
            anchors: anchors.clone(),
        }))
    });
    let _restore = RestoreContext(previous);
    f()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct FocusState {
    pub is_focused: bool,
    pub has_focus_within: bool,
}
impl FocusState {
    pub fn is_focused(&self) -> bool {
        self.is_focused
    }
    pub fn has_focus_within(&self) -> bool {
        self.has_focus_within
    }
    pub fn is_active(&self) -> bool {
        self.is_focused || self.has_focus_within
    }
}

/// Read the currently bound App's focus. Outside a manager context it is empty.
pub fn use_focus(id: FocusId) -> FocusState {
    VIEW_CONTEXT.with(|context| {
        context
            .borrow()
            .as_ref()
            .map(|view| FocusState {
                is_focused: view.focused == Some(id),
                has_focus_within: view.ancestors.contains(&id),
            })
            .unwrap_or_default()
    })
}
pub fn is_focused(id: FocusId) -> bool {
    use_focus(id).is_focused()
}
pub fn is_layer_open(id: FocusId) -> bool {
    VIEW_CONTEXT.with(|context| {
        context
            .borrow()
            .as_ref()
            .is_some_and(|view| view.open.contains(&id))
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FocusHandle {
    id: FocusId,
}
impl FocusHandle {
    pub fn new() -> Self {
        Self { id: FocusId::new() }
    }
    pub fn from_id(id: FocusId) -> Self {
        Self { id }
    }
    pub fn id(&self) -> FocusId {
        self.id
    }
    pub fn state(&self) -> FocusState {
        use_focus(self.id)
    }
    pub fn is_focused(&self) -> bool {
        self.state().is_focused()
    }
    pub fn has_focus_within(&self) -> bool {
        self.state().has_focus_within()
    }
    pub fn is_active(&self) -> bool {
        self.state().is_active()
    }
}
impl Default for FocusHandle {
    fn default() -> Self {
        Self::new()
    }
}

/// Last pointer anchor for an open context menu in the currently bound App.
pub fn layer_anchor(id: FocusId) -> Option<(u16, u16)> {
    VIEW_CONTEXT.with(|context| {
        context
            .borrow()
            .as_ref()
            .and_then(|view| view.anchors.get(&id).copied())
    })
}
