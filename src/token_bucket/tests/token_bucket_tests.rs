#[cfg(test)]
mod sequential_tests {
    use crate::token_bucket::TokenBucket;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn try_acquire_basic() {
        let mut bucket = TokenBucket::new(10, 1);

        assert!(bucket.try_acquire(5));
        assert!(bucket.try_acquire(5));

        assert!(!bucket.try_acquire(1));
    }

    #[test]
    fn try_acquire_with_refill_after_delay() {
        let mut bucket = TokenBucket::new(10, 1);
        assert!(bucket.try_acquire(10));

        thread::sleep(Duration::from_secs(1));
        assert!(bucket.try_acquire(1));
        assert!(!bucket.try_acquire(1));
    }
}
