use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group};

/// Raw Vec<T> insert throughput. This is the ceiling for entity storage.
/// If our ECS spawn is 10x slower than a vec push, something is wrong.
fn vec_push_ceiling(c: &mut Criterion) {
    let mut group = c.benchmark_group("spawn_ceiling");

    for &count in &[1_000, 10_000, 100_000, 1_000_000] {
        group.throughput(Throughput::Elements(count as u64));

        group.bench_with_input(BenchmarkId::new("vec_push", count), &count, |b, &count| {
            b.iter(|| {
                let mut v: Vec<(f32, f32)> = Vec::with_capacity(count);
                for i in 0..count {
                    v.push((i as f32, i as f32 * 0.5));
                }
                black_box(&v);
            });
        });
    }

    group.finish();
}

/// HashMap<TypeId, Box<dyn Any>> insert throughput.
/// This is what World currently uses for resources, gives us a baseline
/// for how much the type-erased map costs.
fn hashmap_typeid_ceiling(c: &mut Criterion) {
    use std::any::{Any, TypeId};
    use std::collections::HashMap;

    let mut group = c.benchmark_group("spawn_ceiling");

    group.bench_function("hashmap_typeid_100_types", |b| {
        b.iter(|| {
            let mut map: HashMap<TypeId, Box<dyn Any>> = HashMap::new();
            // Simulate inserting different "types" via unique u64 keys
            for i in 0u64..100 {
                map.insert(
                    // Abuse TypeId by using the same type but different values
                    TypeId::of::<u64>(),
                    Box::new(i),
                );
            }
            black_box(&map);
        });
    });

    group.finish();
}

/// Measures dense array iteration throughput (what queries reduce to).
/// ECS query iteration should approach this speed.
fn dense_iteration_ceiling(c: &mut Criterion) {
    let mut group = c.benchmark_group("iteration_ceiling");

    for &count in &[1_000, 10_000, 100_000] {
        let positions: Vec<(f32, f32)> = (0..count).map(|i| (i as f32, i as f32)).collect();
        let velocities: Vec<(f32, f32)> =
            (0..count).map(|i| (i as f32 * 0.1, i as f32 * 0.2)).collect();
        group.throughput(Throughput::Elements(count as u64));

        group.bench_with_input(BenchmarkId::new("parallel_arrays", count), &count, |b, &count| {
            let mut pos = positions.clone();
            b.iter(|| {
                for i in 0..count as usize {
                    pos[i].0 += velocities[i].0;
                    pos[i].1 += velocities[i].1;
                }
                black_box(&pos);
            });
        });

        group.bench_with_input(BenchmarkId::new("iter_zip", count), &count, |b, _| {
            let mut pos = positions.clone();
            b.iter(|| {
                pos.iter_mut().zip(velocities.iter()).for_each(|(p, v)| {
                    p.0 += v.0;
                    p.1 += v.1;
                });
                black_box(&pos);
            });
        });
    }

    group.finish();
}

criterion_group!(benches, vec_push_ceiling, hashmap_typeid_ceiling, dense_iteration_ceiling);
