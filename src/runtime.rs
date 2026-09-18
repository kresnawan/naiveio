use std::{sync::{Arc, mpsc}};

use crate::task::Task;

pub struct Runtime {
    scheduled: mpsc::Receiver<Arc<Task>>,
    sender: mpsc::Sender<Arc<Task>>,
}

impl Runtime {
    pub fn new() -> Runtime {
        let (sender, scheduled) = mpsc::channel();
        Runtime {
            scheduled,
            sender,
        }
    }

    pub fn spawn<F>(&mut self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        Task::spawn(future, &self.sender);
    }

    pub fn run(&mut self) {
        while let Ok(task) = self.scheduled.recv() {
            task.clone().poll();
        }
    }
}