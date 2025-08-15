// cargo run --example leaky_bucket_shared_usage
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use rate_limiters::leaky_bucket::LeakyBucketShared;
use rate_limiters::token_bucket::r#impl::RateLimiterShared;

fn main() {
    let bucket = Arc::new(LeakyBucketShared::new(10, 1.0));

    let mut handles = vec![];
    for i in 0..100 {
        let bucket_clone = Arc::clone(&bucket);
        handles.push(thread::spawn(move || {
            let is_acquired = bucket_clone.try_acquire(1);
            println!(
                "Thread #{i} → {}",
                if is_acquired {
                    "Allowed"
                } else {
                    "Rate limited"
                }
            );
        }));

        thread::sleep(Duration::from_millis(250));
    }

    for handle in handles {
        handle.join().unwrap();
    }
}
