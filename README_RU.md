![Crates.io](https://img.shields.io/crates/v/rate-limiters)
![License](https://img.shields.io/badge/license-MIT-blue)

# rate-limiters-rs

`Rust` библиотека для реализации популярных алгоритмов ограничения скорости.

---

Версии `README`:

- **English:** [README.md](README.md)
- **Русский:** [README_RU.md](README_RU.md)

# Описание

Библиотека позволяет легко добавлять `rate limiting` в ваши Rust-приложения, чтобы контролировать поток запросов или нагрузку на сервис. Доступные реализации:

- [`Leaky Bucket`](./src/leaky_bucket/impl.rs)
- [`Token Bucket`](./src/token_bucket/impl.rs)
- [`Fixed Window Counter`](./src/fixed_window_counter/impl.rs)
- [`Sliding Window Log`](./src/sliding_window_log/impl.rs)
- [`Sliding Window Counter`](./src/sliding_window_counter/impl.rs)

# Установка

```bash
cargo add rate_limiters
```

# Использование

Все примеры использования можно посмотреть в директории [`examples`](./examples/).

## Leaky Bucket

Код:

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

Вывод:

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

# Лицензия

MIT License. Подробнее см. [LICENSE](./LICENSE)
