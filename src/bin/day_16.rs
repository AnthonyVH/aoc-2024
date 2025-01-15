fn main() {
    util::init!();

    {
        let input: String = util::read_resource("day_16.txt").unwrap();
        println!(
            "[Day 16 - Part A] {}",
            aoc_2024::day_16::part_a(&input)
        )
    }

    {
        let input: String = util::read_resource("day_16.txt").unwrap();
        println!("[Day 16 - Part B] {}", aoc_2024::day_16::part_b(&input))
    }
}
