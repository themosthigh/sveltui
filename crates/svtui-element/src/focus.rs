use crate::events::HitMap;
use std::cell::{Cell as StdCell, RefCell};
use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};

static USER_FOCUS_COUNTER: AtomicUsize = AtomicUsize::new(100_000);

thread_local! {
    static CURRENT_FOCUS: StdCell<Option<FocusId>> = const { StdCell::new(None) };
    static FOCUS_WITHIN_SET: RefCell<HashSet<FocusId>> = RefCell::new(HashSet::new());
    static RENDER_FOCUS_COUNTER: StdCell<usize> = const { StdCell::new(0) };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FocusId(pub usize);

impl FocusId {
    /// Allocates a globally unique FocusId for application-level state handles.
    pub fn new() -> Self {
        FocusId(USER_FOCUS_COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for FocusId {
    fn default() -> Self {
        next_render_focus_id()
    }
}

impl From<usize> for FocusId {
    fn from(val: usize) -> Self {
        FocusId(val)
    }
}

pub fn reset_focus_counter() {
    RENDER_FOCUS_COUNTER.with(|c| c.set(0));
}

pub fn next_render_focus_id() -> FocusId {
    RENDER_FOCUS_COUNTER.with(|c| {
        let val = c.get();
        c.set(val + 1);
        FocusId(val)
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct FocusState {
    pub is_focused: bool,
    pub has_focus_within: bool,
}

impl FocusState {
    #[inline]
    pub fn is_focused(&self) -> bool {
        self.is_focused
    }

    #[inline]
    pub fn has_focus_within(&self) -> bool {
        self.has_focus_within
    }

    #[inline]
    pub fn is_active(&self) -> bool {
        self.is_focused || self.has_focus_within
    }
}

/// Completely opt-in hook to query focus state (both `:focus` and `:focus-within`) for any component or element.
pub fn use_focus(id: FocusId) -> FocusState {
    let is_focused = CURRENT_FOCUS.with(|f| f.get() == Some(id));
    let has_focus_within = FOCUS_WITHIN_SET.with(|s| s.borrow().contains(&id));
    FocusState {
        is_focused,
        has_focus_within,
    }
}

/// Reusable handle to manage and query focus state on custom components.
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
        use_focus(self.id).is_focused()
    }

    pub fn has_focus_within(&self) -> bool {
        use_focus(self.id).has_focus_within()
    }

    pub fn is_active(&self) -> bool {
        use_focus(self.id).is_active()
    }
}

impl Default for FocusHandle {
    fn default() -> Self {
        Self::new()
    }
}

pub fn is_focused(id: FocusId) -> bool {
    CURRENT_FOCUS.with(|f| f.get() == Some(id))
}

pub fn get_focused() -> Option<FocusId> {
    CURRENT_FOCUS.with(|f| f.get())
}

pub fn set_focused(id: Option<FocusId>) {
    CURRENT_FOCUS.with(|f| f.set(id));
}

pub fn update_focus_context(focused_id: Option<FocusId>, hit_map: &HitMap) {
    CURRENT_FOCUS.with(|f| f.set(focused_id));
    FOCUS_WITHIN_SET.with(|s| {
        let mut set = s.borrow_mut();
        set.clear();
        if let Some(ancestors) = focused_id.and_then(|active| hit_map.ancestors.get(&active)) {
            for &anc in ancestors {
                set.insert(anc);
            }
        }
    });
}
