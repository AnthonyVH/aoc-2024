use rayon::prelude::*;

#[derive(Debug)]
struct Equation {
    target: u64,
    values: Vec<u64>,
}

#[derive(Copy, Clone, Debug)]
enum Operator {
    Add,
    Mult,
    Concat,
}

impl Operator {
    fn eval(self, lhs: u64, rhs: u64) -> u64 {
        match self {
            Operator::Add => lhs + rhs,
            Operator::Mult => lhs * rhs,
            Operator::Concat => {
                let lhs_shifted = lhs * 10_u64.pow(Operator::_digit_width_base10(rhs));
                lhs_shifted + rhs
            }
        }
    }

    fn _digit_width_base10(x: u64) -> u32 {
        x.ilog10() + 1
    }
}

impl std::str::FromStr for Equation {
    type Err = std::string::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut splits = s.split(": ");

        Ok(Equation {
            target: splits.next().unwrap().parse().unwrap(),
            values: splits
                .next()
                .unwrap()
                .split_whitespace()
                .map(|e| e.parse().unwrap())
                .collect(),
        })
    }
}

impl Equation {
    const MAX_STACK_STACK_SIZE: usize = 20;

    fn _recurse_solution(&self, result: u64, values: &[u64], operators: &[Operator]) -> bool {
        if values.is_empty() {
            return result == self.target;
        } else if result > self.target {
            return false; // Result too large to ever become a solution.
        }

        for op in operators.iter() {
            let next_result = op.eval(result, values[0]);
            if self._recurse_solution(next_result, &values[1..], operators) {
                return true; // Found a solution.
            }
        }

        false
    }

    /// Stack-based implementation of the recursive implementation. It's faster.
    fn _iterate_solution(&self, operators: &[Operator]) -> bool {
        struct State {
            result: u64,
            op_idx: u8,
            value_idx: u8,
        }

        let mut stack: Vec<State> = Vec::with_capacity(Self::MAX_STACK_STACK_SIZE);
        let mut found_solution = false;

        // Prime stack with first value.
        stack.push(State {
            result: self.values[0],
            op_idx: 0,
            value_idx: 1,
        });

        while !stack.is_empty() {
            let state = stack.last_mut().unwrap();

            if state.op_idx as usize == operators.len() {
                // Already evaluated all operators for this value.
                stack.pop();
            } else if state.value_idx as usize == self.values.len() {
                // Remember state.
                found_solution = state.result == self.target;

                // No more values to process.
                stack.pop();

                if found_solution {
                    break; // Solution was found, stop searching.
                }
            } else {
                // Try the next operator.
                let op = operators[state.op_idx as usize];
                let value = self.values[state.value_idx as usize];
                let next_result = op.eval(state.result, value);
                let next_value_idx = state.value_idx + 1;

                // Prepare next iteration of this "stack frame".
                state.op_idx += 1;

                if next_result <= self.target {
                    // Potential solution, push state to evaluate next value.
                    stack.push(State {
                        result: next_result,
                        op_idx: 0,
                        value_idx: next_value_idx as u8,
                    });
                }
            }
        }

        found_solution
    }

    fn solvable(&self, operators: &[Operator], looping: Looping) -> bool {
        match looping {
            Looping::Recursive => self._solvable_iterative(operators),
            Looping::Iterative => self._solvable_recursive(operators),
        }
    }

    fn _solvable_recursive(&self, operators: &[Operator]) -> bool {
        // More compiler hints == faster.
        assert!(self.values.len() >= 2);

        let possible = self._recurse_solution(self.values[0], &self.values[1..], operators);
        log::debug!("{:?} => {}", self, possible);
        possible
    }

    fn _solvable_iterative(&self, operators: &[Operator]) -> bool {
        let possible = self._iterate_solution(operators);
        log::debug!("{:?} => {}", self, possible);
        possible
    }
}

#[derive(Copy, Clone)]
pub enum Looping {
    Iterative,
    Recursive,
}

pub fn part_a_configurable(input: &str, looping: Looping) -> u64 {
    let problem: Vec<Equation> = input.lines().map(|e| e.parse().unwrap()).collect();
    let operators = [Operator::Add, Operator::Mult];

    problem
        .par_iter()
        .filter(|eq| eq.solvable(&operators, looping))
        .map(|e| e.target)
        .sum()
}

pub fn part_b_configurable(input: &str, looping: Looping) -> u64 {
    let problem: Vec<Equation> = input.lines().map(|e| e.parse().unwrap()).collect();
    let operators = [Operator::Add, Operator::Mult, Operator::Concat];

    problem
        .par_iter()
        .filter(|eq| eq.solvable(&operators, looping))
        .map(|e| e.target)
        .sum()
}

pub fn part_a(input: &str) -> u64 {
    part_a_configurable(input, Looping::Iterative)
}

pub fn part_b(input: &str) -> u64 {
    part_b_configurable(input, Looping::Iterative)
}

#[cfg(test)]
mod tests {
    use crate::day_07::Looping;

    const EXPECTED_A: u64 = 3749;
    const EXPECTED_B: u64 = 11387;

    #[test]
    fn example_a_recursive() {
        util::run_test(|| {
            assert_eq!(
                crate::day_07::part_a_configurable(
                    &util::read_resource("example_07.txt").unwrap(),
                    Looping::Recursive
                ),
                EXPECTED_A
            );
        });
    }

    #[test]
    fn example_a_iterative() {
        util::run_test(|| {
            assert_eq!(
                crate::day_07::part_a_configurable(
                    &util::read_resource("example_07.txt").unwrap(),
                    Looping::Iterative
                ),
                EXPECTED_A
            );
        });
    }

    #[test]
    fn example_b_recursive() {
        util::run_test(|| {
            assert_eq!(
                crate::day_07::part_b_configurable(
                    &util::read_resource("example_07.txt").unwrap(),
                    Looping::Recursive
                ),
                EXPECTED_B
            );
        });
    }

    #[test]
    fn example_b_iterative() {
        util::run_test(|| {
            assert_eq!(
                crate::day_07::part_b_configurable(
                    &util::read_resource("example_07.txt").unwrap(),
                    Looping::Iterative
                ),
                EXPECTED_B
            );
        });
    }
}
