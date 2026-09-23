//! NaiveIO is a simple asynchronous runtime which heavily inspired by Tokio. It aims for
//! beginner who studying how asynchronous runtime works

pub mod handle;
mod runtime;
pub mod task;
pub mod time;

pub use runtime::Runtime;

use crate::handle::{Handle, JoinHandle};

pub fn spawn<F, T>(future: F) -> JoinHandle<T>
where
    F: Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    let current = Handle::current();
    current.spawn(future)
}
