//! NaiveIO is a simple asynchronous runtime which heavily inspired by Tokio. It aims for
//! beginner who studying how asynchronous runtime works

mod runtime;
pub mod task;
pub mod time;

pub use runtime::Runtime;
