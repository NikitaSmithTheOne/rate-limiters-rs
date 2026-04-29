#[cfg(test)]
mod sequential_tests {
    use crate::sliding_window_counter::SlidingWindowCounter;
    use crate::token_bucket::r#impl::RateLimiter;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn basic_test() {
        let now_unix = || {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        };

        let mut bucket = SlidingWindowCounter::new(10, 2);
        assert_eq!(bucket.get_limit(), 10);
        assert_eq!(bucket.get_remaining(), 10);
        assert_eq!(bucket.get_used(), 0);
        assert!(bucket.get_reset() >= now_unix());

        assert!(bucket.try_acquire(5));
        assert_eq!(bucket.get_limit(), 10);
        assert_eq!(bucket.get_remaining(), 5);
        assert_eq!(bucket.get_used(), 5);
        let diff = bucket.get_reset() - now_unix();
        assert!(diff <= 2 && diff >= 1);

        assert!(bucket.try_acquire(5));
        assert_eq!(bucket.get_limit(), 10);
        assert_eq!(bucket.get_remaining(), 0);
        assert_eq!(bucket.get_used(), 10);
        let diff = bucket.get_reset() - now_unix();
        assert!(diff <= 2 && diff >= 1);

        assert!(!bucket.try_acquire(1));
        assert_eq!(bucket.get_limit(), 10);
        assert_eq!(bucket.get_remaining(), 0);
        assert_eq!(bucket.get_used(), 10);
        let diff = bucket.get_reset() - now_unix();
        assert!(diff <= 2 && diff >= 1);

        thread::sleep(Duration::from_secs(1));
        bucket.refresh(); // <-- Call refresh to update details w/ try_acquire call
        assert_eq!(bucket.get_limit(), 10);
        assert_eq!(bucket.get_remaining(), 0);
        assert_eq!(bucket.get_used(), 10);
        let diff = bucket.get_reset() - now_unix();
        assert!(diff <= 2 && diff >= 1);

        thread::sleep(Duration::from_secs(1));
        bucket.refresh(); // <-- Call refresh to update details w/ try_acquire call
        assert_eq!(bucket.get_limit(), 10);
        assert_eq!(bucket.get_remaining(), 10);
        assert_eq!(bucket.get_used(), 0);
        assert!(bucket.get_reset() >= now_unix());
    }

    #[test]
    fn get_reset_anchored_on_last_tick_not_now() {
        // Regression: get_reset() used `now_unix() + (idx + 1)`, which is wrong
        // when refresh() hasn't been called recently — it ignored the time
        // already elapsed since the last tick. It now anchors on `last_tick`.
        let now_unix = || {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        };

        let mut bucket = SlidingWindowCounter::new(10, 3);
        assert!(bucket.try_acquire(5));
        let initial_reset = bucket.get_reset();
        let initial_now = now_unix();
        // Slot is in the back; expires after 3 seconds.
        assert!(initial_reset - initial_now <= 3);
        assert!(initial_reset - initial_now >= 2);

        // Without calling refresh, after 1s the absolute reset timestamp must
        // not move forward (slot expiry is anchored on last_tick, not now).
        thread::sleep(Duration::from_secs(1));
        let later_reset = bucket.get_reset();
        assert!(
            later_reset <= initial_reset,
            "get_reset drifted forward: initial={}, later={}",
            initial_reset,
            later_reset
        );
    }
}
