use std::sync::{Arc, Mutex};
use std::time::{Instant, UNIX_EPOCH};

// *** TOKEN BUCKET ***
pub trait RateLimiter {
    fn refill(&mut self);
    fn try_acquire(&mut self, tokens: u32) -> bool;

    fn get_limit(&self) -> u32;
    fn get_remaining(&self) -> u32;
    fn get_used(&self) -> u32;
    fn get_reset(&self) -> u64;
}

pub struct TokenBucket {
    capacity: u32,
    tokens: u32,
    refill_rate: u32,
    last_refill: Instant,
}

impl TokenBucket {
    pub fn new(capacity: u32, refill_rate: u32) -> Self {
        Self {
            capacity,
            tokens: capacity,
            refill_rate,
            last_refill: Instant::now(),
        }
    }
}

impl RateLimiter for TokenBucket {
    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill);
        let new_tokens = (elapsed.as_secs_f64() * self.refill_rate as f64).floor() as u32;

        if new_tokens > 0 {
            self.tokens = std::cmp::min(self.capacity, self.tokens + new_tokens);
            self.last_refill = now;
        }
    }

    fn try_acquire(&mut self, tokens: u32) -> bool {
        self.refill();
        if self.tokens >= tokens {
            self.tokens -= tokens;
            true
        } else {
            false
        }
    }

    fn get_limit(&self) -> u32 {
        self.capacity
    }

    fn get_remaining(&self) -> u32 {
        self.tokens
    }

    fn get_used(&self) -> u32 {
        self.capacity - self.tokens
    }

    fn get_reset(&self) -> u64 {
        let now = std::time::SystemTime::now();
        let refill_secs = (self.capacity - self.tokens) as f64 / self.refill_rate as f64;
        let reset_time = now + std::time::Duration::from_secs_f64(refill_secs);
        reset_time.duration_since(UNIX_EPOCH).unwrap().as_secs()
    }
}

// *** TOKEN BUCKET SHARED ***
pub trait RateLimiterShared {
    fn refill(&self);
    fn try_acquire(&self, tokens: u32) -> bool;

    fn get_limit(&self) -> u32;
    fn get_remaining(&self) -> u32;
    fn get_used(&self) -> u32;
    fn get_reset(&self) -> u64;
}

pub struct TokenBucketShared {
    inner: Arc<Mutex<TokenBucket>>,
}

impl TokenBucketShared {
    pub fn new(capacity: u32, refill_rate: u32) -> Self {
        Self {
            inner: Arc::new(Mutex::new(TokenBucket::new(capacity, refill_rate))),
        }
    }
}

impl RateLimiterShared for TokenBucketShared {
    fn refill(&self) {
        let mut bucket = self.inner.lock().unwrap();
        bucket.refill()
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
