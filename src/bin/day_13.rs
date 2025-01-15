fn main() {
    util::init!();

    {
        let input: String = util::read_resource("day_13.txt").unwrap();
        println!("[Day 13 - Part A] {}", aoc_2024::day_13::part_a(&input))
    }

    {
        let input: String = util::read_resource("day_13.txt").unwrap();
        println!("[Day 13 - Part B] {}", aoc_2024::day_13::part_b(&input))
    }
}
