// naiveio's naive time driver.
// it uses the Hashed and Hierarchical Time Wheels approach to manage the delays

use std::{
    sync::{Arc, Mutex},
    task::Waker,
    thread,
    time::Duration,
};

mod delay;

pub use delay::Delay;

const NUM_SLOTS: usize = 64;
const NUM_LEVELS: usize = 4;
const TICK_DURATION: Duration = Duration::from_millis(1);

pub struct TimerEntry {
    expiration: u64,
    waker: Waker,
}

pub struct Level {
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
    levels: Vec<Level>,
    current_tick: u64,
}

impl Wheel {
    pub fn new() -> Wheel {
        Wheel {
            levels: (0..NUM_LEVELS).map(|_| Level::new()).collect(),
            current_tick: 0,
        }
    }

    pub fn insert(&mut self, waker: Waker, ticks: usize) {
        let expiration = self.current_tick + ticks as u64;
        self.insert_entry(TimerEntry { expiration, waker });
    }

    pub fn insert_entry(&mut self, timer_entry: TimerEntry) {
        let delta = timer_entry.expiration.saturating_sub(self.current_tick);
        let level = Self::level_for_delta(delta);
        let span = (NUM_SLOTS as u64).pow(level as u32);

        let steps = delta / span;
        let slot = (self.levels[level].current as u64 + steps) as usize % NUM_SLOTS;

        self.levels[level].slots[slot].push(timer_entry);
    }

    fn level_for_delta(delta: u64) -> usize {
        let mut level = 0;
        let mut capacity = NUM_SLOTS as u64;
        while level + 1 < NUM_LEVELS && delta >= capacity {
            level += 1;
            capacity *= NUM_SLOTS as u64;
        }
        level
    }

    pub fn tick(&mut self) -> Vec<Waker> {
        self.current_tick += 1;
        self.advance_level(0).into_iter().map(|e| e.waker).collect()
    }

    fn advance_level(&mut self, level: usize) -> Vec<TimerEntry> {
        self.levels[level].current = (self.levels[level].current + 1) % NUM_SLOTS;
        let wrapped = self.levels[level].current == 0;

        if wrapped && level + 1 < NUM_LEVELS {
            let cascaded = self.advance_level(level + 1);
            for entry in cascaded {
                self.insert_entry(entry);
            }
        }

        let current = self.levels[level].current;

        std::mem::take(&mut self.levels[level].slots[current])
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
