use crate::element::node::VNode;
use crate::reactive::{ComponentScope, Runtime, ScopeId, SignalId};
use std::cell::{Cell, RefCell};
use std::fmt;
use std::rc::Rc;

type Build = Rc<dyn Fn() -> VNode>;

struct Slot {
    build: RefCell<Build>,
    node: RefCell<VNode>,
    reads: RefCell<Vec<(SignalId, u64)>>,
    scope: Cell<Option<ScopeId>>,
    stale: Cell<bool>,
    runs: Cell<usize>,
}

impl Drop for Slot {
    fn drop(&mut self) {
        if let Some(scope) = self.scope.get() {
            ComponentScope::destroy_detached(scope);
        }
    }
}

/// A piece of the tree that builds itself from a function and remembers
/// what it read. `refresh` builds it again only when something it read has
/// been written, so a signal write costs the slots that read that signal and
/// no others.
///
/// The slot keeps a scope of its own between runs, so the state and cleanups
/// of what it builds live as long as the slot does, and a run that builds
/// something else drops what the last run left behind.
///
/// Anything the function captures that is not a signal is invisible to the
/// tracking. If the owner can hand the slot a different capture, it has to
/// call `replace`, which makes the next `refresh` build again whatever the
/// signals say.
#[derive(Clone)]
pub struct DynamicSlot(Rc<Slot>);

impl DynamicSlot {
    pub fn new(build: impl Fn() -> VNode + 'static) -> Self {
        Self(Rc::new(Slot {
            build: RefCell::new(Rc::new(build)),
            node: RefCell::new(VNode::Empty),
            reads: RefCell::new(Vec::new()),
            scope: Cell::new(None),
            stale: Cell::new(true),
            runs: Cell::new(0),
        }))
    }

    /// Swaps in a new build function, for when its captures changed.
    pub fn replace(&self, build: impl Fn() -> VNode + 'static) {
        *self.0.build.borrow_mut() = Rc::new(build);
        self.0.stale.set(true);
    }

    /// Makes the next `refresh` build again, for when something the slot
    /// holds outside the signals changed.
    pub fn invalidate(&self) {
        self.0.stale.set(true);
    }

    /// Whether `refresh` would build again.
    pub fn is_stale(&self) -> bool {
        self.0.stale.get() || Runtime::reads_changed(&self.0.reads.borrow())
    }

    /// Builds again if the slot is stale, and returns what it holds now.
    pub fn refresh(&self) -> VNode {
        if self.is_stale() {
            let build = self.0.build.borrow().clone();
            let (node, reads) = ComponentScope::run_detached("dynamic", &self.0.scope, || {
                Runtime::collect_reads(|| build())
            });
            *self.0.node.borrow_mut() = node;
            *self.0.reads.borrow_mut() = reads;
            self.0.stale.set(false);
            self.0.runs.set(self.0.runs.get() + 1);
        }
        self.0.node.borrow().clone()
    }

    /// How many times the build function has run.
    pub fn runs(&self) -> usize {
        self.0.runs.get()
    }

    /// The signals the last run read.
    pub fn reads(&self) -> Vec<SignalId> {
        self.0.reads.borrow().iter().map(|&(id, _)| id).collect()
    }
}

impl fmt::Debug for DynamicSlot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DynamicSlot")
            .field("runs", &self.runs())
            .field("stale", &self.is_stale())
            .finish()
    }
}
