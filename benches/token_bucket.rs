use std::sync::Arc;
use std::thread;

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use rate_limiters::token_bucket::{TokenBucket, TokenBucketConfig, TokenBucketShared};
use rate_limiters::traits::{RateLimiter, RateLimiterShared};

fn bench_try_acquire_success(c: &mut Criterion) {
    let mut group = c.benchmark_group("TokenBucket/try_acquire_success");
    group.throughput(Throughput::Elements(1));
    group.bench_function("single_token", |b| {
        // Effectively unlimited capacity + refill so each call succeeds.
        let mut bucket = TokenBucket::new(TokenBucketConfig {
            capacity: u32::MAX,
            refill_rate: u32::MAX,
        });
        b.iter(|| black_box(bucket.try_acquire(black_box(1))));
    });
    group.finish();
}

fn bench_try_acquire_rejected(c: &mut Criterion) {
    let mut group = c.benchmark_group("TokenBucket/try_acquire_rejected");
    group.throughput(Throughput::Elements(1));
    group.bench_function("empty_bucket", |b| {
        // Zero refill rate prevents any tokens from coming back.
        let mut bucket = TokenBucket::new(TokenBucketConfig {
            capacity: 0,
            refill_rate: 0,
        });
        b.iter(|| black_box(bucket.try_acquire(black_box(1))));
    });
    group.finish();
}

fn bench_refresh(c: &mut Criterion) {
    let mut group = c.benchmark_group("TokenBucket/refresh");
    for &rate in &[0u32, 1_000, 1_000_000] {
        group.bench_with_input(BenchmarkId::from_parameter(rate), &rate, |b, &rate| {
            let mut bucket = TokenBucket::new(TokenBucketConfig {
                capacity: u32::MAX,
                refill_rate: rate,
            });
            b.iter(|| {
                bucket.refresh();
                black_box(&bucket);
            });
        });
    }
    group.finish();
}

fn bench_getters(c: &mut Criterion) {
    let mut group = c.benchmark_group("TokenBucket/getters");
    let bucket = {
        let mut b = TokenBucket::new(TokenBucketConfig {
            capacity: 1_000,
            refill_rate: 100,
        });
        b.try_acquire(250);
        b
    };
    group.bench_function("get_limit", |b| b.iter(|| black_box(bucket.get_limit())));
    group.bench_function("get_remaining", |b| {
        b.iter(|| black_box(bucket.get_remaining()))
    });
    group.bench_function("get_used", |b| b.iter(|| black_box(bucket.get_used())));
    group.bench_function("get_reset", |b| b.iter(|| black_box(bucket.get_reset())));
    group.finish();
}

fn bench_shared_uncontended(c: &mut Criterion) {
    let mut group = c.benchmark_group("TokenBucketShared/uncontended");
    group.throughput(Throughput::Elements(1));
    group.bench_function("try_acquire", |b| {
        let bucket = TokenBucketShared::new(TokenBucketConfig {
            capacity: u32::MAX,
            refill_rate: u32::MAX,
        });
        b.iter(|| black_box(bucket.try_acquire(black_box(1))));
    });
    group.finish();
}

fn bench_shared_contended(c: &mut Criterion) {
    let mut group = c.benchmark_group("TokenBucketShared/contended");
    for &threads in &[2usize, 4, 8] {
        group.throughput(Throughput::Elements(threads as u64 * 1_000));
        group.bench_with_input(
            BenchmarkId::from_parameter(threads),
            &threads,
            |b, &threads| {
                b.iter_custom(|iters| {
                    let bucket = Arc::new(TokenBucketShared::new(TokenBucketConfig {
                        capacity: u32::MAX,
                        refill_rate: u32::MAX,
                    }));
                    let start = std::time::Instant::now();
                    let handles: Vec<_> = (0..threads)
                        .map(|_| {
                            let bucket = Arc::clone(&bucket);
                            thread::spawn(move || {
                                for _ in 0..iters {
                                    black_box(bucket.try_acquire(black_box(1)));
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
