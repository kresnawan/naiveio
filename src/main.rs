use naiveio::{Delay, Runtime, time::TimeDriver};
use std::time::{Duration, Instant};

fn main() {
    // Create a new runtime instance
    let mut rt = Runtime::new();

    let wheel = TimeDriver::start();
    let wheel_cloned = wheel.clone();

    let time = Instant::now();

    // Spawn an asynchronous task, equal to tokio::spawn()
    rt.spawn(async move {
        let _ = Delay::new(Duration::from_millis(20), wheel).await;
        println!("task 1");

        println!("{:?}", time.elapsed());
    });

    rt.spawn(async move {
        let _ = Delay::new(Duration::from_millis(50), wheel_cloned).await;
        println!("task 2");

        println!("{:?}", time.elapsed());
    });

    // Run the runtime
    rt.run();
}
