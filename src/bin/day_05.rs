fn main() {
    {
        let input: String = util::read_resource("day_05.txt").unwrap();
        println!("[Day 5 - Part A] {}", aoc_2024::day_05::part_a(&input))
    }

    {
        let input: String = util::read_resource("day_05.txt").unwrap();
        println!("[Day 5 - Part B] {}", aoc_2024::day_05::part_b(&input))
    }
}
