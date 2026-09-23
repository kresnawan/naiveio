use crate::handle::{self, EnterGuard, Handle};
use std::{
    collections::VecDeque,
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};

pub struct Runtime {
    pub name: String,
    handle: Handle,
}

impl Runtime {
    pub fn new(name: &str) -> Runtime {
        Runtime {
            name: name.to_string(),
            handle: Handle {
                queue: Arc::new((
                    Mutex::new(VecDeque::new()),
                    Condvar::new(),
                    AtomicUsize::new(0),
                )),
            },
        }
    }

    pub fn enter(&self) -> EnterGuard {
        let prev = handle::set_current(self.handle.clone());
        EnterGuard::new(prev)
    }

    pub fn spawn<F>(&self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.handle.spawn(future);
    }

    pub fn block_on<F, T>(&mut self, future: F) -> F::Output
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        let _enter_guard = self.enter();
        let j_handle = self.handle.spawn(future);

        loop {
            if let Some(result) = j_handle.stage.lock().unwrap().result.take() {
                return result;
            }

            let (queue, cvar, _) = &*self.handle.queue;

            let mut guard = queue.lock().unwrap();

            while guard.is_empty() {
                guard = cvar.wait(guard).unwrap();
            }

            let task = guard.pop_front();
            drop(guard);

            if let Some(task) = task {
                task.poll();
            }
        }
    }

    pub fn run(&mut self) {
        loop {
            let (queue, cvar, t_count) = &*self.handle.queue;

            if t_count.load(Ordering::SeqCst) == 0 {
                break;
            }

            let mut guard = queue.lock().unwrap();

            while guard.is_empty() {
                guard = cvar.wait(guard).unwrap();
            }

            let task = guard.pop_front();
            drop(guard);

            if let Some(task) = task {
                task.poll();
            }
        }
    }
}
