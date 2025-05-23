#[cfg(test)]
mod sequential_tests {
    use crate::token_bucket::SharedTokenBucket;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn try_acquire_basic() {
        let bucket = SharedTokenBucket::new(10, 1);

        assert!(bucket.try_acquire(5));
        assert!(bucket.try_acquire(5));

        assert!(!bucket.try_acquire(1));
    }

    #[test]
    fn try_acquire_with_refill_after_delay() {
        let bucket = SharedTokenBucket::new(10, 1);
        assert!(bucket.try_acquire(10));

        thread::sleep(Duration::from_secs(1));
        assert!(bucket.try_acquire(1));
        assert!(!bucket.try_acquire(1));
    }
}

#[cfg(test)]
mod parallel_tests {
    use crate::token_bucket::SharedTokenBucket;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::thread;
    use std::time::{Duration, Instant};

    #[test]
    fn race_condition_test() {
        let bucket = SharedTokenBucket::new(10, 0);
        let bucket = Arc::new(bucket);
        let success_count = Arc::new(AtomicU32::new(0));

        let mut handles = vec![];
        for _ in 0..20 {
            let bucket_clone = Arc::clone(&bucket);
            let counter_clone = Arc::clone(&success_count);

            let handle = thread::spawn(move || {
                if bucket_clone.try_acquire(1) {
                    counter_clone.fetch_add(1, Ordering::SeqCst);
                }
            });

            handles.push(handle);
        }
        for handle in handles {
            handle.join().unwrap();
        }

        let result = success_count.load(Ordering::SeqCst);
        assert_eq!(result, 10, "Race condition: {} tokens used!", result);
    }

    #[test]
    fn race_condition_with_refill() {
        let bucket = SharedTokenBucket::new(5, 5);
        let bucket = Arc::new(bucket);
        let success_count = Arc::new(AtomicU32::new(0));
        let start = Instant::now();
        let duration = Duration::from_secs(3);

        let mut handles = vec![];

        for _ in 0..10 {
            let bucket_clone = Arc::clone(&bucket);
            let counter_clone = Arc::clone(&success_count);

            let handle = thread::spawn(move || {
                while Instant::now().duration_since(start) < duration {
                    if bucket_clone.try_acquire(1) {
                        counter_clone.fetch_add(1, Ordering::SeqCst);
                    }
                    thread::sleep(Duration::from_millis(50));
                }
            });

            handles.push(handle);
        }
        for handle in handles {
            handle.join().unwrap();
        }

        let result = success_count.load(Ordering::SeqCst);
        assert!(result <= 20, "More tokens used than expected: {}", result);
    }
}
