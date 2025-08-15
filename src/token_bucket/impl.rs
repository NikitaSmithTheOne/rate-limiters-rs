use std::sync::{Arc, Mutex};
use std::time::{Instant, UNIX_EPOCH};

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

    pub fn get_limit(&self) -> u32 {
        self.capacity
    }

    pub fn get_remaining(&mut self) -> u32 {
        self.refill();
        self.tokens
    }

    pub fn get_used(&mut self) -> u32 {
        self.refill();
        self.capacity - self.tokens
    }

    pub fn get_reset(&self) -> u64 {
        let now = std::time::SystemTime::now();
        let refill_secs = (self.capacity - self.tokens) as f64 / self.refill_rate as f64;
        let reset_time = now + std::time::Duration::from_secs_f64(refill_secs);
        reset_time.duration_since(UNIX_EPOCH).unwrap().as_secs()
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
