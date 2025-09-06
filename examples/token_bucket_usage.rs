// cargo run --example token_bucket_usage
use std::thread;
use std::time::{Duration, Instant};

use rate_limiters::token_bucket::r#impl::RateLimiter;
use rate_limiters::token_bucket::TokenBucket;

fn main() {
    let start = Instant::now();
    let mut bucket = TokenBucket::new(5, 2);

    for i in 0..100 {
        bucket.refresh();
        let limit = bucket.get_limit();
        let remaining = bucket.get_remaining();
        let used = bucket.get_used();
        let reset = bucket.get_reset();
        let is_acquired = bucket.try_acquire(1);

        let elapsed = start.elapsed().as_secs_f32();
        println!(
            "[{elapsed:5.2}s] Request #{:03} | {:<12} | Limit: {:2} | Remaining: {:2} | Used: {:2} | Reset: {}",
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
