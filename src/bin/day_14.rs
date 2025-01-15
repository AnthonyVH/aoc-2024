fn main() {
    util::init!();

    {
        let input: String = util::read_resource("day_14.txt").unwrap();
        println!("[Day 14 - Part A] {}", aoc_2024::day_14::part_a(&input))
    }

    {
        let input: String = util::read_resource("day_14.txt").unwrap();
        println!("[Day 14 - Part B] {}", aoc_2024::day_14::part_b(&input))
    }
}
