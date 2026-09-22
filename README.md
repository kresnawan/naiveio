# NaiveIO
This project aims to give a picture of how Rust asynchronous runtime works, heavily inspired by [tokio](https://github.com/tokio-rs/tokio). It uses thread-local context to demonstrate how we could spawn a task without borrowing the Runtime directly.

This project is far from perfect since my intention in making this project is to study how it works myself.

## Limitations
- Single-thread scheduler only.
- No JoinHandle yet — A task can't return a value.
- Sleep-based timer wheel (1ms) per tick, a bit of drift from real-world time.
- No cancellation / timeout for task yet.
- Not for production, only education.

## Give it a try
In your project:
```rust
use std::time::Duration;

use naiveio::{Runtime, time::{Delay, TimeDriver}};

fn main() {
    let mut rt = Runtime::new("rt1");
    let _guard = rt.enter();

    let timer = TimeDriver::start();

    naiveio::spawn(async {
        let _ = Delay::new(Duration::from_secs(2), timer).await;
        println!("delayed for 2 seconds!");
    });

    rt.run();
}
```
Expected result:
```
delayed for 2 seconds!
```
Run the test by:
```bash
cargo test
```