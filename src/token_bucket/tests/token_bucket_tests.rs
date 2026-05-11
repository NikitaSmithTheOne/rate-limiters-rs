#[cfg(test)]
mod sequential_tests {
    use crate::token_bucket::r#impl::RateLimiter;
    use crate::token_bucket::{TokenBucket, TokenBucketConfig};
    use std::thread;
    use std::time::Duration;

    #[test]
    fn basic_test() {
        let now_unix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut bucket = TokenBucket::new(TokenBucketConfig {
            capacity: 10,
            refill_rate: 1,
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

    #[test]
    fn refresh_does_not_drift_at_subtick_intervals() {
        // Regression: refresh() used to set last_refill = now, discarding the
        // sub-token fraction of elapsed time. Over many sub-tick refreshes the
        // effective refill rate fell well below the configured rate.
        let mut bucket = TokenBucket::new(TokenBucketConfig {
            capacity: 20,
            refill_rate: 1,
        });
        assert!(bucket.try_acquire(20)); // drain
        assert_eq!(bucket.get_remaining(), 0);

        // Three cycles of 1.7s = ~5.1s real time. At rate 1/s the bucket
        // should accumulate 5 tokens (1 + 2 + 2 across the three refreshes).
        // The buggy version generated only 1 per cycle = 3 total.
        for _ in 0..3 {
            thread::sleep(Duration::from_millis(1700));
            bucket.refresh();
        }
        assert!(
            bucket.get_remaining() >= 5,
            "drift regression: expected >= 5 tokens after ~5.1s, got {}",
            bucket.get_remaining()
        );
    }

    #[test]
    fn zero_refill_rate_never_resets() {
        let mut bucket = TokenBucket::new(TokenBucketConfig {
            capacity: 3,
            refill_rate: 0,
        });
        assert!(bucket.try_acquire(3));
        assert!(!bucket.try_acquire(1));
        bucket.refresh();

        assert_eq!(bucket.get_remaining(), 0);
        assert_eq!(bucket.get_used(), 3);
        assert_eq!(bucket.get_reset(), u64::MAX);

        thread::sleep(Duration::from_secs(1));
        bucket.refresh();
        assert_eq!(bucket.get_remaining(), 0);
        assert_eq!(bucket.get_reset(), u64::MAX);
    }
}
