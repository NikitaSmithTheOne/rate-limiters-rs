![Crates.io](https://img.shields.io/crates/v/rate-limiters)
![License](https://img.shields.io/badge/license-MIT-blue)

# Популярные `Rate Limiter` алгоритмы для `Rust`

`Rust` библиотека c реализацией популярных алгоритмов ограничения скорости с поддержкой многопоточного использования.

> ⭐ Если вам нравится этот пакет и он оказался полезным, поставьте, пожалуйста, звезду — это очень помогает!

---

Версии `README`:

- **English:** [README.md](README.md)
- **Русский:** [README_RU.md](README_RU.md)

# Содержание

- [Описание](#описание)
- [Объяснение алгоритмов (Доступное для детей)](#объяснение-алгоритмов-доступное-для-детей)
  - [`Leaky Bucket`](#leaky-bucket)
  - [`Token Bucket`](#token-bucket)
  - [`Fixed Window Counter`](#fixed-window-counter)
  - [`Sliding Window Log`](#sliding-window-log)
  - [`Sliding Window Counter`](#sliding-window-counter)
- [Установка](#установка)
- [Использование](#использование)
  - [Пример `Leaky Bucket`](#пример-leaky-bucket)
- [Лицензия](#лицензия)

# Описание

Библиотека позволяет легко добавлять `rate limiting` в ваши Rust-приложения, чтобы контролировать поток запросов или нагрузку на сервис. Доступные реализации:

- [`Leaky Bucket`](./src/leaky_bucket/impl.rs)
- [`Token Bucket`](./src/token_bucket/impl.rs)
- [`Fixed Window Counter`](./src/fixed_window_counter/impl.rs)
- [`Sliding Window Log`](./src/sliding_window_log/impl.rs)
- [`Sliding Window Counter`](./src/sliding_window_counter/impl.rs)

## Объяснение алгоритмов (Доступное для детей)

Смешные и понятные аналогии для детей, которые объясняют алгоритмы.

### `Leaky Bucket`

1. У тебя есть ведро с определённой ёмкостью (например, 10), в которое можно складывать конфеты.
2. Ты можешь класть в ведро конфеты по одной или сразу несколько.
3. Ведро имеет отверстие, из которого равномерно и постоянно вываливается одна или несколько конфет.
4. Если ведро становится полным, то в него больше нельзя положить новые конфеты, пока не выпадут те, которые уже внутри.

### `Token Bucket`

1. У тебя есть ведро, в которое с определённой скоростью падают жетоны (например, по одному жетону в секунду). Жетоны можно обменивать на конфеты.
2. Ведро имеет ограниченную ёмкость (например, максимум 10 жетонов).
3. Когда ты берёшь конфету, ты должен достать жетон. Если жетон есть — ты берёшь конфету, если жетона нет — конфету взять нельзя.
4. Если жетонов накопилось больше, чем вмещает ведро, лишние жетоны просто исчезают.

### `Fixed Window Counter`

1. У тебя есть корзинка с ограниченной ёмкостью (например, 10) и часы, которые делят время на равные интервалы (например, по 1 секунде).
2. В каждом интервале ты можешь взять только столько конфет, сколько вмещает корзинка.
3. Если ты взял все конфеты в этом интервале, то до его конца больше брать конфеты нельзя.
4. Когда начинается новый интервал (например, прошла секунда) — корзинка снова пустая, и можно брать конфеты заново.

### `Sliding Window Log`

1. У тебя есть корзинка и тетрадка, где ты записываешь время, когда берёшь каждую конфету.
2. За последние N секунд (например, 1) можно взять не больше K конфет (например, 10).
3. Каждый раз, когда хочешь взять конфету, смотришь в тетрадку, чтобы проверить сколько конфет уже взял за последние N секунд.
4. Если взял меньше K — берёшь новую конфету и записываешь время, иначе придётся подождать, пока старые записи «устареют».

### `Sliding Window Counter`

1. У тебя есть большая корзина и маленькие корзинки для каждого кусочка времени (например, по 1 секунде).
2. В каждой маленькой корзинке ты считаешь, сколько конфет взял за время, отведённое этой корзинке.
3. Каждый раз, когда хочешь взять конфету из большой корзины, смотришь на сумму конфет во всех маленьких корзинках за последние N секунд.
4. Если суммарно меньше K — берёшь конфету и добавляешь её в текущую маленькую корзинку, иначе придётся подождать.

# Установка

```bash
cargo add rate_limiters
```

# Использование

Все примеры использования можно посмотреть в директории [`examples`](./examples/).

## Пример `Leaky Bucket`

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
