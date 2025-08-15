use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, UNIX_EPOCH};

use crate::token_bucket::r#impl::{RateLimiter, RateLimiterShared};

// *** FIXED RATE LIMITER ***
pub struct FixedRateLimiter {
    limit: u32,
    remaining: u32,
    window: Duration,
    last_reset: Instant,
}

impl FixedRateLimiter {
    pub fn new(limit: u32, window_secs: u64) -> Self {
        Self {
            limit,
            remaining: limit,
            window: Duration::from_secs(window_secs),
            last_reset: Instant::now(),
        }
    }
}

impl RateLimiter for FixedRateLimiter {
    fn refresh(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_reset) >= self.window {
            self.remaining = self.limit;
            self.last_reset = now;
        }
    }

    fn try_acquire(&mut self, tokens: u32) -> bool {
        self.refresh();
        if self.remaining >= tokens {
            self.remaining -= tokens;
            true
        } else {
            false
        }
    }

    fn get_limit(&self) -> u32 {
        self.limit
    }

    fn get_remaining(&self) -> u32 {
        self.remaining
    }

    fn get_used(&self) -> u32 {
        self.limit - self.remaining
    }

    fn get_reset(&self) -> u64 {
        let now = std::time::SystemTime::now();
        let elapsed = Instant::now().duration_since(self.last_reset);
        let remaining = if elapsed < self.window {
            self.window - elapsed
        } else {
            Duration::from_secs(0)
        };
        (now + remaining)
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}

// *** FIXED RATE LIMITER SHARED ***
pub struct FixedRateLimiterShared {
    inner: Arc<Mutex<FixedRateLimiter>>,
}

impl FixedRateLimiterShared {
    pub fn new(limit: u32, window_secs: u64) -> Self {
        Self {
            inner: Arc::new(Mutex::new(FixedRateLimiter::new(limit, window_secs))),
        }
    }
}

impl RateLimiterShared for FixedRateLimiterShared {
    fn refresh(&self) {
        let mut limiter = self.inner.lock().unwrap();
        limiter.refresh()
    }

    fn try_acquire(&self, tokens: u32) -> bool {
        let mut limiter = self.inner.lock().unwrap();
        limiter.try_acquire(tokens)
    }

    fn get_limit(&self) -> u32 {
        let limiter = self.inner.lock().unwrap();
        limiter.get_limit()
    }

    fn get_remaining(&self) -> u32 {
        let limiter = self.inner.lock().unwrap();
        limiter.get_remaining()
    }

    fn get_used(&self) -> u32 {
        let limiter = self.inner.lock().unwrap();
        limiter.get_used()
    }

    fn get_reset(&self) -> u64 {
        let limiter = self.inner.lock().unwrap();
        limiter.get_reset()
    }
}
