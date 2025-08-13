#[cfg(test)]
mod sequential_tests {
    use crate::leaky_bucket::LeakyBucket;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn try_acquire_basic() {
        let mut bucket = LeakyBucket::new(10, 1.0);

        assert!(bucket.try_acquire(5));
        assert!(bucket.try_acquire(5));

        assert!(!bucket.try_acquire(1));
    }

    #[test]
    fn try_acquire_with_leak_after_delay() {
        let mut bucket = LeakyBucket::new(10, 1.0);

        assert!(bucket.try_acquire(10));
        assert!(!bucket.try_acquire(1));

        thread::sleep(Duration::from_secs(1));
        assert!(bucket.try_acquire(1));
        assert!(!bucket.try_acquire(1));
    }
}
