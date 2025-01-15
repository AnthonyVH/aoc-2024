use criterion;

fn bench_part_a(bench: &mut criterion::Criterion) {
    let input: String = util::read_resource("day_06.txt").unwrap();
    bench.bench_function("Day 6 - Part A", |b| {
        b.iter(|| aoc_2024::day_06::part_a(&input))
    });
}

fn bench_part_b(bench: &mut criterion::Criterion) {
    let input: String = util::read_resource("day_06.txt").unwrap();
    bench.bench_function("Day 6 - Part B", |b| {
        b.iter(|| aoc_2024::day_06::part_b(&input))
    });
}

criterion::criterion_group!(benches, bench_part_a, bench_part_b);
criterion::criterion_main!(benches);
