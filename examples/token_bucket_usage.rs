// cargo run --example token_bucket_usage
use std::thread;
use std::time::Duration;

use rate_limiters::token_bucket::TokenBucket;

fn main() {
    let mut bucket = TokenBucket::new(10, 1);

    for i in 0..100 {
        let is_acquired = bucket.try_acquire(1);
        println!(
            "Request #{i}: {}",
            if is_acquired {
                "Allowed"
            } else {
                "Rate limited"
            }
        );

        thread::sleep(Duration::from_millis(250));
    }
}
