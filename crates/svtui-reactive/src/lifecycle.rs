use std::cell::RefCell;
use std::rc::Rc;

type CleanupFn = Box<dyn FnOnce()>;
type CleanupList = Rc<RefCell<Vec<CleanupFn>>>;

thread_local! {
    static CURRENT_SCOPE: RefCell<Option<ScopeHandle>> = const { RefCell::new(None) };
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
    pub fn run<R>(f: impl FnOnce() -> R) -> (R, ScopeHandle) {
        let scope = ScopeHandle::new();
        let prev = CURRENT_SCOPE.with(|s| s.borrow_mut().replace(scope.clone()));
        let result = f();
        CURRENT_SCOPE.with(|s| {
            *s.borrow_mut() = prev;
        });
        (result, scope)
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
    let cleanup = f().into_cleanup();
    if let Some(c) = cleanup {
        CURRENT_SCOPE.with(|scope| {
            if let Some(ref s) = *scope.borrow() {
                s.add_cleanup(c);
            }
        });
    }
}

pub fn on_destroy(f: impl FnOnce() + 'static) {
    CURRENT_SCOPE.with(|scope| {
        if let Some(ref s) = *scope.borrow() {
            s.add_cleanup(f);
        }
    });
}
