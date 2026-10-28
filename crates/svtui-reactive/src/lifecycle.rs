use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

pub type ScopeId = usize;
type CleanupFn = Box<dyn FnOnce()>;
type CleanupList = Rc<RefCell<Vec<CleanupFn>>>;

thread_local! {
    static SCOPE_MANAGER: RefCell<ScopeManager> = RefCell::new(ScopeManager::new());
    static CURRENT_HANDLE: RefCell<Option<ScopeHandle>> = const { RefCell::new(None) };
}

pub struct ScopeNode {
    pub id: ScopeId,
    pub name: &'static str,
    pub slots: Vec<Box<dyn Any>>,
    pub slot_cursor: usize,
    pub children: Vec<ScopeId>,
    pub child_cursor: usize,
    pub cleanups: Vec<CleanupFn>,
    pub is_first_render: bool,
}

pub struct ScopeManager {
    pub scopes: Vec<Option<ScopeNode>>,
    pub scope_stack: Vec<ScopeId>,
    pub root_id: ScopeId,
}

impl Default for ScopeManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ScopeManager {
    pub fn new() -> Self {
        let root = ScopeNode {
            id: 0,
            name: "root",
            slots: Vec::new(),
            slot_cursor: 0,
            children: Vec::new(),
            child_cursor: 0,
            cleanups: Vec::new(),
            is_first_render: true,
        };
        Self {
            scopes: vec![Some(root)],
            scope_stack: Vec::new(),
            root_id: 0,
        }
    }

    pub fn begin_frame(&mut self) {
        for scope in self.scopes.iter_mut().flatten() {
            scope.slot_cursor = 0;
            scope.child_cursor = 0;
        }
        self.scope_stack.clear();
        self.scope_stack.push(self.root_id);
    }

    pub fn end_frame(&mut self) {
        self.reconcile_children(self.root_id);
        self.scope_stack.clear();
    }

    pub fn enter_scope(&mut self, name: &'static str) -> (ScopeId, bool) {
        let pushed_root = if self.scope_stack.is_empty() {
            self.scope_stack.push(self.root_id);
            true
        } else {
            false
        };

        let parent_id = *self
            .scope_stack
            .last()
            .expect("Scope stack must not be empty");
        let parent = self.scopes[parent_id]
            .as_mut()
            .expect("Parent scope exists");
        let cursor = parent.child_cursor;
        parent.child_cursor += 1;

        let child_id = if cursor < parent.children.len() {
            let existing_id = parent.children[cursor];
            if self.scopes[existing_id]
                .as_ref()
                .is_some_and(|c| c.name == name)
            {
                if let Some(ref mut child) = self.scopes[existing_id] {
                    child.slot_cursor = 0;
                    child.child_cursor = 0;
                }
                existing_id
            } else {
                self.destroy_scope(existing_id);
                let new_id = self.alloc_scope(name);
                self.scopes[parent_id].as_mut().unwrap().children[cursor] = new_id;
                new_id
            }
        } else {
            let new_id = self.alloc_scope(name);
            self.scopes[parent_id]
                .as_mut()
                .unwrap()
                .children
                .push(new_id);
            new_id
        };

        self.scope_stack.push(child_id);
        (child_id, pushed_root)
    }

    pub fn exit_scope(&mut self, scope_id: ScopeId, pushed_root: bool) {
        self.reconcile_children(scope_id);
        if let Some(pos) = self.scope_stack.iter().rposition(|&id| id == scope_id) {
            self.scope_stack.truncate(pos);
        }
        if pushed_root && self.scope_stack.last() == Some(&self.root_id) {
            self.scope_stack.pop();
        }
    }

    pub fn reconcile_children(&mut self, scope_id: ScopeId) {
        let unmounted: Vec<ScopeId> = if let Some(ref mut node) = self.scopes[scope_id] {
            node.is_first_render = false;
            if node.child_cursor < node.children.len() {
                node.children.drain(node.child_cursor..).collect()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        for child_id in unmounted {
            self.destroy_scope(child_id);
        }
    }

    pub fn destroy_scope(&mut self, scope_id: ScopeId) {
        if scope_id >= self.scopes.len() {
            return;
        }
        if let Some(mut node) = self.scopes[scope_id].take() {
            for child_id in node.children {
                self.destroy_scope(child_id);
            }
            for cleanup in node.cleanups.drain(..) {
                cleanup();
            }
        }
    }

    fn alloc_scope(&mut self, name: &'static str) -> ScopeId {
        let node = ScopeNode {
            id: 0,
            name,
            slots: Vec::new(),
            slot_cursor: 0,
            children: Vec::new(),
            child_cursor: 0,
            cleanups: Vec::new(),
            is_first_render: true,
        };

        if let Some(idx) = self.scopes.iter().position(|s| s.is_none()) {
            let mut node = node;
            node.id = idx;
            self.scopes[idx] = Some(node);
            idx
        } else {
            let idx = self.scopes.len();
            let mut node = node;
            node.id = idx;
            self.scopes.push(Some(node));
            idx
        }
    }
}

pub struct ScopeGuard {
    scope_id: ScopeId,
    pushed_root: bool,
}

impl Drop for ScopeGuard {
    fn drop(&mut self) {
        SCOPE_MANAGER.with(|sm| {
            sm.borrow_mut().exit_scope(self.scope_id, self.pushed_root);
        });
    }
}

pub fn begin_frame() {
    SCOPE_MANAGER.with(|sm| sm.borrow_mut().begin_frame());
}

pub fn end_frame() {
    SCOPE_MANAGER.with(|sm| sm.borrow_mut().end_frame());
}

pub fn use_hook<T: Clone + 'static>(init: impl FnOnce() -> T) -> T {
    let target = SCOPE_MANAGER.with(|sm| {
        let mut mgr = sm.borrow_mut();
        if let Some(&current_id) = mgr.scope_stack.last()
            && let Some(scope) = mgr.scopes[current_id].as_mut()
        {
            let cursor = scope.slot_cursor;
            scope.slot_cursor += 1;

            if cursor < scope.slots.len()
                && let Some(existing) = scope.slots[cursor].downcast_ref::<T>()
            {
                return Ok(existing.clone());
            }
            return Err(Some((current_id, cursor)));
        }
        Err(None)
    });

    match target {
        Ok(existing) => existing,
        Err(Some((scope_id, cursor))) => {
            let new_val = init();
            SCOPE_MANAGER.with(|sm| {
                let mut mgr = sm.borrow_mut();
                if let Some(Some(scope)) = mgr.scopes.get_mut(scope_id) {
                    if cursor < scope.slots.len() {
                        scope.slots[cursor] = Box::new(new_val.clone());
                    } else {
                        while scope.slots.len() < cursor {
                            scope.slots.push(Box::new(()));
                        }
                        scope.slots.push(Box::new(new_val.clone()));
                    }
                }
            });
            new_val
        }
        Err(None) => init(),
    }
}

#[derive(Clone, Default)]
pub struct ScopeHandle {
    cleanups: CleanupList,
}

impl ScopeHandle {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_cleanup(&self, f: impl FnOnce() + 'static) {
        self.cleanups.borrow_mut().push(Box::new(f));
    }

    pub fn run_cleanups(&self) {
        let cleanups: Vec<CleanupFn> = self.cleanups.borrow_mut().drain(..).collect();
        for cleanup in cleanups {
            cleanup();
        }
    }
}

pub struct ComponentScope;

impl ComponentScope {
    pub fn run_scoped<R>(name: &'static str, f: impl FnOnce() -> R) -> R {
        let (scope_id, pushed_root) = SCOPE_MANAGER.with(|sm| sm.borrow_mut().enter_scope(name));
        let _guard = ScopeGuard {
            scope_id,
            pushed_root,
        };
        f()
    }

    pub fn run<R>(f: impl FnOnce() -> R) -> (R, ScopeHandle) {
        let handle = ScopeHandle::new();
        let (scope_id, pushed_root) =
            SCOPE_MANAGER.with(|sm| sm.borrow_mut().enter_scope("anonymous"));
        let _guard = ScopeGuard {
            scope_id,
            pushed_root,
        };
        let prev = CURRENT_HANDLE.with(|h| h.borrow_mut().replace(handle.clone()));
        let result = f();
        CURRENT_HANDLE.with(|h| *h.borrow_mut() = prev);
        (result, handle)
    }

    pub fn register_cleanup(cleanup: Box<dyn FnOnce() + 'static>) {
        CURRENT_HANDLE.with(|h| {
            if let Some(ref handle) = *h.borrow() {
                handle.add_cleanup(cleanup);
                return;
            }
            SCOPE_MANAGER.with(|sm| {
                let mut mgr = sm.borrow_mut();
                if let Some(&current_id) = mgr.scope_stack.last()
                    && let Some(ref mut scope) = mgr.scopes[current_id]
                {
                    scope.cleanups.push(cleanup);
                }
            });
        });
    }
}

pub trait IntoCleanup {
    fn into_cleanup(self) -> Option<Box<dyn FnOnce() + 'static>>;
}

impl IntoCleanup for () {
    fn into_cleanup(self) -> Option<Box<dyn FnOnce() + 'static>> {
        None
    }
}

impl<F: FnOnce() + 'static> IntoCleanup for F {
    fn into_cleanup(self) -> Option<Box<dyn FnOnce() + 'static>> {
        Some(Box::new(self))
    }
}

pub fn on_mount<F, C>(f: F)
where
    F: FnOnce() -> C + 'static,
    C: IntoCleanup + 'static,
{
    use_hook(|| {
        let cleanup = f().into_cleanup();
        if let Some(c) = cleanup {
            ComponentScope::register_cleanup(c);
        }
    });
}

pub fn on_destroy(f: impl FnOnce() + 'static) {
    ComponentScope::register_cleanup(Box::new(f));
}
