use criterion;

fn bench_part_a(bench: &mut criterion::Criterion) {
    let room_size = util::Coord { row: 103, col: 101 };
    let input: String = util::read_resource("day_14.txt").unwrap();
    bench.bench_function("Day 14 - Part A", |b| {
        b.iter(|| aoc_2024::day_14::part_a(&input, room_size))
    });
}

fn bench_part_b(bench: &mut criterion::Criterion) {
    let input: String = util::read_resource("day_14.txt").unwrap();
    bench.bench_function("Day 14 - Part B", |b| {
        b.iter(|| aoc_2024::day_14::part_b(&input))
    });
}

criterion::criterion_group!(benches, bench_part_a, bench_part_b);
criterion::criterion_main!(benches);