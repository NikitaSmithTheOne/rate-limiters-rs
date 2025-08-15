use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, UNIX_EPOCH};

use crate::token_bucket::r#impl::{RateLimiter, RateLimiterShared};

pub struct LeakyBucket {
    capacity: u32,
    leak_rate: f64,
    water: f64,
    last_check: Instant,
}

impl LeakyBucket {
    pub fn new(capacity: u32, leak_rate: f64) -> Self {
        Self {
            capacity,
            leak_rate,
            water: 0.0,
            last_check: Instant::now(),
        }
    }
}

impl RateLimiter for LeakyBucket {
    fn refresh(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_check).as_secs_f64();
        let leaked = elapsed * self.leak_rate;

        if leaked > 0.0 {
            self.water = (self.water - leaked).max(0.0);
            self.last_check = now;
        }
    }

    fn try_acquire(&mut self, amount: u32) -> bool {
        self.refresh();
        if self.water + amount as f64 <= self.capacity as f64 {
            self.water += amount as f64;
            true
        } else {
            false
        }
    }

    fn get_limit(&self) -> u32 {
        self.capacity
    }

    fn get_remaining(&self) -> u32 {
        (self.capacity as f64 - self.water.round()) as u32
    }

    fn get_used(&self) -> u32 {
        self.water.round() as u32
    }

    fn get_reset(&self) -> u64 {
        let now = std::time::SystemTime::now();
        let seconds = self.water / self.leak_rate;
        let reset_time = now + Duration::from_secs_f64(seconds);
        reset_time.duration_since(UNIX_EPOCH).unwrap().as_secs()
    }
}

pub struct LeakyBucketShared {
    inner: Arc<Mutex<LeakyBucket>>,
}

impl LeakyBucketShared {
    pub fn new(capacity: u32, leak_rate: f64) -> Self {
        Self {
            inner: Arc::new(Mutex::new(LeakyBucket::new(capacity, leak_rate))),
        }
    }
}

impl RateLimiterShared for LeakyBucketShared {
    fn refresh(&self) {
        let mut bucket = self.inner.lock().unwrap();
        bucket.refresh()
    }

    fn try_acquire(&self, amount: u32) -> bool {
        let mut bucket = self.inner.lock().unwrap();
        bucket.try_acquire(amount)
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
