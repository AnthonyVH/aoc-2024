fn main() {
    util::init!();

    {
        let room_size = util::Coord { row: 103, col: 101 };
        let input: String = util::read_resource("day_14.txt").unwrap();
        println!(
            "[Day 14 - Part A] {}",
            aoc_2024::day_14::part_a(&input, room_size)
        )
    }

    {
        let input: String = util::read_resource("day_14.txt").unwrap();
        println!("[Day 14 - Part B] {}", aoc_2024::day_14::part_b(&input))
    }
}
