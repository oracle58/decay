use criterion::{Criterion, black_box, criterion_group};
use decay::core::Context;

/// Measures World resource insert + lookup.
/// Compare against raw HashMap to see the overhead of our abstraction.
fn world_resource_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("world_resources");

    group.bench_function("insert_1", |b| {
        b.iter(|| {
            let mut w = Context::new();
            w.insert_store(42u32);
            black_box(&w);
        });
    });

    group.bench_function("insert_10", |b| {
        b.iter(|| {
            let mut w = Context::new();
            w.insert_store(1u8);
            w.insert_store(2u16);
            w.insert_store(3u32);
            w.insert_store(4u64);
            w.insert_store(5i8);
            w.insert_store(6i16);
            w.insert_store(7i32);
            w.insert_store(8i64);
            w.insert_store(9f32);
            w.insert_store(10f64);
            black_box(&w);
        });
    });

    group.finish();
}

fn world_resource_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("world_resources");

    let mut w = Context::new();
    w.insert_store(42u32);
    w.insert_store(3.14f64);
    w.insert_store(String::from("hello"));

    group.bench_function("lookup_hit", |b| {
        b.iter(|| {
            black_box(w.store::<u32>());
            black_box(w.store::<f64>());
            black_box(w.store::<String>());
        });
    });

    group.bench_function("lookup_miss", |b| {
        b.iter(|| {
            black_box(w.store::<u8>());
            black_box(w.store::<Vec<u8>>());
        });
    });

    group.finish();
}

criterion_group!(benches, world_resource_insert, world_resource_lookup);
