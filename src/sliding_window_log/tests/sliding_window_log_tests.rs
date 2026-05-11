#[cfg(test)]
mod sequential_tests {
    use std::thread;
    use std::time::Duration;

    use crate::sliding_window_log::{SlidingWindowLog, SlidingWindowLogConfig};
    use crate::token_bucket::r#impl::RateLimiter;

    #[test]
    fn basic_test() {
        let now_unix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut bucket = SlidingWindowLog::new(SlidingWindowLogConfig {
            capacity: 10,
            window_secs: 2,
        });
        assert_eq!(bucket.get_limit(), 10);
        assert_eq!(bucket.get_remaining(), 10);
        assert_eq!(bucket.get_used(), 0);
        assert!(bucket.get_reset() >= now_unix);

        assert!(bucket.try_acquire(5));
        assert_eq!(bucket.get_limit(), 10);
        assert_eq!(bucket.get_remaining(), 5);
        assert_eq!(bucket.get_used(), 5);
        let diff = bucket.get_reset() - now_unix;
        assert!(diff <= 2 && diff >= 1);

        assert!(bucket.try_acquire(5));
        assert_eq!(bucket.get_limit(), 10);
        assert_eq!(bucket.get_remaining(), 0);
        assert_eq!(bucket.get_used(), 10);
        let diff = bucket.get_reset() - now_unix;
        assert!(diff <= 2 && diff >= 1);

        thread::sleep(Duration::from_secs(1));
        bucket.refresh(); // <-- Call refresh to update details w/ try_acquire call
        assert_eq!(bucket.get_limit(), 10);
        assert_eq!(bucket.get_remaining(), 0);
        assert_eq!(bucket.get_used(), 10);
        let diff = bucket.get_reset() - now_unix;
        assert!(diff <= 2 && diff >= 1);

        thread::sleep(Duration::from_secs(1));
        bucket.refresh(); // <-- Call refresh to update details w/ try_acquire call
        assert_eq!(bucket.get_limit(), 10);
        assert_eq!(bucket.get_remaining(), 10);
        assert_eq!(bucket.get_used(), 0);
        let diff = bucket.get_reset() - now_unix;
        assert!(diff <= 2 && diff >= 1);
    }

    #[test]
    fn get_reset_skips_stale_entries_without_refresh() {
        // Regression: get_reset() used to return `oldest + window` even when
        // the oldest entries had aged past the window — yielding a timestamp
        // in the past. It now skips stale entries inline.
        let mut bucket = SlidingWindowLog::new(SlidingWindowLogConfig {
            capacity: 5,
            window_secs: 2,
        });
        assert!(bucket.try_acquire(3));

        thread::sleep(Duration::from_secs(3));

        let now_unix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        // Intentionally do NOT call refresh — caller's get_reset() should
        // still return a sensible (non-past) value.
        let reset = bucket.get_reset();
        assert!(
            reset >= now_unix,
            "get_reset returned a past timestamp: {} < now {}",
            reset,
            now_unix
        );
        assert!(
            reset <= now_unix + 1,
            "get_reset returned a value too far in the future: {} > now {}",
            reset,
            now_unix
        );
    }
}
