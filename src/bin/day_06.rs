fn main() {
    util::init!();

    {
        let input: String = util::read_resource("day_06.txt").unwrap();
        println!("[Day 6 - Part A] {}", aoc_2024::day_06::part_a(&input))
    }

    {
        let input: String = util::read_resource("day_06.txt").unwrap();
        println!("[Day 6 - Part B] {}", aoc_2024::day_06::part_b(&input))
    }
}
