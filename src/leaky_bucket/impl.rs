use std::sync::{Arc, Mutex};
use std::time::Instant;

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

    fn leak(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_check).as_secs_f64();
        let leaked = elapsed * self.leak_rate;

        if leaked > 0.0 {
            self.water = (self.water - leaked).max(0.0);
            self.last_check = now;
        }
    }

    pub fn try_acquire(&mut self, amount: u32) -> bool {
        self.leak();
        if self.water + amount as f64 <= self.capacity as f64 {
            self.water += amount as f64;
            true
        } else {
            false
        }
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

    pub fn try_acquire(&self, amount: u32) -> bool {
        let mut bucket = self.inner.lock().unwrap();
        bucket.try_acquire(amount)
    }
}
