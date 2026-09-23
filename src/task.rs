use std::{
    pin::Pin,
    sync::{Arc, Mutex, atomic::Ordering},
    task::{Context, Poll, Wake, Waker},
};

use crate::handle::{Handle, Stage};

pub trait Schedulable {
    fn poll(self: Arc<Self>);
}

pub struct Task<T> {
    pub task_future: Mutex<TaskFuture<T>>,
    pub stage: Arc<Mutex<Stage<T>>>,
    pub handle: Handle,
}

impl<T> Schedulable for Task<T>
where
    T: Send + 'static,
{
    fn poll(self: Arc<Self>) {
        let waker = Waker::from(self.clone());
        let mut cx = Context::from_waker(&waker);

        let mut task_future = self.task_future.try_lock().unwrap();
        task_future.poll(&mut cx, self.handle.clone(), self.stage.clone());
    }
}

impl<T> Task<T>
where
    T: Send + 'static,
{
    fn schedule(self: &Arc<Self>) {
        let mut handle = self.handle.queue.0.lock().unwrap();
        handle.push_back(self.clone());
        self.handle.queue.1.notify_one();
    }

    pub fn spawn<F>(future: F, handle: Handle, stage: &Arc<Mutex<Stage<T>>>)
    where
        F: Future<Output = T> + Send + 'static,
    {
        let task = Arc::new(Task {
            task_future: Mutex::new(TaskFuture::new(future)),
            handle: handle.clone(),
            stage: stage.clone(),
        });

        let (queue, cvar, _) = &*handle.queue;

        queue.lock().unwrap().push_back(task);
        cvar.notify_one();
    }
}

// impl ArcWake for Task {
//     fn wake_by_ref(arc_self: &Arc<Self>) {
//         arc_self.schedule();
//     }
// }

impl<T: Send + 'static> Wake for Task<T> {
    fn wake_by_ref(self: &Arc<Self>) {
        self.schedule();
    }

    fn wake(self: Arc<Self>) {
        self.schedule();
    }
}

pub struct TaskFuture<T> {
    pub future: Pin<Box<dyn Future<Output = T> + Send>>,
    pub is_ready: bool,
}

impl<T> TaskFuture<T> {
    fn new(future: impl Future<Output = T> + Send + 'static) -> TaskFuture<T> {
        TaskFuture {
            future: Box::pin(future),
            is_ready: false,
        }
    }

    fn poll(&mut self, cx: &mut Context<'_>, handle: Handle, stage: Arc<Mutex<Stage<T>>>) {
        if !self.is_ready {
            let result = self.future.as_mut().poll(cx);
            if let Poll::Ready(v) = result {
                handle.queue.2.fetch_sub(1, Ordering::SeqCst);

                let maybe_waker: Option<Waker>;

                {
                    let mut lock = stage.lock().unwrap();
                    lock.result.replace(v);
                    maybe_waker = lock.waker.take();
                }

                self.is_ready = true;
                if let Some(waker) = maybe_waker {
                    waker.wake();
                }
            }
        }
    }
}
