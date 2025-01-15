fn main() {
    {
        let input: String = util::read_resource("day_04.txt").unwrap();
        println!("[Day 04 - Part A] {}", aoc_2024::day_04::part_a(&input))
    }

    {
        let input: String = util::read_resource("day_04.txt").unwrap();
        println!("[Day 04 - Part B] {}", aoc_2024::day_04::part_b(&input))
    }
}
