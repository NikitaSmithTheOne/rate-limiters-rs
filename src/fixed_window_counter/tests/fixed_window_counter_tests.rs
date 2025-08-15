#[cfg(test)]
mod sequential_tests {
    use std::thread;
    use std::time::Duration;

    use crate::fixed_window_counter::FixedRateLimiter;
    use crate::token_bucket::r#impl::RateLimiter;

    #[test]
    fn basic_test() {
        let now_unix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut bucket = FixedRateLimiter::new(10, 2);
        assert_eq!(bucket.get_limit(), 10);
        assert_eq!(bucket.get_remaining(), 10);
        assert_eq!(bucket.get_used(), 0);
        assert!(bucket.get_reset() >= now_unix);

        assert!(bucket.try_acquire(5));
        assert_eq!(bucket.get_limit(), 10);
        assert_eq!(bucket.get_remaining(), 5);
        assert_eq!(bucket.get_used(), 5);
        let diff = bucket.get_reset() - now_unix;
        assert_eq!(diff, 2);

        assert!(bucket.try_acquire(5));
        assert_eq!(bucket.get_limit(), 10);
        assert_eq!(bucket.get_remaining(), 0);
        assert_eq!(bucket.get_used(), 10);
        let diff = bucket.get_reset() - now_unix;
        assert_eq!(diff, 2);

        assert!(!bucket.try_acquire(1));
        assert_eq!(bucket.get_limit(), 10);
        assert_eq!(bucket.get_remaining(), 0);
        assert_eq!(bucket.get_used(), 10);
        let diff = bucket.get_reset() - now_unix;
        assert_eq!(diff, 2);

        thread::sleep(Duration::from_secs(2));
        bucket.refresh(); // <-- Call refresh to update details w/ try_acquire call
        assert_eq!(bucket.get_limit(), 10);
        assert_eq!(bucket.get_remaining(), 10);
        assert_eq!(bucket.get_used(), 0);
        let diff = bucket.get_reset() - now_unix;
        assert_eq!(diff, 4);
    }
}
