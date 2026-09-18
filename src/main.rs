use naiveio::{Runtime, time::{Delay, TimeDriver}};
use std::time::{Duration, Instant};

fn main() {
    // Create a new runtime instance
    let mut rt = Runtime::new();

    let wheel = TimeDriver::start();
    let wheel_cloned = wheel.clone();

    let time = Instant::now();

    // Spawn an asynchronous task, equal to tokio::spawn()
    rt.spawn(async move {
        let _ = Delay::new(Duration::from_secs(2), wheel).await;
        println!("task 1 done!");

        println!("{:?}", time.elapsed());
    });

    rt.spawn(async move {
        let _ = Delay::new(Duration::from_secs(1), wheel_cloned).await;
        println!("task 2 done!");

        println!("{:?}", time.elapsed());
    });

    // Run the runtime
    rt.run();
}
