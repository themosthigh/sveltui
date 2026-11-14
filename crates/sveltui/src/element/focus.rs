use crate::element::interaction::InteractionTree;
use crate::layout::Rect;
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
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

/// Ids made inside a scope sit between the explicit ones (low) and the
/// allocated ones (top bit): this bit, the scope's id, then the position
/// among that scope's own ids.
const SCOPED_FLAG: usize = 1 << (usize::BITS - 2);
const SCOPED_SLOTS: usize = 1 << 16;

/// An id for a view that did not name one. Inside a scope it follows the
/// scope and the view's place within it, so it stays the same when the
/// views around it come, go or are skipped. Outside any scope there is
/// nothing to hang it on, and it comes from a counter that
/// `reset_focus_counter` restarts.
pub fn next_render_focus_id() -> FocusId {
    if let Some((scope, n)) = crate::reactive::lifecycle::next_scope_focus_slot() {
        return FocusId(SCOPED_FLAG | (scope * SCOPED_SLOTS) | (n % SCOPED_SLOTS));
    }
    RENDER_FOCUS_COUNTER.with(|counter| {
        let id = counter.get();
        counter.set(id + 1);
        FocusId(id)
    })
}

struct ViewContext {
    focused: Option<FocusId>,
    focus_visible: bool,
    ancestors: HashSet<FocusId>,
    open: HashSet<FocusId>,
    anchors: HashMap<FocusId, (u16, u16)>,
    layouts: Rc<HashMap<FocusId, Rect>>,
}
struct RestoreContext(Option<ViewContext>);
impl Drop for RestoreContext {
    fn drop(&mut self) {
        VIEW_CONTEXT.with(|context| *context.borrow_mut() = self.0.take());
    }
}

pub(crate) fn with_context<T>(
    focused: Option<FocusId>,
    focus_visible: bool,
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
            focus_visible,
            ancestors,
            open: open.clone(),
            anchors: anchors.clone(),
            layouts: tree.layouts.clone(),
        }))
    });
    let _restore = RestoreContext(previous);
    f()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct FocusState {
    pub is_focused: bool,
    pub has_focus_within: bool,
    /// Whether focus got where it is from the keyboard rather than a
    /// pointer press, like CSS `:focus-visible`.
    pub focus_visible: bool,
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
    /// Focused, or containing focus, and the focus came from the keyboard.
    /// What should show a focus ring or open a tooltip; a click that
    /// happens to focus a button shouldn't.
    pub fn is_active_visible(&self) -> bool {
        self.is_active() && self.focus_visible
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
                focus_visible: view.focus_visible,
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

/// A reference to a view, for `<TextInput bind:this={field} />`. It holds the
/// view's focus id, which stays the same from frame to frame, so the
/// handlers and the render code can ask about the view or move focus to it.
/// Make one in a component with `node_ref()`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NodeRef {
    id: crate::reactive::State<FocusId>,
}
impl NodeRef {
    pub fn id(&self) -> FocusId {
        self.id.get()
    }
    pub fn state(&self) -> FocusState {
        use_focus(self.id())
    }
    pub fn is_focused(&self) -> bool {
        self.state().is_focused()
    }
    pub fn has_focus_within(&self) -> bool {
        self.state().has_focus_within()
    }
    /// Moves focus to the view once the handler returns. Nothing happens
    /// if the view isn't on screen or can't take focus.
    pub fn focus(&self, ctx: &mut crate::element::interaction::EventContext) {
        ctx.focus(self.id());
    }
}

/// A reference that lives as long as the component, for `bind:this`.
pub fn node_ref() -> NodeRef {
    NodeRef {
        id: crate::reactive::state_with(FocusId::new),
    }
}

/// Where the view with this identity landed in the last frame, in screen
/// cells. `None` before its first paint or outside a bound App.
pub fn use_layout(id: FocusId) -> Option<Rect> {
    VIEW_CONTEXT.with(|context| {
        context
            .borrow()
            .as_ref()
            .and_then(|view| view.layouts.get(&id).copied())
    })
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
