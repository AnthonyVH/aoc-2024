use rayon::prelude::*;
use std::simd::{
    cmp::{SimdPartialEq, SimdPartialOrd},
    Simd,
};

type Heights = Simd<u8, 8>;

#[derive(Debug)]
struct Problem {
    locks: Vec<Heights>,
    keys: Vec<Heights>,
}

impl TryFrom<&str> for Problem {
    type Error = std::string::ParseError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let mut result = Problem {
            locks: Vec::new(),
            keys: Vec::new(),
        };

        const LINES_PER_ENTRY: usize = Problem::MAX_HEIGHT as usize + 2;
        for chunk in s
            .lines()
            .into_iter()
            .filter(|line| !line.is_empty())
            .array_chunks::<LINES_PER_ENTRY>()
        {
            let heights: Heights = chunk[1..LINES_PER_ENTRY - 1]
                .iter()
                .map(|line| -> Heights {
                    assert_eq!(line.len(), Problem::NUM_ELEM as usize);
                    let result = Simd::load_or_default(line.as_bytes());
                    let mask = result.simd_eq(Simd::splat(b'#'));
                    mask.select(Simd::splat(1u8), Simd::splat(0u8))
                })
                .fold(Heights::default(), |mut acc, iter| {
                    acc += iter;
                    acc
                });

            match chunk[0].as_bytes()[0] {
                b'#' => result.locks.push(heights),
                b'.' => result.keys.push(heights),
                _ => unreachable!(),
            }
        }

        Ok(result)
    }
}

impl Problem {
    const NUM_ELEM: u8 = 5;
    const MAX_HEIGHT: u8 = 5;

    fn overlap(lsh: &Heights, rhs: &Heights) -> bool {
        // NOTE: Storing the sum of elements and short-circuiting the element-
        // wise comparison if the sum of elements > NUM_ELEM * MAX_HEIGHT
        // doesn't really improve runtime.
        // TODO: Load multiple rhs'es and compare them all at once. E.g. with
        // 16 elements up to 3 rhs can be compared at the same time.
        (lsh + rhs).simd_gt(Simd::splat(Self::MAX_HEIGHT)).any()
    }
}

pub fn part_a(input: &str) -> u64 {
    let problem = Problem::try_from(input).unwrap();
    log::debug!("{:?}", problem);

    problem
        .locks
        .par_iter()
        .map(|lock| {
            problem
                .keys
                .iter()
                .filter(|key| Problem::overlap(lock, key))
                .count() as u64
        })
        .sum()
}

#[cfg(test)]
mod tests {
    #[test]
    fn example_a() {
        util::run_test(|| {
            let expected: u64 = 3;
            assert_eq!(
                crate::day_25::part_a(&util::read_resource("example_25.txt").unwrap()),
                expected
            );
        });
    }

    // No part B on the last problem.
}
