fn input_to_reports(input: &str) -> Vec<Vec<i32>> {
    input
        .lines()
        .map(|e| {
            e.trim()
                .split_whitespace()
                .map(|w| w.parse().unwrap())
                .collect()
        })
        .collect()
}

trait Report {
    fn is_safe(&self) -> bool;
    fn is_tolerable(&self) -> bool;
}

impl Report for Vec<i32> {
    fn is_safe(&self) -> bool {
        let diffs: Vec<_> = self.windows(2).map(|w| w[0] - w[1]).collect();
        let in_range: bool = diffs.iter().all(|e| e.abs() >= 1 && e.abs() <= 3);
        let is_monotonic: bool =
            diffs.iter().all(|e| e.signum() == -1) || diffs.iter().all(|e| e.signum() == 1);

        in_range && is_monotonic
    }

    fn is_tolerable(&self) -> bool {
        // Brute-force: if values aren't "safe", try removing one element at a
        // time and check if it's safe.
        self.is_safe()
            || (0..self.len()).any(|remove_idx: usize| {
                let shortened: Vec<_> = self
                    .iter()
                    .enumerate()
                    .filter(|(idx, _)| *idx != remove_idx)
                    .map(|(_, v)| *v)
                    .collect();
                shortened.is_safe()
            })
    }
}

pub fn part_a(input: &str) -> usize {
    input_to_reports(input)
        .iter()
        .filter(|e| e.is_safe())
        .count()
}

pub fn part_b(input: &str) -> usize {
    input_to_reports(input)
        .iter()
        .filter(|e| e.is_tolerable())
        .count()
}

#[cfg(test)]
mod tests {
    const INPUT: &'static str = "7 6 4 2 1
                                 1 2 7 8 9
                                 9 7 6 2 1
                                 1 3 2 4 5
                                 8 6 4 4 1
                                 1 3 6 7 9";

    #[test]
    fn example_a() {
        let expected: usize = 2;
        assert_eq!(crate::day_02::part_a(INPUT), expected);
    }

    #[test]
    fn example_b() {
        let expected: usize = 4;
        assert_eq!(crate::day_02::part_b(INPUT), expected);
    }
}
