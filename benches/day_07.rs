use criterion;

fn bench_part_a_iterative(bench: &mut criterion::Criterion) {
    let input: String = util::read_resource("day_07.txt").unwrap();
    bench.bench_function("Day 07 - Part A - Iterative", |b| {
        b.iter(|| aoc_2024::day_07::part_a(&input, true))
    });
}

fn bench_part_a_recursive(bench: &mut criterion::Criterion) {
    let input: String = util::read_resource("day_07.txt").unwrap();
    bench.bench_function("Day 07 - Part A - Recursive", |b| {
        b.iter(|| aoc_2024::day_07::part_a(&input, false))
    });
}

fn bench_part_b_iterative(bench: &mut criterion::Criterion) {
    let input: String = util::read_resource("day_07.txt").unwrap();
    bench.bench_function("Day 07 - Part B - Iterative", |b| {
        b.iter(|| aoc_2024::day_07::part_b(&input, true))
    });
}

fn bench_part_b_recursive(bench: &mut criterion::Criterion) {
    let input: String = util::read_resource("day_07.txt").unwrap();
    bench.bench_function("Day 07 - Part B - Recursive", |b| {
        b.iter(|| aoc_2024::day_07::part_b(&input, false))
    });
}

criterion::criterion_group!(
    benches,
    bench_part_a_iterative,
    bench_part_a_recursive,
    bench_part_b_iterative,
    bench_part_b_recursive
);
criterion::criterion_main!(benches);
