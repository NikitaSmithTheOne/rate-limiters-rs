use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::token_bucket::r#impl::{RateLimiter, RateLimiterShared};

/// *** SLIDING WINDOW COUNTER ***
pub struct SlidingWindowCounter {
    capacity: u32,
    window: Duration,
    events: VecDeque<Instant>,
}

impl SlidingWindowCounter {
    pub fn new(capacity: u32, window_secs: u64) -> Self {
        Self {
            capacity,
            window: Duration::from_secs(window_secs),
            events: VecDeque::new(),
        }
    }

    fn purge_old(&mut self) {
        let now = Instant::now();
        while let Some(&front) = self.events.front() {
            if now.duration_since(front) > self.window {
                self.events.pop_front();
            } else {
                break;
            }
        }
    }
}

impl RateLimiter for SlidingWindowCounter {
    fn refresh(&mut self) {
        self.purge_old();
    }

    fn try_acquire(&mut self, tokens: u32) -> bool {
        self.refresh();
        if (self.events.len() as u32 + tokens) <= self.capacity {
            let now = Instant::now();
            for _ in 0..tokens {
                self.events.push_back(now);
            }
            true
        } else {
            false
        }
    }

    fn get_limit(&self) -> u32 {
        self.capacity
    }

    fn get_remaining(&self) -> u32 {
        if (self.events.len() as u32) >= self.capacity {
            0
        } else {
            self.capacity - self.events.len() as u32
        }
    }

    fn get_used(&self) -> u32 {
        self.events.len() as u32
    }

    fn get_reset(&self) -> u64 {
        if let Some(&first) = self.events.front() {
            let expire = first + self.window;
            let reset_time = SystemTime::now() + expire.saturating_duration_since(Instant::now());
            reset_time.duration_since(UNIX_EPOCH).unwrap().as_secs()
        } else {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
        }
    }
}

/// *** SLIDING WINDOW COUNTER SHARED ***
pub struct SlidingWindowCounterShared {
    inner: Arc<Mutex<SlidingWindowCounter>>,
}

impl SlidingWindowCounterShared {
    pub fn new(capacity: u32, window_secs: u64) -> Self {
        Self {
            inner: Arc::new(Mutex::new(SlidingWindowCounter::new(capacity, window_secs))),
        }
    }
}

impl RateLimiterShared for SlidingWindowCounterShared {
    fn refresh(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.refresh()
    }

    fn try_acquire(&self, tokens: u32) -> bool {
        let mut inner = self.inner.lock().unwrap();
        inner.try_acquire(tokens)
    }

    fn get_limit(&self) -> u32 {
        let bucket = self.inner.lock().unwrap();
        bucket.get_limit()
    }

    fn get_remaining(&self) -> u32 {
        let bucket = self.inner.lock().unwrap();
        bucket.get_remaining()
    }

    fn get_used(&self) -> u32 {
        let bucket = self.inner.lock().unwrap();
        bucket.get_used()
    }

    fn get_reset(&self) -> u64 {
        let bucket = self.inner.lock().unwrap();
        bucket.get_reset()
    }
}
