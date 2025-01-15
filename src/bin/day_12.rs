fn main() {
    util::init!();

    {
        let input: String = util::read_resource("day_12.txt").unwrap();
        println!("[Day 12 - Part A] {}", aoc_2024::day_12::part_a(&input))
    }

    {
        let input: String = util::read_resource("day_12.txt").unwrap();
        println!("[Day 12 - Part B] {}", aoc_2024::day_12::part_b(&input))
    }
}
