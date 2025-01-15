fn main() {
    util::init!();

    {
        let input: String = util::read_resource("day_09.txt").unwrap();
        println!("[Day 09 - Part A] {}", aoc_2024::day_09::part_a(&input))
    }

    {
        let input: String = util::read_resource("day_09.txt").unwrap();
        println!("[Day 09 - Part B] {}", aoc_2024::day_09::part_b(&input))
    }
}
