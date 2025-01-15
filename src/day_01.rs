fn input_to_lists(input: &str) -> (Vec<u32>, Vec<u32>) {
    let mut col_left = vec![];
    let mut col_right = vec![];

    input
        .lines()
        .map(|e| e.trim().split_once(' ').unwrap())
        .for_each(|(l, r)| {
            col_left.push(str::parse::<u32>(l.trim()).unwrap());
            col_right.push(str::parse::<u32>(r.trim()).unwrap());
        });

    return (col_left, col_right);
}

pub fn part_a(input: &str) -> u32 {
    let (mut col_left, mut col_right) = input_to_lists(input);

    col_left.sort();
    col_right.sort();

    std::iter::zip(col_left.iter(), col_right.iter()).fold(0, |sum, (l, r)| sum + l.abs_diff(*r))
}

pub fn part_b(input: &str) -> u32 {
    let (col_left, col_right) = input_to_lists(input);
    let counts = col_right
        .iter()
        .fold(std::collections::HashMap::new(), |mut map, e| {
            *map.entry(e).or_insert(0) += 1;
            map
        });

    col_left
        .iter()
        .fold(0, |sum, e| sum + e * counts.get(e).or(Some(&0)).unwrap())
}

#[cfg(test)]
mod tests {
    const INPUT: &'static str = "3   4
                                 4   3
                                 2   5
                                 1   3
                                 3   9
                                 3   3";

    #[test]
    fn example_a() {
        let expected: u32 = 11;
        assert_eq!(crate::day_01::part_a(INPUT), expected);
    }

    #[test]
    fn example_b() {
        let expected: u32 = 31;
        assert_eq!(crate::day_01::part_b(INPUT), expected);
    }
}
