use aoc_2024::day_11::{Execution, Looping, NUM_BLINKS_B};
use criterion;

fn bench_part_a(bench: &mut criterion::Criterion) {
    let input: String = util::read_resource("day_11.txt").unwrap();
    bench.bench_function("Day 11 - Part A", |b| {
        b.iter(|| aoc_2024::day_11::part_a(&input))
    });
}

fn bench_part_b(bench: &mut criterion::Criterion) {
    let input: String = util::read_resource("day_11.txt").unwrap();
    bench.bench_function("Day 11 - Part B", |b| {
        b.iter(|| aoc_2024::day_11::part_b(&input))
    });
}

fn bench_part_b_sequential_iterative(bench: &mut criterion::Criterion) {
    let input: String = util::read_resource("day_11.txt").unwrap();
    bench.bench_function("Day 11 - Part B - Sequential & Iterative", |b| {
        b.iter(|| {
            aoc_2024::day_11::parse_and_solve(
                &input,
                NUM_BLINKS_B,
                Execution::Sequential,
                Looping::Iterative,
            )
        })
    });
}

fn bench_part_b_sequential_recursive(bench: &mut criterion::Criterion) {
    let input: String = util::read_resource("day_11.txt").unwrap();
    bench.bench_function("Day 11 - Part B - Sequential & Recursive", |b| {
        b.iter(|| {
            aoc_2024::day_11::parse_and_solve(
                &input,
                NUM_BLINKS_B,
                Execution::Sequential,
                Looping::Recursive,
            )
        })
    });
}

fn bench_part_b_parallel_iterative(bench: &mut criterion::Criterion) {
    let input: String = util::read_resource("day_11.txt").unwrap();
    bench.bench_function("Day 11 - Part B - Parallel & Iterative", |b| {
        b.iter(|| {
            aoc_2024::day_11::parse_and_solve(
                &input,
                NUM_BLINKS_B,
                Execution::Parallel,
                Looping::Iterative,
            )
        })
    });
}

fn bench_part_b_parallel_recursive(bench: &mut criterion::Criterion) {
    let input: String = util::read_resource("day_11.txt").unwrap();
    bench.bench_function("Day 11 - Part B - Parallel & Recursive", |b| {
        b.iter(|| {
            aoc_2024::day_11::parse_and_solve(
                &input,
                NUM_BLINKS_B,
                Execution::Parallel,
                Looping::Recursive,
            )
        })
    });
}

criterion::criterion_group!(
    benches,
    bench_part_a,
    bench_part_b,
    bench_part_b_sequential_iterative,
    bench_part_b_sequential_recursive,
    bench_part_b_parallel_iterative,
    bench_part_b_parallel_recursive
);
criterion::criterion_main!(benches);
