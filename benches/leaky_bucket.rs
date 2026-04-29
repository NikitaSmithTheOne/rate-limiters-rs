use std::hint::black_box;
use std::sync::Arc;
use std::thread;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use rate_limiters::leaky_bucket::{LeakyBucket, LeakyBucketShared};
use rate_limiters::traits::{RateLimiter, RateLimiterShared};

fn bench_try_acquire_success(c: &mut Criterion) {
    let mut group = c.benchmark_group("LeakyBucket/try_acquire_success");
    group.throughput(Throughput::Elements(1));
    group.bench_function("single_token", |b| {
        // Effectively unlimited capacity + fast leak so each call succeeds.
        let mut bucket = LeakyBucket::new(u32::MAX, 1e12);
        b.iter(|| black_box(bucket.try_acquire(black_box(1))));
    });
    group.finish();
}

fn bench_try_acquire_rejected(c: &mut Criterion) {
    let mut group = c.benchmark_group("LeakyBucket/try_acquire_rejected");
    group.throughput(Throughput::Elements(1));
    group.bench_function("full_bucket", |b| {
        // Zero capacity + zero leak rate guarantees rejection.
        let mut bucket = LeakyBucket::new(0, 0.0);
        b.iter(|| black_box(bucket.try_acquire(black_box(1))));
    });
    group.finish();
}

fn bench_refresh(c: &mut Criterion) {
    let mut group = c.benchmark_group("LeakyBucket/refresh");
    for &rate in &[0.0f64, 1_000.0, 1_000_000.0] {
        group.bench_with_input(
            BenchmarkId::from_parameter(rate as u64),
            &rate,
            |b, &rate| {
                let mut bucket = LeakyBucket::new(u32::MAX, rate);
                b.iter(|| {
                    bucket.refresh();
                    black_box(&bucket);
                });
            },
        );
    }
    group.finish();
}

fn bench_getters(c: &mut Criterion) {
    let mut group = c.benchmark_group("LeakyBucket/getters");
    let bucket = {
        let mut b = LeakyBucket::new(1_000, 100.0);
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
    let mut group = c.benchmark_group("LeakyBucketShared/uncontended");
    group.throughput(Throughput::Elements(1));
    group.bench_function("try_acquire", |b| {
        let bucket = LeakyBucketShared::new(u32::MAX, 1e12);
        b.iter(|| black_box(bucket.try_acquire(black_box(1))));
    });
    group.finish();
}

fn bench_shared_contended(c: &mut Criterion) {
    let mut group = c.benchmark_group("LeakyBucketShared/contended");
    for &threads in &[2usize, 4, 8] {
        group.throughput(Throughput::Elements(threads as u64 * 1_000));
        group.bench_with_input(
            BenchmarkId::from_parameter(threads),
            &threads,
            |b, &threads| {
                b.iter_custom(|iters| {
                    let bucket = Arc::new(LeakyBucketShared::new(u32::MAX, 1e12));
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
