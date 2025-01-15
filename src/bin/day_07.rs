fn main() {
    util::init!();

    {
        let input: String = util::read_resource("day_07.txt").unwrap();
        println!("[Day 07 - Part A] {}", aoc_2024::day_07::part_a(&input, true))
    }

    {
        let input: String = util::read_resource("day_07.txt").unwrap();
        println!("[Day 07 - Part B] {}", aoc_2024::day_07::part_b(&input, true))
    }
}
