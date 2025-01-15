fn main() {
    {
        let input: String = util::read_resource("day_03.txt").unwrap();
        println!("[Day 3 - Part A] {}", aoc_2024::day_03::part_a(&input))
    }

    {
        let input: String = util::read_resource("day_03.txt").unwrap();
        println!("[Day 3 - Part B] {}", aoc_2024::day_03::part_b(&input))
    }
}
