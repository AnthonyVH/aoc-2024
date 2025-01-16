use rayon::prelude::*;
use smallvec::SmallVec;

const NUM_ELEMENTS: usize = 12;

#[derive(Debug)]
struct Equation {
    target: u64,
    values: SmallVec<[u16; NUM_ELEMENTS]>,
}

#[derive(Copy, Clone, Debug)]
enum Operator {
    Add,
    Mult,
    Concat,
}

impl Operator {
    fn reverse_eval(self, lhs: u64, rhs: u64) -> Option<u64> {
        match self {
            Operator::Add => {
                assert!(lhs >= rhs); // Guaranteed by _solve_reverse() function.
                Some(lhs - rhs)
            }
            Operator::Mult => match lhs % rhs {
                0 => Some(lhs / rhs),
                _ => None,
            },
            Operator::Concat => {
                const DIVISORS: [u64; 3] = [10, 100, 1000];

                let divisor_idx = util::digit_width_base10(rhs) - 1;
                assert!(divisor_idx <= 2);
                let divisor = DIVISORS[divisor_idx as usize];

                match lhs % divisor == rhs {
                    false => None,
                    true => Some(lhs / divisor),
                }
            }
        }
    }
}

impl std::str::FromStr for Equation {
    type Err = std::string::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut ascii = s.as_bytes();
        let (target, pos) = atoi_simd::parse_any_pos(&ascii).unwrap();
        ascii = &ascii[pos + 1..]; // Skip colon (not space!).

        let mut values = SmallVec::new();
        while ascii.len() > 0 {
            let (value, pos) = atoi_simd::parse_any_pos(&ascii[1..]).unwrap();
            ascii = &ascii[pos + 1..]; // Also skip over initial space.
            values.push(value)
        }

        Ok(Equation { target, values })
    }
}

impl Equation {
    fn _solve_reversed(&self, target: u64, values: &[u16], operators: &[Operator]) -> bool {
        if values.len() == 1 {
            return values[0] as u64 == target;
        } else if target < values[values.len() - 1] as u64 {
            // If target is smaller than a value, then this can never solve.
            return false;
        }

        for op in operators.iter() {
            match op.reverse_eval(target, values[values.len() - 1] as u64) {
                None => continue,
                Some(next_target) => {
                    if self._solve_reversed(next_target, &values[..values.len() - 1], operators) {
                        return true;
                    }
                }
            }
        }

        false
    }

    fn solvable(&self, operators: &[Operator]) -> bool {
        // Solve from "back to front". I.e. start with the expected value and
        // work back through the list of values until the first one is reached
        // and matches the remaining expected value.
        self._solve_reversed(self.target, &self.values, operators)
    }
}

fn solve(input: &str, operators: &[Operator]) -> u64 {
    // Collect into vector to allow rayon to efficiently split objects across
    // its workers.
    let equations: Vec<_> = input.lines().map(|e| e.parse().unwrap()).collect();

    equations
        .par_iter()
        .filter(|eq: &&Equation| eq.solvable(operators))
        .map(|e| e.target)
        .sum()
}

pub fn part_a(input: &str) -> u64 {
    let operators = [Operator::Mult, Operator::Add];
    solve(input, &operators)
}

pub fn part_b(input: &str) -> u64 {
    let operators = [Operator::Concat, Operator::Mult, Operator::Add];
    solve(input, &operators)
}

#[cfg(test)]
mod tests {
    #[test]
    fn example_a() {
        util::run_test(|| {
            let expected: u64 = 3749;
            assert_eq!(
                crate::day_07::part_a(&util::read_resource("example_07.txt").unwrap()),
                expected
            );
        });
    }

    #[test]
    fn example_b() {
        util::run_test(|| {
            let expected: u64 = 11387;
            assert_eq!(
                crate::day_07::part_b(&util::read_resource("example_07.txt").unwrap()),
                expected
            );
        });
    }
}
