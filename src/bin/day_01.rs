fn main() {
    {
        let input: String = util::read_resource("day_01.txt").unwrap();
        println!("[Day 1 - Part A] {}", aoc_2024::day_01::part_a(&input))
    }

    {
        let input: String = util::read_resource("day_01.txt").unwrap();
        println!("[Day 1 - Part B] {}", aoc_2024::day_01::part_b(&input))
    }
}
