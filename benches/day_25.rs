use criterion;

fn bench_part_a(bench: &mut criterion::Criterion) {
    let input: String = util::read_resource("day_25.txt").unwrap();
    bench.bench_function("Day 25 - Part A", |b| {
        b.iter(|| aoc_2024::day_25::part_a(&input))
    });
}

criterion::criterion_group!(benches, bench_part_a);
criterion::criterion_main!(benches);
