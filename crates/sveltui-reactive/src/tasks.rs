use crate::signal::{State, state};
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use std::thread;

pub fn reset_hook_counter() {
    crate::lifecycle::begin_frame();
}

pub fn clear_async_hooks() {}

#[derive(Clone)]
pub struct TaskHandle {
    cancelled: Arc<AtomicBool>,
}

impl TaskHandle {
    pub fn abort(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    pub fn is_aborted(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
}

pub fn spawn_task(fut: impl Future<Output = ()> + Send + 'static) -> TaskHandle {
    let cancelled = Arc::new(AtomicBool::new(false));
    let cancelled_clone = cancelled.clone();

    thread::spawn(move || {
        let mut pinned = Box::pin(fut);
        let waker = Waker::noop();
        let mut cx = Context::from_waker(waker);

        while !cancelled_clone.load(Ordering::SeqCst) {
            match pinned.as_mut().poll(&mut cx) {
                Poll::Ready(()) => break,
                Poll::Pending => {
                    thread::sleep(std::time::Duration::from_millis(5));
                }
            }
        }
    });

    TaskHandle { cancelled }
}

pub struct BlockingFuture<R> {
    result: Arc<Mutex<Option<R>>>,
}

impl<R> Future for BlockingFuture<R> {
    type Output = R;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut lock = self.result.lock().unwrap();
        if let Some(val) = lock.take() {
            Poll::Ready(val)
        } else {
            Poll::Pending
        }
    }
}

pub fn spawn_blocking<F, R>(f: F) -> BlockingFuture<R>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    let result = Arc::new(Mutex::new(None));
    let result_clone = result.clone();

    thread::spawn(move || {
        let res = f();
        *result_clone.lock().unwrap() = Some(res);
    });

    BlockingFuture { result }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AsyncState<T, E> {
    Loading,
    Resolved(T),
    Rejected(E),
}

pub trait IntoAsyncState<T, E> {
    fn into_async_state(self) -> AsyncState<T, E>;
}

impl<T, E: ToString> IntoAsyncState<T, String> for Result<T, E> {
    fn into_async_state(self) -> AsyncState<T, String> {
        match self {
            Ok(v) => AsyncState::Resolved(v),
            Err(e) => AsyncState::Rejected(e.to_string()),
        }
    }
}

pub fn use_async_state<Fut, Res, T>(future: Fut) -> AsyncState<T, String>
where
    Fut: Future<Output = Res> + Send + 'static,
    Res: IntoAsyncState<T, String> + 'static,
    T: Clone + Send + Sync + 'static,
{
    let s: State<AsyncState<T, String>> = crate::lifecycle::use_hook(|| {
        let s: State<AsyncState<T, String>> = state(AsyncState::Loading);
        let s_clone = s;
        spawn_task(async move {
            let res = future.await;
            s_clone.set(res.into_async_state());
        });
        s
    });

    s.get()
}
