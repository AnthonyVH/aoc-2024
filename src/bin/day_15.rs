fn main() {
    util::init!();

    {
        let input: String = util::read_resource("day_15.txt").unwrap();
        println!(
            "[Day 15 - Part A] {}",
            aoc_2024::day_15::part_a(&input)
        )
    }

    {
        let input: String = util::read_resource("day_15.txt").unwrap();
        println!("[Day 15 - Part B] {}", aoc_2024::day_15::part_b(&input))
    }
}
