// cargo run --example leaky_bucket_usage
use std::thread;
use std::time::Duration;

use rate_limiters::leaky_bucket::LeakyBucket;
use rate_limiters::token_bucket::r#impl::RateLimiter;

fn main() {
    let mut bucket = LeakyBucket::new(10, 1.0);

    for i in 0..100 {
        let limit = bucket.get_limit();
        let remaining = bucket.get_remaining();
        let used = bucket.get_used();
        let reset = bucket.get_reset();
        let is_acquired = bucket.try_acquire(1);

        println!(
            "Request #{:03} | {:<12} | Limit: {:2} | Remaining: {:2} | Used: {:2} | Reset: {}",
            i + 1,
            if is_acquired {
                "Allowed"
            } else {
                "Rate limited"
            },
            limit,
            remaining,
            used,
            reset
        );

        thread::sleep(Duration::from_millis(300));
    }
}
