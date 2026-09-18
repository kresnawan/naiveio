use std::{
    sync::{Arc, Mutex},
    task::Waker,
    thread,
    time::Duration,
};

const NUM_SLOTS: usize = 64;
const TICK_DURATION: Duration = Duration::from_millis(1);

struct TimerEntry {
    expiration: u64,
    waker: Waker,
}

struct Level {
    slots: Vec<Vec<TimerEntry>>,
    current: usize,
}

impl Level {
    fn new() -> Level {
        Level {
            slots: (0..NUM_SLOTS).map(|_| Vec::new()).collect(),
            current: 0,
        }
    }
}


pub struct Wheel {
    levels: Vec<Vec<Level>>,
    current_tick: u64,
}

impl Wheel {
    pub fn new() -> Wheel {
        Wheel {
            levels: (0..NUM_SLOTS).map(|_| Vec::new()).collect(),
            current_tick: 0,
        }
    }

    pub fn insert(&mut self, waker: Waker, ticks: usize) {
        let expiration = self.current_tick + ticks as u64;
        self.insert_entry(TimerEntry { expiration, waker });
    }

    pub(crate) fn insert_entry(&mut self, timer_entry: TimerEntry) {

    }

    pub fn tick(&mut self) -> Vec<Waker> {
        
    }
}

pub struct TimeDriver {
    pub wheel: Arc<Mutex<Wheel>>,
}

impl TimeDriver {
    pub fn start() -> Arc<Mutex<Wheel>> {
        let wheel = Arc::new(Mutex::new(Wheel::new()));
        let wheel_clone = wheel.clone();

        thread::spawn(move || {
            loop {
                thread::sleep(TICK_DURATION);
                let expired = {
                    let mut w = wheel_clone.lock().unwrap();
                    w.tick()
                };

                for waker in expired {
                    waker.wake();
                }
            }
        });

        wheel
    }
}
