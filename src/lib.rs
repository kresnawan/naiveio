//! NaiveIO is a simple asynchronous runtime which heavily inspired by Tokio. It aims for
//! beginner who studying how asynchronous runtime works

use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

mod runtime;
pub mod task;
pub mod time;

pub use runtime::Runtime;

use crate::time::Wheel;

/// Works like tokio::sleep() but dumber one.
pub struct Delay {
    pub when: Instant,
    pub is_registered: bool,
    pub wheel: Arc<Mutex<Wheel>>,
}

impl Delay {
    pub fn new(duration: Duration, wheel: Arc<Mutex<Wheel>>) -> Delay {
        Delay {
            when: Instant::now() + duration,
            is_registered: false,
            wheel,
        }
    }
}

impl Future for Delay {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        if Instant::now() >= self.when {
            return Poll::Ready(());
        }

        if !self.is_registered {
            let remaining = self.when.saturating_duration_since(Instant::now());
            let ticks = remaining.as_millis() as usize;

            self.wheel
                .lock()
                .unwrap()
                .insert(cx.waker().clone(), ticks.max(1));
            self.is_registered = true;
        }

        return Poll::Pending;
    }
}
