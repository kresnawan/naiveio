use std::{
    pin::Pin,
    sync::{Arc, Mutex, atomic::Ordering},
    task::{Context, Poll, Wake, Waker},
};

use crate::handle::Handle;

pub struct Task {
    pub task_future: Mutex<TaskFuture>,
    pub handle: Handle,
}

impl Task {
    pub fn poll(self: Arc<Self>) {
        let waker = Waker::from(self.clone());
        let mut cx = Context::from_waker(&waker);

        let mut task_future = self.task_future.try_lock().unwrap();
        task_future.poll(&mut cx, self.handle.clone());
    }

    fn schedule(self: &Arc<Self>) {
        let mut handle = self.handle.queue.0.lock().unwrap();
        handle.push_back(self.clone());
        self.handle.queue.1.notify_one();
    }

    pub fn spawn<F>(future: F, handle: Handle)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let task = Arc::new(Task {
            task_future: Mutex::new(TaskFuture::new(future)),
            handle: handle.clone(),
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

impl Wake for Task {
    fn wake_by_ref(self: &Arc<Self>) {
        self.schedule();
    }

    fn wake(self: Arc<Self>) {
        self.schedule();
    }
}

pub struct TaskFuture {
    pub future: Pin<Box<dyn Future<Output = ()> + Send>>,
    pub poll: Poll<()>,
}

impl TaskFuture {
    fn new(future: impl Future<Output = ()> + Send + 'static) -> TaskFuture {
        TaskFuture {
            future: Box::pin(future),
            poll: Poll::Pending,
        }
    }

    fn poll(&mut self, cx: &mut Context<'_>, handle: Handle) {
        if self.poll.is_pending() {
            self.poll = self.future.as_mut().poll(cx);
            if self.poll.is_ready() {
                handle.queue.2.fetch_sub(1, Ordering::SeqCst);
            }
        }
    }
}
