// cargo run --example leaky_bucket_shared_usage
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use rate_limiters::fixed_window_counter::FixedWindowCounterShared;
use rate_limiters::token_bucket::r#impl::RateLimiterShared;

fn main() {
    let bucket = Arc::new(FixedWindowCounterShared::new(10, 2));

    let start = Instant::now();
    let mut handles = vec![];

    for client_id in 0..5 {
        let bucket_clone = Arc::clone(&bucket);
        handles.push(thread::spawn(move || {
            let mut allowed = 0;
            let mut denied = 0;

            for req_id in 0..20 {
                let ok = bucket_clone.try_acquire(1);
                
                let elapsed = start.elapsed().as_secs_f32();
                if ok {
                    allowed += 1;
                    println!(
                        "[{elapsed:5.2}s] Client #{client_id} - Request #{req_id} - Allowed - Remaining {}",
                        bucket_clone.get_remaining()
                    );
                } else {
                    denied += 1;
                    bucket_clone.refresh(); // <-- It must to get correct reset value w/ "jumping"
                    println!(
                        "[{elapsed:5.2}s] Client #{client_id} - Request #{req_id} - Rejected - Reset UNIX {}",
                        bucket_clone.get_reset()
                    );
                }

                thread::sleep(Duration::from_millis(500));
            }

            println!("Client #{client_id} finished: allowed={allowed}, denied={denied}");
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    println!(
        "\n[Final] Used: {}, Remaining: {} (limit={})",
        bucket.get_used(),
        bucket.get_remaining(),
        bucket.get_limit()
    );
}
