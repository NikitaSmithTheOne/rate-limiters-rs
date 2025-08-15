#[cfg(test)]
mod sequential_tests {
    use std::thread;
    use std::time::Duration;

    use crate::leaky_bucket::LeakyBucket;
    use crate::token_bucket::r#impl::RateLimiter;

    #[test]
    fn basic_test() {
        let now_unix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut bucket = LeakyBucket::new(10, 1.0);
        assert_eq!(bucket.get_limit(), 10);
        assert_eq!(bucket.get_remaining(), 10);
        assert_eq!(bucket.get_used(), 0);
        assert!(bucket.get_reset() >= now_unix);

        assert!(bucket.try_acquire(5));
        assert_eq!(bucket.get_limit(), 10);
        assert_eq!(bucket.get_remaining(), 5);
        assert_eq!(bucket.get_used(), 5);
        let diff = bucket.get_reset() - now_unix;
        assert!(diff <= 5 && diff >= 4);

        assert!(bucket.try_acquire(5));
        assert_eq!(bucket.get_limit(), 10);
        assert_eq!(bucket.get_remaining(), 0);
        assert_eq!(bucket.get_used(), 10);
        let diff = bucket.get_reset() - now_unix;
        assert!(diff <= 10 && diff >= 9);

        assert!(!bucket.try_acquire(1));
        assert_eq!(bucket.get_limit(), 10);
        assert_eq!(bucket.get_remaining(), 0);
        assert_eq!(bucket.get_used(), 10);
        let diff = bucket.get_reset() - now_unix;
        assert!(diff <= 10 && diff >= 9);

        thread::sleep(Duration::from_secs(1));
        bucket.refresh(); // <-- Call refresh to update details w/ try_acquire call
        assert_eq!(bucket.get_limit(), 10);
        assert_eq!(bucket.get_remaining(), 1);
        assert_eq!(bucket.get_used(), 9);

        thread::sleep(Duration::from_secs(1));
        bucket.refresh(); // <-- Call refresh to update details w/ try_acquire call
        assert_eq!(bucket.get_limit(), 10);
        assert_eq!(bucket.get_remaining(), 2);
        assert_eq!(bucket.get_used(), 8);
    }
}
