use std::{pin::Pin, sync::{Arc, Mutex, mpsc}, task::{Context, Poll, Wake, Waker}};

pub struct Task {
    pub task_future: Mutex<TaskFuture>,
    pub executor: mpsc::Sender<Arc<Task>>,
}

impl Task {
    pub fn poll(self: Arc<Self>) {
        let waker = Waker::from(self.clone());
        let mut cx = Context::from_waker(&waker);

        let mut task_future = self.task_future.try_lock().unwrap();
        task_future.poll(&mut cx);
    }

    fn schedule(self: &Arc<Self>) {
        let _ = self.executor.send(self.clone());
    }

    pub fn spawn<F>(future: F, sender: &mpsc::Sender<Arc<Task>>)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let task = Arc::new(Task {
            task_future: Mutex::new(TaskFuture::new(future)),
            executor: sender.clone(),
        });

        let _ = sender.send(task);
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

    fn poll(&mut self, cx: &mut Context<'_>) {
        if self.poll.is_pending() {
            self.poll = self.future.as_mut().poll(cx);
        }
    }
}

