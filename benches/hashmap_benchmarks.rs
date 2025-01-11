use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::time::Duration;

use hashmap::workloads::{
    generators, HashMapBehavior, KeyDistributionWorkload, KeyPattern, LoadFactorWorkload,
    OperationMixWorkload,
};
use hashmap::{chaining, open_addressing};

// Benchmark scenarios
fn bench_load_factor<M: HashMapBehavior<String, String>>(c: &mut Criterion) {
    let mut group = c.benchmark_group("load_factor");
    group.measurement_time(Duration::from_secs(10));

    for size in [1000, 10_000, 100_000].iter() {
        let workload = LoadFactorWorkload {
            size: *size,
            value_size: 50,
        };
        group.bench_with_input(
            BenchmarkId::new(
                format!("{}_sequential_insert", std::any::type_name::<M>()),
                size,
            ),
            &workload,
            |b, workload| {
                b.iter(|| generators::run_load_factor_workload::<M>(workload));
            },
        );
    }
    group.finish();
}

fn bench_key_distribution<M: HashMapBehavior<String, String>>(c: &mut Criterion) {
    let mut group = c.benchmark_group("key_distribution");
    group.measurement_time(Duration::from_secs(10));

    let patterns = [
        (KeyPattern::Uniform, "uniform"),
        (KeyPattern::Clustered, "clustered"),
        (KeyPattern::Sequential, "sequential"),
    ];

    for (pattern, name) in patterns.iter() {
        let size = 1000;
        let workload = KeyDistributionWorkload {
            size,
            pattern: pattern.clone(),
        };

        group.bench_function(format!("{}_{}", std::any::type_name::<M>(), name), |b| {
            b.iter(|| generators::run_key_distribution_workload::<M>(&workload));
        });
    }

    group.finish();
}

fn bench_operation_mix<M: HashMapBehavior<String, String>>(c: &mut Criterion) {
    let mut group = c.benchmark_group("operation_mix");
    group.measurement_time(Duration::from_secs(10));

    let workloads = [
        (90, 5, "read_heavy"),
        (5, 90, "write_heavy"),
        (33, 33, "balanced"),
        (80, 15, "typical_web"),
    ];

    for (read_pct, write_pct, name) in workloads.iter() {
        let workload = OperationMixWorkload {
            initial_size: 1000,
            operations: 10_000,
            read_pct: *read_pct,
            write_pct: *write_pct,
        };

        group.bench_function(format!("{}_{}", std::any::type_name::<M>(), name), |b| {
            b.iter(|| generators::run_operation_mix_workload::<M>(&workload));
        });
    }

    group.finish();
}

fn criterion_benchmark(c: &mut Criterion) {
    // Run benchmarks for chained implementation
    bench_load_factor::<chaining::HashMap<_, _>>(c);
    bench_key_distribution::<chaining::HashMap<_, _>>(c);
    bench_operation_mix::<chaining::HashMap<_, _>>(c);

    // Run benchmarks for open addressing implementation
    bench_load_factor::<open_addressing::HashMap<_, _>>(c);
    bench_key_distribution::<open_addressing::HashMap<_, _>>(c);
    bench_operation_mix::<open_addressing::HashMap<_, _>>(c);
}

criterion_group!(
    name = benches;
    config = Criterion::default()
        .sample_size(10)
        .measurement_time(Duration::from_secs(30));
    targets = criterion_benchmark
);
criterion_main!(benches);
