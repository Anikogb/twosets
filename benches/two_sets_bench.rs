use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use twosets::two_sets::{
    greedy_prefix_fold_partitioner,
    congruence_pattern_constructor,
    generate_feasible_n_mod0,
    generate_feasible_n_mod3,
    generate_infeasible_n,
};

/// Benchmark harness comparing Algorithm A (GreedyPrefixFoldPartitioner) and
/// Algorithm B (CongruencePatternConstructor) across three input shapes and
/// five cardinality scales: 1 000, 10 000, 100 000, 500 000, 1 000 000.
fn benchmark_two_sets_algorithms(criterion: &mut Criterion) {
    let scales: &[usize] = &[1_000, 10_000, 100_000, 500_000, 1_000_000];

    // ── Shape 1: n ≡ 0 (mod 4) — paired high-low fill pattern ───────────────
    {
        let mut group = criterion.benchmark_group("TwoSets / n_mod4_eq_0 (Feasible)");
        for &scale in scales {
            let cardinality = generate_feasible_n_mod0(scale);

            group.bench_with_input(
                BenchmarkId::new("AlgorithmA_GreedyPrefixFold", cardinality),
                &cardinality,
                |bencher, &n| {
                    bencher.iter(|| greedy_prefix_fold_partitioner(black_box(n)))
                },
            );

            group.bench_with_input(
                BenchmarkId::new("AlgorithmB_CongruencePattern", cardinality),
                &cardinality,
                |bencher, &n| {
                    bencher.iter(|| congruence_pattern_constructor(black_box(n)))
                },
            );
        }
        group.finish();
    }

    // ── Shape 2: n ≡ 3 (mod 4) — singleton anchor + paired fill ─────────────
    {
        let mut group = criterion.benchmark_group("TwoSets / n_mod4_eq_3 (Feasible)");
        for &scale in scales {
            let cardinality = generate_feasible_n_mod3(scale);

            group.bench_with_input(
                BenchmarkId::new("AlgorithmA_GreedyPrefixFold", cardinality),
                &cardinality,
                |bencher, &n| {
                    bencher.iter(|| greedy_prefix_fold_partitioner(black_box(n)))
                },
            );

            group.bench_with_input(
                BenchmarkId::new("AlgorithmB_CongruencePattern", cardinality),
                &cardinality,
                |bencher, &n| {
                    bencher.iter(|| congruence_pattern_constructor(black_box(n)))
                },
            );
        }
        group.finish();
    }

    // ── Shape 3: n ≡ 1 (mod 4) — infeasible, early-exit path ─────────────────
    {
        let mut group = criterion.benchmark_group("TwoSets / n_mod4_eq_1 (Infeasible)");
        for &scale in scales {
            let cardinality = generate_infeasible_n(scale);

            group.bench_with_input(
                BenchmarkId::new("AlgorithmA_GreedyPrefixFold", cardinality),
                &cardinality,
                |bencher, &n| {
                    bencher.iter(|| greedy_prefix_fold_partitioner(black_box(n)))
                },
            );

            group.bench_with_input(
                BenchmarkId::new("AlgorithmB_CongruencePattern", cardinality),
                &cardinality,
                |bencher, &n| {
                    bencher.iter(|| congruence_pattern_constructor(black_box(n)))
                },
            );
        }
        group.finish();
    }
}

criterion_group!(benches, benchmark_two_sets_algorithms);
criterion_main!(benches);
