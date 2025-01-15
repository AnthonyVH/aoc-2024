use itertools::Itertools; // For: next_tuple()

#[derive(Clone, Debug)]
struct Problem {
    print_after: std::collections::HashMap<u32, Vec<u32>>,
    updates: Vec<Vec<u32>>,
}

impl Problem {
    fn new() -> Problem {
        Problem {
            print_after: std::collections::HashMap::new(),
            updates: Vec::new(),
        }
    }

    fn can_print_page(
        &self,
        page_printed: &std::collections::HashSet<u32>,
        update: &[u32],
        page: u32,
    ) -> Result<(), u32> {
        // Check if page can be printed. If not, return the first page on which it depends.

        if let Some(before_pages) = self.print_after.get(&page) {
            // If "before page" exist in the update it must have been printed before.
            for before_page in before_pages {
                if update.contains(before_page) && !page_printed.contains(before_page) {
                    return Err(*before_page);
                }
            }
        }

        // Page does not have dependencies, or all dependencies were met, so OK to print.
        Ok(())
    }

    fn try_print_page(
        &self,
        page_printed: &mut std::collections::HashSet<u32>,
        update: &[u32],
        page: u32,
    ) -> Result<(), u32> {
        let can_print = self.can_print_page(&page_printed, update, page);
        if can_print.is_ok() {
            page_printed.insert(page);
        }
        can_print
    }

    fn is_valid_update(&self, update: &[u32]) -> Result<(), (u32, u32)> {
        let mut page_printed = std::collections::HashSet::<u32>::new();
        for page in update {
            if let Err(before_page) = self.try_print_page(&mut page_printed, update, *page) {
                return Err((*page, before_page));
            }
        }
        Ok(())
    }

    fn make_valid_update(&self, update: &[u32]) -> Vec<u32> {
        let mut result: Vec<u32> = update.to_vec();

        // Brute-force: for each page in the update, check if it can be printed.
        // If not, move that page one to the back. Then retry the whole update.
        while let Err((page, before_page)) = self.is_valid_update(result.as_slice()) {
            // Move before_page before the current page.
            let page_idx = result.iter().position(|&e| e == page).unwrap();
            let before_page_idx = result.iter().position(|&e| e == before_page).unwrap();
            assert!(page_idx < before_page_idx);

            result[page_idx..=before_page_idx].rotate_right(1);
        }

        result
    }
}

impl std::str::FromStr for Problem {
    type Err = std::string::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut result = Problem::new();

        let first_section = s.lines().take_while(|e| !e.trim().is_empty());
        for line in first_section {
            let (before, after) = line
                .trim()
                .split("|")
                .map(|e| e.parse().unwrap())
                .next_tuple()
                .unwrap();

            // Create HashMap entry if it doesn't exist.
            result.print_after.entry(after).or_default().push(before);
        }

        // TODO: How to do this the nice way?
        let second_section = s.lines().skip_while(|e| !e.trim().is_empty()).skip(1);

        result.updates = second_section
            .map(|line| line.trim().split(",").map(|e| e.parse().unwrap()).collect())
            .collect();

        Ok(result)
    }
}

pub fn part_a(input: &str) -> usize {
    let problem: Problem = input.parse().unwrap();

    problem
        .updates
        .iter()
        .filter(|update| problem.is_valid_update(update).is_ok())
        .fold(0, |sum, update| sum + update[update.len() / 2] as usize)
}

pub fn part_b(input: &str) -> usize {
    let problem: Problem = input.parse().unwrap();

    problem
        .updates
        .iter()
        .filter(|update| problem.is_valid_update(update).is_err())
        .map(|update| problem.make_valid_update(update))
        .fold(0, |sum, update| sum + update[update.len() / 2] as usize)
}

#[cfg(test)]
mod tests {
    const INPUT: &'static str = "47|53
                                 97|13
                                 97|61
                                 97|47
                                 75|29
                                 61|13
                                 75|53
                                 29|13
                                 97|29
                                 53|29
                                 61|53
                                 97|53
                                 61|29
                                 47|13
                                 75|47
                                 97|75
                                 47|61
                                 75|61
                                 47|29
                                 75|13
                                 53|13

                                 75,47,61,53,29
                                 97,61,53,29,13
                                 75,29,13
                                 75,97,47,61,53
                                 61,13,29
                                 97,13,75,29,47";

    #[test]
    fn example_a() {
        let expected: usize = 143;
        assert_eq!(crate::day_05::part_a(INPUT), expected);
    }

    #[test]
    fn example_b() {
        let expected: usize = 123;
        assert_eq!(crate::day_05::part_b(INPUT), expected);
    }
}
