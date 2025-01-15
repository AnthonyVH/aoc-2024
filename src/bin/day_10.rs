fn main() {
    util::init!();

    {
        let input: String = util::read_resource("day_10.txt").unwrap();
        println!("[Day 10 - Part A] {}", aoc_2024::day_10::part_a(&input))
    }

    {
        let input: String = util::read_resource("day_10.txt").unwrap();
        println!("[Day 10 - Part B] {}", aoc_2024::day_10::part_b(&input))
    }
}
