use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::traits::{RateLimiter, RateLimiterShared};

// *** SLIDING WINDOW LOG ***
pub struct SlidingWindowLog {
    capacity: u32,
    window: Duration,
    log: VecDeque<u64>,
}

impl SlidingWindowLog {
    pub fn new(capacity: u32, window_secs: u64) -> Self {
        Self {
            capacity,
            window: Duration::from_secs(window_secs),
            log: VecDeque::new(),
        }
    }

    fn now_secs() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    fn cleanup(&mut self) {
        let now = Self::now_secs();
        while let Some(&ts) = self.log.front() {
            if now - ts >= self.window.as_secs() {
                self.log.pop_front();
            } else {
                break;
            }
        }
    }
}

impl RateLimiter for SlidingWindowLog {
    fn refresh(&mut self) {
        self.cleanup();
    }

    fn try_acquire(&mut self, tokens: u32) -> bool {
        self.cleanup();
        if self.log.len() + tokens as usize <= self.capacity as usize {
            let now = Self::now_secs();
            for _ in 0..tokens {
                self.log.push_back(now);
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
        self.capacity.saturating_sub(self.log.len() as u32)
    }

    fn get_used(&self) -> u32 {
        self.log.len() as u32
    }

    fn get_reset(&self) -> u64 {
        // Skip stale entries inline so callers get an accurate reset even if
        // refresh() wasn't called first.
        let now = Self::now_secs();
        let window = self.window.as_secs();
        for &ts in self.log.iter() {
            if now.saturating_sub(ts) < window {
                return ts + window;
            }
        }
        now
    }
}

// *** SLIDING WINDOW LOG SHARED ***
pub struct SlidingWindowLogShared {
    inner: Arc<Mutex<SlidingWindowLog>>,
}

impl SlidingWindowLogShared {
    pub fn new(capacity: u32, window_secs: u64) -> Self {
        Self {
            inner: Arc::new(Mutex::new(SlidingWindowLog::new(capacity, window_secs))),
        }
    }
}

impl RateLimiterShared for SlidingWindowLogShared {
    fn refresh(&self) {
        let mut bucket = self.inner.lock().unwrap();
        bucket.refresh();
    }

    fn try_acquire(&self, tokens: u32) -> bool {
        let mut bucket = self.inner.lock().unwrap();
        bucket.try_acquire(tokens)
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
