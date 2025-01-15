use std::cell::RefCell;

struct Problem<'a> {
    // Use trie to efficiently find all matching prefixes.
    patterns: yada::DoubleArray<Vec<u8>>,
    designs: Vec<&'a [u8]>,
}

impl<'a> From<&'a str> for Problem<'a> {
    fn from(s: &'a str) -> Self {
        let mut lines = s.lines();

        // Prepare pattern set for trie building.
        // NOTE: A value of type u32 required, so we just use zero everywhere,
        // since we're only interested in prefix lengths and we don't actually
        // need an associated value.
        let mut pattern_set: Vec<(&[u8], u32)> = lines
            .next()
            .unwrap()
            .split(", ")
            .map(|e| (e.as_bytes(), 0))
            .collect();

        // Sort patterns alphabetically, required for trie building.
        pattern_set.sort_by_key(|(k, _)| -> &[u8] { k });

        // Build the trie.
        let trie_builder = yada::builder::DoubleArrayBuilder::build(&pattern_set);

        Problem {
            patterns: yada::DoubleArray::new(trie_builder.unwrap()),
            designs: lines
                .skip_while(|e| e.is_empty())
                .map(|e| e.as_bytes())
                .collect(),
        }
    }
}

impl<'a> Problem<'a> {
    fn _is_design_possible(
        &self,
        design: &[u8],
        offset: usize,
        offset_possible: &RefCell<Vec<Option<bool>>>,
    ) -> bool {
        if let Some(success) = offset_possible.borrow()[offset] {
            return success;
        }

        // If there are any suffixes, and they can be matched, then there's a
        // match. Otherwise, no solution is possible for this haystack.
        let success = self
            .patterns
            .common_prefix_search(design)
            .any(|(_, prefix_length)| {
                self._is_design_possible(
                    &design[prefix_length as usize..],
                    offset + prefix_length as usize,
                    offset_possible,
                )
            });

        // Cache solution.
        offset_possible.borrow_mut()[offset] = Some(success);
        success
    }

    fn is_design_possible(&self, design: &[u8]) -> bool {
        // Prepare cache and prime it with success for zero length haystack.
        let offset_possible = RefCell::new(vec![None; design.len() + 1]);
        offset_possible.borrow_mut()[design.len()] = Some(true);

        self._is_design_possible(design, 0, &offset_possible)
    }

    fn _count_designs(
        &self,
        design: &[u8],
        offset: usize,
        offset_counts: &RefCell<Vec<Option<usize>>>,
    ) -> usize {
        if let Some(count) = offset_counts.borrow()[offset] {
            return count;
        }

        // Sum all solutions for matching suffixes in the haystack.
        let num_solutions = self
            .patterns
            .common_prefix_search(design)
            .map(|(_, prefix_length)| {
                self._count_designs(
                    &design[prefix_length as usize..],
                    offset + prefix_length as usize,
                    offset_counts,
                )
            })
            .sum();

        // Cache solution.
        offset_counts.borrow_mut()[offset] = Some(num_solutions);
        num_solutions
    }

    fn count_designs(&self, design: &[u8]) -> usize {
        // Create cache and prime it with 1 solution for a zero length haystack.
        let offset_counts = RefCell::new(vec![None; design.len() + 1]);
        offset_counts.borrow_mut()[design.len()] = Some(1);

        let result = self._count_designs(design, 0, &offset_counts);
        log::debug!(
            "# solutions for {}: {}",
            std::str::from_utf8(design).unwrap(),
            result
        );
        result
    }
}

pub fn part_a(input: &str) -> usize {
    let problem: Problem = input.into();
    problem
        .designs
        .iter()
        .filter(|e| problem.is_design_possible(e))
        .count()
}

pub fn part_b(input: &str) -> usize {
    let problem: Problem = input.into();
    problem
        .designs
        .iter()
        .map(|e| problem.count_designs(e))
        .sum()
}

#[cfg(test)]
mod tests {
    #[test]
    fn example_a() {
        util::run_test(|| {
            let expected: usize = 6;
            assert_eq!(
                crate::day_19::part_a(&util::read_resource("example_19.txt").unwrap()),
                expected
            );
        });
    }

    #[test]
    fn example_b() {
        util::run_test(|| {
            let expected: usize = 16;
            assert_eq!(
                crate::day_19::part_b(&util::read_resource("example_19.txt").unwrap()),
                expected
            );
        });
    }
}
