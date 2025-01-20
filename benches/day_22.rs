use criterion;

fn bench_part_a(bench: &mut criterion::Criterion) {
    // Initialize lookup tables which could be hard-coded (input-independent).
    aoc_2024::day_22::init();

    let input: String = util::read_resource("day_22.txt").unwrap();
    bench.bench_function("Day 22 - Part A", |b| {
        b.iter(|| aoc_2024::day_22::part_a(&input))
    });
}

fn bench_part_b(bench: &mut criterion::Criterion) {
    // Initialize lookup tables which could be hard-coded (input-independent).
    aoc_2024::day_22::init();

    let input: String = util::read_resource("day_22.txt").unwrap();
    bench.bench_function("Day 22 - Part B", |b| {
        b.iter(|| aoc_2024::day_22::part_b(&input))
    });
}

criterion::criterion_group!(benches, bench_part_a, bench_part_b);
criterion::criterion_main!(benches);
