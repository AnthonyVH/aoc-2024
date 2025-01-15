use criterion;

fn bench_part_a(bench: &mut criterion::Criterion) {
    let input: String = util::read_resource("day_11.txt").unwrap();
    bench.bench_function("Day 11 - Part A", |b| {
        b.iter(|| aoc_2024::day_11::part_a(&input))
    });
}

fn bench_part_b_sequential(bench: &mut criterion::Criterion) {
    let input: String = util::read_resource("day_11.txt").unwrap();
    bench.bench_function("Day 11 - Part B - Sequential", |b| {
        b.iter(|| aoc_2024::day_11::part_b(&input, false))
    });
}

fn bench_part_b_parallel(bench: &mut criterion::Criterion) {
    let input: String = util::read_resource("day_11.txt").unwrap();
    bench.bench_function("Day 11 - Part B - Parallel", |b| {
        b.iter(|| aoc_2024::day_11::part_b(&input, true))
    });
}

criterion::criterion_group!(
    benches,
    bench_part_a,
    bench_part_b_sequential,
    bench_part_b_parallel
);
criterion::criterion_main!(benches);
