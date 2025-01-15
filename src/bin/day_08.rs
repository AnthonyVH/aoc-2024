fn main() {
    util::init!();

    {
        let input: String = util::read_resource("day_08.txt").unwrap();
        println!("[Day 08 - Part A] {}", aoc_2024::day_08::part_a(&input))
    }

    {
        let input: String = util::read_resource("day_08.txt").unwrap();
        println!("[Day 08 - Part B] {}", aoc_2024::day_08::part_b(&input))
    }
}