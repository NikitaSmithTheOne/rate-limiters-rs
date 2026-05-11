use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::traits::{RateLimiter, RateLimiterShared};

#[derive(Debug, Clone, Copy)]
pub struct SlidingWindowCounterConfig {
    pub capacity: u32,
    pub window_secs: u64,
}

/// *** SLIDING WINDOW COUNTER ***
pub struct SlidingWindowCounter {
    capacity: u32,
    tick: Duration,
    slots: VecDeque<u32>,
    used: u32,
    last_tick: Instant,
}

impl SlidingWindowCounter {
    pub fn new(config: SlidingWindowCounterConfig) -> Self {
        let window_secs = config.window_secs.max(1);
        let tick = Duration::from_secs(1);
        let slot_count = window_secs as usize;

        Self {
            capacity: config.capacity,
            tick,
            slots: VecDeque::from(vec![0; slot_count]),
            used: 0,
            last_tick: Instant::now(),
        }
    }

    fn now_unix() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    fn advance_ticks(&mut self) {
        let elapsed = Instant::now().duration_since(self.last_tick);
        let ticks = (elapsed.as_secs_f64() / self.tick.as_secs_f64()).floor() as u64;

        if ticks == 0 {
            return;
        }

        let step = std::cmp::min(ticks as usize, self.slots.len());
        for _ in 0..step {
            if let Some(expired) = self.slots.pop_front() {
                self.used = self.used.saturating_sub(expired);
            }
            self.slots.push_back(0);
        }

        // If we advanced more than the number of slots, everything expired.
        if ticks as usize > step {
            self.used = 0;
            for slot in self.slots.iter_mut() {
                *slot = 0;
            }
        }

        // Advance by the exact time consumed by whole ticks so sub-tick
        // fractions are preserved across calls (prevents tick drift).
        self.last_tick += Duration::from_secs(ticks);
    }
}

impl RateLimiter for SlidingWindowCounter {
    fn refresh(&mut self) {
        self.advance_ticks();
    }

    fn try_acquire(&mut self, tokens: u32) -> bool {
        self.refresh();
        if self.used.saturating_add(tokens) <= self.capacity {
            if let Some(cur) = self.slots.back_mut() {
                *cur = cur.saturating_add(tokens);
            }
            self.used = self.used.saturating_add(tokens);
            true
        } else {
            false
        }
    }

    fn get_limit(&self) -> u32 {
        self.capacity
    }

    fn get_remaining(&self) -> u32 {
        self.capacity.saturating_sub(self.used)
    }

    fn get_used(&self) -> u32 {
        self.used
    }

    fn get_reset(&self) -> u64 {
        let now_unix = Self::now_unix();
        if self.used == 0 {
            return now_unix;
        }

        // Slot at index `i` expires at `last_tick + (i + 1) * tick`. Anchor on
        // `last_tick` (not `now`) so callers don't have to refresh() first to
        // get an accurate value, and skip slots whose expiry is already past.
        let now_instant = Instant::now();
        for (idx, &count) in self.slots.iter().enumerate() {
            if count == 0 {
                continue;
            }
            let expiry = self.last_tick + Duration::from_secs(idx as u64 + 1);
            if expiry > now_instant {
                // Round any sub-second remainder up to the next whole second so
                // a slot that expires moments from now never reports "0".
                let delta = expiry.duration_since(now_instant);
                let secs = delta.as_secs() + (delta.subsec_nanos() > 0) as u64;
                return now_unix + secs;
            }
        }
        now_unix
    }
}

/// *** SLIDING WINDOW COUNTER SHARED ***
pub struct SlidingWindowCounterShared {
    inner: Arc<Mutex<SlidingWindowCounter>>,
}

impl SlidingWindowCounterShared {
    pub fn new(config: SlidingWindowCounterConfig) -> Self {
        Self {
            inner: Arc::new(Mutex::new(SlidingWindowCounter::new(config))),
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
