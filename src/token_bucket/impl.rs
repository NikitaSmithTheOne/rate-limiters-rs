use std::sync::{Arc, Mutex};
use std::time::Instant;

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

    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill);
        let new_tokens = (elapsed.as_secs_f64() * self.refill_rate as f64).floor() as u32;

        if new_tokens > 0 {
            self.tokens = std::cmp::min(self.capacity, self.tokens + new_tokens);
            self.last_refill = now;
        }
    }

    pub fn try_acquire(&mut self, tokens: u32) -> bool {
        self.refill();
        if self.tokens >= tokens {
            self.tokens -= tokens;
            true
        } else {
            false
        }
    }
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

    pub fn try_acquire(&self, tokens: u32) -> bool {
        let mut bucket = self.inner.lock().unwrap();
        bucket.try_acquire(tokens)
    }
}
