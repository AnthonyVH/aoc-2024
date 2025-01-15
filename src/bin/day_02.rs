fn main() {
    {
        let input: String = util::read_resource("day_02.txt").unwrap();
        println!("[Day 2 - Part A] {}", aoc_2024::day_02::part_a(&input))
    }

    {
        let input: String = util::read_resource("day_02.txt").unwrap();
        println!("[Day 2 - Part B] {}", aoc_2024::day_02::part_b(&input))
    }
}
