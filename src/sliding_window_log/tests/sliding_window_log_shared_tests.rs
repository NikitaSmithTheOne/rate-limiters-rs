#[cfg(test)]
mod sequential_tests {
    use crate::sliding_window_log::SlidingWindowLogShared;
    use crate::token_bucket::r#impl::RateLimiterShared;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn basic_test() {
        let now_unix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let bucket = SlidingWindowLogShared::new(10, 2);
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

        thread::sleep(Duration::from_secs(1));
        bucket.refresh(); // <-- Call refresh to update details w/ try_acquire call
        assert_eq!(bucket.get_limit(), 10);
        assert_eq!(bucket.get_remaining(), 0);
        assert_eq!(bucket.get_used(), 10);
        let diff = bucket.get_reset() - now_unix;
        assert_eq!(diff, 2);

        thread::sleep(Duration::from_secs(1));
        bucket.refresh(); // <-- Call refresh to update details w/ try_acquire call
        assert_eq!(bucket.get_limit(), 10);
        assert_eq!(bucket.get_remaining(), 10);
        assert_eq!(bucket.get_used(), 0);
        let diff = bucket.get_reset() - now_unix;
        assert_eq!(diff, 2);
    }
}

#[cfg(test)]
mod parallel_tests {
    use crate::sliding_window_log::SlidingWindowLogShared;
    use crate::token_bucket::r#impl::RateLimiterShared;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::{Arc, Barrier};
    use std::thread;
    use std::time::Duration;

    #[test]
    fn race_condition_test() {
        let bucket = Arc::new(SlidingWindowLogShared::new(10, 1));
        let success_count = Arc::new(AtomicU32::new(0));
        let barrier = Arc::new(Barrier::new(21));

        let mut handles = vec![];
        for i in 0..20 {
            let bucket_clone = Arc::clone(&bucket);
            let success_count_clone = Arc::clone(&success_count);
            let barrier_clone = Arc::clone(&barrier);

            let handle = thread::spawn(move || {
                println!("[Thread {i}] reached barrier");
                barrier_clone.wait();
                println!("[Thread {i}] started race");

                if bucket_clone.try_acquire(1) {
                    println!("[Thread {i}] acquired token");
                    success_count_clone.fetch_add(1, Ordering::SeqCst);
                } else {
                    println!("[Thread {i}] rejected");
                    let _ = bucket_clone.get_remaining();
                    let _ = bucket_clone.get_used();
                    let _ = bucket_clone.get_reset();
                }
            });
            handles.push(handle);
        }

        println!("[Main] releasing barrier...");
        barrier.wait();

        for handle in handles {
            handle.join().unwrap();
        }

        let result = success_count.load(Ordering::SeqCst);
        assert_eq!(result, 10, "Race condition: {} tokens acquired!", result);

        assert_eq!(bucket.get_used(), 10);
        assert_eq!(bucket.get_remaining(), 0);

        thread::sleep(Duration::from_secs(1));
        bucket.refresh();

        let mut success2 = 0;
        for _ in 0..10 {
            if bucket.try_acquire(1) {
                success2 += 1;
            }
        }
        assert_eq!(success2, 10, "After reset should allow 10 new tokens");
        assert_eq!(bucket.get_used(), 10);
        assert_eq!(bucket.get_remaining(), 0);
    }
}
