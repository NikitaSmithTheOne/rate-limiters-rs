/// Common rate limiter interface for single-threaded usage (mutable access).
pub trait RateLimiter {
    fn refresh(&mut self);
    fn try_acquire(&mut self, tokens: u32) -> bool;

    fn get_limit(&self) -> u32;
    fn get_remaining(&self) -> u32;
    fn get_used(&self) -> u32;
    fn get_reset(&self) -> u64;
}

/// Common rate limiter interface for multi-threaded usage (shared access).
pub trait RateLimiterShared {
    fn refresh(&self);
    fn try_acquire(&self, tokens: u32) -> bool;

    fn get_limit(&self) -> u32;
    fn get_remaining(&self) -> u32;
    fn get_used(&self) -> u32;
    fn get_reset(&self) -> u64;
}

