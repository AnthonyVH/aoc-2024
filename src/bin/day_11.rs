fn main() {
    util::init!();

    {
        let input: String = util::read_resource("day_11.txt").unwrap();
        println!("[Day 11 - Part A] {}", aoc_2024::day_11::part_a(&input))
    }

    {
        let input: String = util::read_resource("day_11.txt").unwrap();
        println!(
            "[Day 11 - Part B] {}",
            aoc_2024::day_11::part_b(&input, false)
        )
    }
}
