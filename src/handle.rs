use std::{
    cell::RefCell,
    collections::VecDeque,
    marker::PhantomData,
    pin::Pin,
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    task::{Context, Poll, Waker},
};

use crate::task::{Schedulable, Task};

thread_local! {
    static CURRENT: RefCell<Option<Handle>> = const {RefCell::new(None)};
}

pub struct Stage<T> {
    pub result: Option<T>,
    pub waker: Option<Waker>,
}

pub struct JoinHandle<T> {
    pub stage: Arc<Mutex<Stage<T>>>,
}

impl<T> Future for JoinHandle<T> {
    type Output = T;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut lock = self.stage.lock().unwrap();
        if lock.result.is_some() {
            let result = lock.result.take().unwrap();
            return Poll::Ready(result);
        }

        lock.waker = Some(cx.waker().clone());

        Poll::Pending
    }
}

pub struct EnterGuard {
    pub prev: Option<Handle>,

    // Added to prevent EnterGuard from being moved between threads (it's !Send)
    _not_send: PhantomData<*const ()>,
}

impl Drop for EnterGuard {
    fn drop(&mut self) {
        CURRENT.with_borrow_mut(|c| {
            *c = self.prev.take();
        });
    }
}

impl EnterGuard {
    pub fn new(prev: Option<Handle>) -> EnterGuard {
        EnterGuard {
            prev,
            _not_send: PhantomData,
        }
    }
}

#[derive(Clone)]
pub struct Handle {
    pub queue: Arc<(
        Mutex<VecDeque<Arc<dyn Schedulable + Send + Sync>>>,
        Condvar,
        AtomicUsize,
    )>,
}

impl Handle {
    pub fn current() -> Handle {
        CURRENT
            .with_borrow(|c| c.clone())
            .expect("Runtime not found")
    }

    pub fn spawn<F, T>(&self, future: F) -> JoinHandle<T>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        let result = Arc::new(Mutex::new(Stage {
            result: None,
            waker: None,
        }));
        self.queue.2.fetch_add(1, Ordering::SeqCst);
        Task::spawn(future, self.clone(), &result);

        JoinHandle { stage: result }
    }
}

// Returns previous CURRENT
pub fn set_current(handle: Handle) -> Option<Handle> {
    CURRENT.with_borrow_mut(|c| c.replace(handle))
}

pub fn set_current_none() {
    CURRENT.with_borrow_mut(|c| {
        *c = None;
    })
}

pub fn with_current() -> Option<Handle> {
    CURRENT.with_borrow(|c| match c {
        Some(handle) => Some(handle.clone()),

        None => None,
    })
}
