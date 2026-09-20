use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

use crate::time::Wheel;

/// Works like tokio::sleep() but naiver one. It doesn't hold the waker, the waker is held 
/// by the time driver (src/time.rs) since the time driver is responsible to act like a 
/// reactor/notifier if any delay is on deadline
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
