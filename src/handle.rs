use std::{
    cell::RefCell,
    collections::VecDeque,
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};

use crate::task::Task;

thread_local! {
    static CURRENT: RefCell<Option<Handle>> = const {RefCell::new(None)};
}

pub struct EnterGuard {
    pub prev: Option<Handle>,
}

impl Drop for EnterGuard {
    fn drop(&mut self) {
        CURRENT.with_borrow_mut(|c| {
            *c = self.prev.take();
        });
    }
}

#[derive(Clone)]
pub struct Handle {
    pub queue: Arc<(Mutex<VecDeque<Arc<Task>>>, Condvar, AtomicUsize)>,
}

impl Handle {
    pub fn current() -> Handle {
        CURRENT
            .with_borrow(|c| c.clone())
            .expect("Runtime not found")
    }

    pub fn spawn<F>(&self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.queue.2.fetch_add(1, Ordering::SeqCst);
        Task::spawn(future, self.clone());
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
