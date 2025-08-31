![Crates.io](https://img.shields.io/crates/v/rate-limiters)
![License](https://img.shields.io/badge/license-MIT-blue)

# Popular `Rate Limiter` Algorithms for `Rust`

A `Rust` library implementing popular rate limiting algorithms with support for multithreaded usage.

---

`README` versions:

- **English:** [README.md](README.md)
- **Русский:** [README_RU.md](README_RU.md)

# Description

This library provides simple and efficient implementations of `rate limiting` algorithms for Rust applications. Useful for controlling request flow or limiting load on your services. Available implementations:

- [`Leaky Bucket`](./src/leaky_bucket/impl.rs)
- [`Token Bucket`](./src/token_bucket/impl.rs)
- [`Fixed Window Counter`](./src/fixed_window_counter/impl.rs)
- [`Sliding Window Log`](./src/sliding_window_log/impl.rs)
- [`Sliding Window Counter`](./src/sliding_window_counter/impl.rs)

# Installation

```bash
cargo add rate_limiters
```

# Usage

All usage examples can be found in the [`examples`](./examples/) directory.

## Algorithm Explanations (Kid-Friendly)

Funny and easy-to-understand analogies that explain rate limiting algorithms.

### `Leaky Bucket`

1. You have a bucket with a fixed capacity (e.g., 10) where you can put candies.
2. You can put candies into the bucket one by one or several at a time.
3. The bucket has a hole, and candies leak out evenly and constantly.
4. If the bucket becomes full, you can’t add more candies until some of the existing ones leak out.

### `Token Bucket`

1. You have a bucket where tokens fall at a fixed rate (e.g., one token per second). Tokens can be exchanged for candies.
2. The bucket has a limited capacity (e.g., a maximum of 10 tokens).
3. To take a candy, you must spend a token. If a token is available, you take a candy; if not — you cannot take a candy.
4. If more tokens arrive than the bucket can hold, the extra tokens simply disappear.

### `Fixed Window Counter`

1. You have a basket with limited capacity (e.g., 10) and a clock that divides time into equal intervals (e.g., 1 second).
2. In each interval, you can take only as many candies as the basket can hold.
3. If you’ve taken all candies in this interval, you can’t take more until the interval ends.
4. When a new interval starts (e.g., a second passes), the basket is empty again, and you can take candies anew.

### `Sliding Window Log`

1. You have a basket and a notebook where you record the time you take each candy.
2. In the last N seconds (e.g., 1 second), you can take no more than K candies (e.g., 10).
3. Each time you want a candy, you check the notebook to see how many candies you’ve taken in the last N seconds.
4. If it’s fewer than K — you take a candy and write down the time; otherwise, you have to wait until old entries “expire.”

### `Sliding Window Counter`

1. You have a big basket and small baskets for each time slice (e.g., 1 second each).
2. In each small basket, you count how many candies you’ve taken in that time slice.
3. Each time you want a candy from the big basket, you check the sum of candies across all small baskets for the last N seconds.
4. If the total is less than K — you take a candy and add it to the current small basket; otherwise, you have to wait.

## Leaky Bucket

Code example:

```rs
use std::thread;
use std::time::{Duration, Instant};

use rate_limiters::leaky_bucket::LeakyBucket;
use rate_limiters::token_bucket::r#impl::RateLimiter;

fn main() {
    let start = Instant::now();
    let mut bucket = LeakyBucket::new(3, 1.0);

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
```

Output:

```text
[ 0.00s] Request #001 | Allowed      | Limit:  3 | Remaining:  3 | Used:  0 | Reset: 1756307371
[ 0.30s] Request #002 | Allowed      | Limit:  3 | Remaining:  2 | Used:  1 | Reset: 1756307372
[ 0.60s] Request #003 | Allowed      | Limit:  3 | Remaining:  2 | Used:  1 | Reset: 1756307373
[ 0.90s] Request #004 | Rate limited | Limit:  3 | Remaining:  1 | Used:  2 | Reset: 1756307374
[ 1.20s] Request #005 | Allowed      | Limit:  3 | Remaining:  1 | Used:  2 | Reset: 1756307374
[ 1.50s] Request #006 | Rate limited | Limit:  3 | Remaining:  1 | Used:  2 | Reset: 1756307375
[ 1.81s] Request #007 | Rate limited | Limit:  3 | Remaining:  1 | Used:  2 | Reset: 1756307375
[ 2.11s] Request #008 | Allowed      | Limit:  3 | Remaining:  1 | Used:  2 | Reset: 1756307375
[ 2.41s] Request #009 | Rate limited | Limit:  3 | Remaining:  0 | Used:  3 | Reset: 1756307376
[ 2.71s] Request #010 | Rate limited | Limit:  3 | Remaining:  1 | Used:  2 | Reset: 1756307376
```

# License

MIT License. See [LICENSE](./LICENSE) for details.
