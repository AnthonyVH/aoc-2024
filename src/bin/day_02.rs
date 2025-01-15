fn main() {
    {
        let input: String = util::read_resource("day_02.txt").unwrap();
        println!("[Day 02 - Part A] {}", aoc_2024::day_02::part_a(&input))
    }

    {
        let input: String = util::read_resource("day_02.txt").unwrap();
        println!("[Day 02 - Part B] {}", aoc_2024::day_02::part_b(&input))
    }
}
