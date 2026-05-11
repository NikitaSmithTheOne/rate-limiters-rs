use std::hint::black_box;
use std::sync::Arc;
use std::thread;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use rate_limiters::fixed_window_counter::{
    FixedWindowCounter, FixedWindowCounterConfig, FixedWindowCounterShared,
};
use rate_limiters::traits::{RateLimiter, RateLimiterShared};

fn bench_try_acquire_success(c: &mut Criterion) {
    let mut group = c.benchmark_group("FixedWindowCounter/try_acquire_success");
    group.throughput(Throughput::Elements(1));
    group.bench_function("single_token", |b| {
        // Effectively unlimited limit so each call succeeds within the window.
        let mut limiter = FixedWindowCounter::new(FixedWindowCounterConfig {
            limit: u32::MAX,
            window_secs: u64::MAX,
        });
        b.iter(|| black_box(limiter.try_acquire(black_box(1))));
    });
    group.finish();
}

fn bench_try_acquire_rejected(c: &mut Criterion) {
    let mut group = c.benchmark_group("FixedWindowCounter/try_acquire_rejected");
    group.throughput(Throughput::Elements(1));
    group.bench_function("exhausted_window", |b| {
        // Zero limit + huge window guarantees rejection without window resets.
        let mut limiter = FixedWindowCounter::new(FixedWindowCounterConfig {
            limit: 0,
            window_secs: u64::MAX,
        });
        b.iter(|| black_box(limiter.try_acquire(black_box(1))));
    });
    group.finish();
}

fn bench_refresh(c: &mut Criterion) {
    let mut group = c.benchmark_group("FixedWindowCounter/refresh");
    for &window in &[1u64, 60, u64::MAX] {
        group.bench_with_input(
            BenchmarkId::from_parameter(window),
            &window,
            |b, &window| {
                let mut limiter = FixedWindowCounter::new(FixedWindowCounterConfig {
                    limit: u32::MAX,
                    window_secs: window,
                });
                b.iter(|| {
                    limiter.refresh();
                    black_box(&limiter);
                });
            },
        );
    }
    group.finish();
}

fn bench_getters(c: &mut Criterion) {
    let mut group = c.benchmark_group("FixedWindowCounter/getters");
    let limiter = {
        let mut l = FixedWindowCounter::new(FixedWindowCounterConfig {
            limit: 1_000,
            window_secs: 60,
        });
        l.try_acquire(250);
        l
    };
    group.bench_function("get_limit", |b| b.iter(|| black_box(limiter.get_limit())));
    group.bench_function("get_remaining", |b| {
        b.iter(|| black_box(limiter.get_remaining()))
    });
    group.bench_function("get_used", |b| b.iter(|| black_box(limiter.get_used())));
    group.bench_function("get_reset", |b| b.iter(|| black_box(limiter.get_reset())));
    group.finish();
}

fn bench_shared_uncontended(c: &mut Criterion) {
    let mut group = c.benchmark_group("FixedWindowCounterShared/uncontended");
    group.throughput(Throughput::Elements(1));
    group.bench_function("try_acquire", |b| {
        let limiter = FixedWindowCounterShared::new(FixedWindowCounterConfig {
            limit: u32::MAX,
            window_secs: u64::MAX,
        });
        b.iter(|| black_box(limiter.try_acquire(black_box(1))));
    });
    group.finish();
}

fn bench_shared_contended(c: &mut Criterion) {
    let mut group = c.benchmark_group("FixedWindowCounterShared/contended");
    for &threads in &[2usize, 4, 8] {
        group.throughput(Throughput::Elements(threads as u64 * 1_000));
        group.bench_with_input(
            BenchmarkId::from_parameter(threads),
            &threads,
            |b, &threads| {
                b.iter_custom(|iters| {
                    let limiter =
                        Arc::new(FixedWindowCounterShared::new(FixedWindowCounterConfig {
                            limit: u32::MAX,
                            window_secs: u64::MAX,
                        }));
                    let start = std::time::Instant::now();
                    let handles: Vec<_> = (0..threads)
                        .map(|_| {
                            let limiter = Arc::clone(&limiter);
                            thread::spawn(move || {
                                for _ in 0..iters {
                                    black_box(limiter.try_acquire(black_box(1)));
                                }
                            })
                        })
                        .collect();
                    for h in handles {
                        h.join().unwrap();
                    }
                    start.elapsed()
                });
            },
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_try_acquire_success,
    bench_try_acquire_rejected,
    bench_refresh,
    bench_getters,
    bench_shared_uncontended,
    bench_shared_contended,
);
criterion_main!(benches);
