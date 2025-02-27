pub fn part_a(input: &str) -> u32 {
    let re = regex::Regex::new(r"mul\((\d+),(\d+)\)").unwrap();
    re.captures_iter(input)
        .map(|e| -> (u32, u32) {
            let (_, [l, r]) = e.extract();
            (l.parse().unwrap(), r.parse().unwrap())
        })
        .fold(0u32, |sum, (l, r)| sum + l * r)
}

pub fn part_b(input: &str) -> u32 {
    let re = regex::Regex::new(r"(mul|don't|do)\((?:(\d+),(\d+))?\)").unwrap();

    let mut enable_mult = true;
    re.captures_iter(input)
        .fold(0u32, |sum, capture| match &capture[1] {
            "mul" => {
                let l: u32 = capture[2].parse().unwrap();
                let r: u32 = capture[3].parse().unwrap();
                sum + enable_mult as u32 * l * r
            }
            "do" => {
                enable_mult = true;
                sum
            }
            "don't" => {
                enable_mult = false;
                sum
            }
            _ => panic!("Unexpected match"),
        })
}

#[cfg(test)]
mod tests {
    #[test]
    fn example_a() {
        let expected: u32 = 161;
        assert_eq!(crate::day_03::part_a(&util::read_resource("example_03-part_a.txt").unwrap()), expected);
    }

    #[test]
    fn example_b() {
        let expected: u32 = 48;
        assert_eq!(crate::day_03::part_b(&util::read_resource("example_03-part_b.txt").unwrap()), expected);
    }
}
