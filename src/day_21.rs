use arrayvec::ArrayVec;
use itertools::Itertools;
use permutohedron::LexicalPermutation;
use rustc_hash::FxHashSet as HashSet;
use std::{array, sync::LazyLock};
use util::Direction;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct KeypadButton(u8);

struct NumericKeypad;
struct DirectionKeypad;

macro_rules! gen_possible_paths {
    ($class: ident, $to: ident, $from: ident) => {{
        // Unfortunately Rust seems to suck pretty hard at compile-time things,
        // or I just can't figure out how to write this in such a way that it's
        // generated at compile-time. So here we are...
        type PathPermutations = ArrayVec<PathVec, { $class::MAX_NUM_PATH_PERMUTATIONS }>;

        static PATHS: LazyLock<[[PathPermutations; $class::NUM_BUTTONS]; $class::NUM_BUTTONS]> =
            LazyLock::new(|| {
                array::from_fn(|from_idx| {
                    array::from_fn(|to_idx| {
                        $class::_permuted_paths(
                            KeypadButton(from_idx as u8),
                            KeypadButton(to_idx as u8),
                        )
                        .into_iter()
                        .collect()
                    })
                })
            });

        let from_button = Self::from_coord($from);
        let to_button = Self::from_coord($to);
        return &PATHS[from_button.0 as usize][to_button.0 as usize];
    }};
}

impl NumericKeypad {
    // A path is at most 5 moves, so max 5! / (3! * 2!) = 10 possible multi-set
    // permutations.
    const MAX_NUM_PATH_PERMUTATIONS: usize = 10;

    /// Return all possible path permutations between the given coordinates.
    fn possible_paths(from: util::Coord, to: util::Coord) -> &'static [PathVec] {
        gen_possible_paths!(NumericKeypad, to, from);
    }
}

impl DirectionKeypad {
    // A path is at most 3 moves, so max 3! / (2! * 1!) = 3 possible multi-set
    // permutations.
    const MAX_NUM_PATH_PERMUTATIONS: usize = 3;

    /// Return all possible path permutations between the given coordinates.
    fn possible_paths(from: util::Coord, to: util::Coord) -> &'static [PathVec] {
        gen_possible_paths!(DirectionKeypad, to, from);
    }
}

trait KeypadInfo {
    const NUM_BUTTONS: usize;
    const KEYPAD_BOUNDS: util::Coord;
    const FORBIDDEN_COORD: util::Coord;

    #[allow(dead_code)]
    fn to_ascii(button: KeypadButton) -> char;
    fn from_ascii(ascii: u8) -> KeypadButton;

    fn to_coord(button: KeypadButton) -> util::Coord;
    fn from_coord(pos: util::Coord) -> KeypadButton;

    fn _is_valid_path(mut start_pos: util::Coord, path: &[KeypadButton]) -> bool {
        assert!(path.len() >= 2);
        assert_eq!(path[0], DirectionKeypad::from_ascii(b'A'));
        assert_eq!(path[path.len() - 1], DirectionKeypad::from_ascii(b'A'));

        for button in path[1..path.len() - 1].iter() {
            let offset = match button {
                KeypadButton(0) => Direction::North,
                KeypadButton(2) => Direction::West,
                KeypadButton(3) => Direction::South,
                KeypadButton(4) => Direction::East,
                _ => unreachable!(),
            };
            start_pos += offset.into();

            if !start_pos.bounded_by(&Self::KEYPAD_BOUNDS) || (start_pos == Self::FORBIDDEN_COORD) {
                return false;
            }
        }

        true
    }

    /// Generate all possible permutations on paths between two buttons.
    fn _permuted_paths(from_button: KeypadButton, to_button: KeypadButton) -> Vec<PathVec> {
        // NOTE: Since this function is only called once for each pair of
        // arguments, its speed doesn't matter. I.e. it's executed less than 150
        // times.
        let from = Self::to_coord(from_button);
        let to = Self::to_coord(to_button);

        // Generate initial path.
        let mut diff = to - from;
        let mut path = PathVec::new();
        path.push(DirectionKeypad::from_ascii(b'A'));

        while diff.row != 0 {
            match diff.row.cmp(&0) {
                std::cmp::Ordering::Less => {
                    path.push(DirectionKeypad::from_ascii(b'^'));
                    diff.row += 1;
                }
                std::cmp::Ordering::Equal => (),
                std::cmp::Ordering::Greater => {
                    path.push(DirectionKeypad::from_ascii(b'v'));
                    diff.row -= 1;
                }
            }
        }

        while diff.col != 0 {
            match diff.col.cmp(&0) {
                std::cmp::Ordering::Less => {
                    path.push(DirectionKeypad::from_ascii(b'<'));
                    diff.col += 1;
                }
                std::cmp::Ordering::Equal => (),
                std::cmp::Ordering::Greater => {
                    path.push(DirectionKeypad::from_ascii(b'>'));
                    diff.col -= 1;
                }
            }
        }

        path.push(DirectionKeypad::from_ascii(b'A'));

        // Generate all permutations.
        let mut result: HashSet<PathVec> = HashSet::default();
        let path_length = path.len();

        path[1..path_length - 1].sort_unstable();

        loop {
            // If a path has more than 1 turn, it can't be an optimal one,
            // since extra turns require extra directional keypad presses.
            let num_turns = path[1..path_length - 1]
                .windows(2)
                .filter(|pair| pair[0] != pair[1])
                .count();

            // Don't store permutations that go along an impossible path.
            if (num_turns <= 1) && Self::_is_valid_path(from, &path) {
                result.insert(path.iter().copied().collect());
            }

            if !path[1..path_length - 1].next_permutation() {
                break;
            }
        }

        result.into_iter().collect()
    }
}

impl KeypadInfo for NumericKeypad {
    // NOTE: The functions here are not optimized, since their runtime is
    // completely unimportant compared to the total runtime.
    const NUM_BUTTONS: usize = 11;
    const KEYPAD_BOUNDS: util::Coord = util::Coord { row: 4, col: 3 };
    const FORBIDDEN_COORD: util::Coord = util::Coord { row: 3, col: 0 };

    fn to_ascii(button: KeypadButton) -> char {
        static LUT: [char; NumericKeypad::NUM_BUTTONS] =
            ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A'];
        LUT[button.0 as usize]
    }

    fn from_ascii(ascii: u8) -> KeypadButton {
        match ascii {
            b'0'..=b'9' => KeypadButton(ascii - b'0'),
            b'A' => KeypadButton(10),
            _ => unreachable!(),
        }
    }

    fn to_coord(button: KeypadButton) -> util::Coord {
        match button {
            KeypadButton(0) => util::Coord { row: 3, col: 1 },
            KeypadButton(1) => util::Coord { row: 2, col: 0 },
            KeypadButton(2) => util::Coord { row: 2, col: 1 },
            KeypadButton(3) => util::Coord { row: 2, col: 2 },
            KeypadButton(4) => util::Coord { row: 1, col: 0 },
            KeypadButton(5) => util::Coord { row: 1, col: 1 },
            KeypadButton(6) => util::Coord { row: 1, col: 2 },
            KeypadButton(7) => util::Coord { row: 0, col: 0 },
            KeypadButton(8) => util::Coord { row: 0, col: 1 },
            KeypadButton(9) => util::Coord { row: 0, col: 2 },
            KeypadButton(10) => util::Coord { row: 3, col: 2 },
            _ => unreachable!(),
        }
    }

    fn from_coord(pos: util::Coord) -> KeypadButton {
        match pos {
            util::Coord { row: 3, col: 1 } => KeypadButton(0),
            util::Coord { row: 2, col: 0 } => KeypadButton(1),
            util::Coord { row: 2, col: 1 } => KeypadButton(2),
            util::Coord { row: 2, col: 2 } => KeypadButton(3),
            util::Coord { row: 1, col: 0 } => KeypadButton(4),
            util::Coord { row: 1, col: 1 } => KeypadButton(5),
            util::Coord { row: 1, col: 2 } => KeypadButton(6),
            util::Coord { row: 0, col: 0 } => KeypadButton(7),
            util::Coord { row: 0, col: 1 } => KeypadButton(8),
            util::Coord { row: 0, col: 2 } => KeypadButton(9),
            util::Coord { row: 3, col: 2 } => KeypadButton(10),
            _ => unreachable!(),
        }
    }
}

impl KeypadInfo for DirectionKeypad {
    const NUM_BUTTONS: usize = 5;
    const KEYPAD_BOUNDS: util::Coord = util::Coord { row: 2, col: 3 };
    const FORBIDDEN_COORD: util::Coord = util::Coord { row: 0, col: 0 };

    fn to_ascii(button: KeypadButton) -> char {
        static LUT: [char; DirectionKeypad::NUM_BUTTONS] = ['^', 'A', '<', 'v', '>'];
        LUT[button.0 as usize]
    }

    fn from_ascii(ascii: u8) -> KeypadButton {
        // NOTE: Indices optimized to allow efficient from_coord lookups.
        match ascii {
            b'^' => KeypadButton(0),
            b'A' => KeypadButton(1),
            b'<' => KeypadButton(2),
            b'v' => KeypadButton(3),
            b'>' => KeypadButton(4),
            _ => unreachable!(),
        }
    }

    fn to_coord(button: KeypadButton) -> util::Coord {
        static LUT: [util::Coord; DirectionKeypad::NUM_BUTTONS] = [
            util::Coord { row: 0, col: 1 },
            util::Coord { row: 0, col: 2 },
            util::Coord { row: 1, col: 0 },
            util::Coord { row: 1, col: 1 },
            util::Coord { row: 1, col: 2 },
        ];
        LUT[button.0 as usize]
    }

    fn from_coord(pos: util::Coord) -> KeypadButton {
        // NOTE: Size optimized to allow fast bit-twidling index calculation.
        static LUT: [KeypadButton; 8] = [
            KeypadButton(u8::MAX),
            KeypadButton(0),
            KeypadButton(1),
            KeypadButton(u8::MAX),
            KeypadButton(2),
            KeypadButton(3),
            KeypadButton(4),
            KeypadButton(u8::MAX),
        ];

        let idx = ((pos.row as u8) << 2) | (pos.col as u8);
        LUT[idx as usize]
    }
}

// NOTE: The longest possible path is 7 steps (e.g. numeric keypad A to 7). This
// is actually 6 steps, but each path must also end in A. Hence the extra step.
const MAX_PATH_LENGTH: usize = 7;
type PathVec = ArrayVec<KeypadButton, MAX_PATH_LENGTH>;

struct SequenceFinder {
    // NOTE: Allowing for more parallelization on e.g. the iterations over the
    // path permutations with an RwLock around the cache makes everything much
    // slower.
    solution_cache:
        Vec<[[Option<u64>; DirectionKeypad::NUM_BUTTONS]; DirectionKeypad::NUM_BUTTONS]>,
}

impl SequenceFinder {
    fn new() -> SequenceFinder {
        SequenceFinder {
            solution_cache: Vec::default(),
        }
    }

    fn _find_shortest_path_permutation_on_directional_keypads(
        &mut self,
        remaining_direction_keypads: usize,
        permuted_paths: &[PathVec],
    ) -> u64 {
        assert_ne!(permuted_paths.len(), 0);
        match remaining_direction_keypads {
            0 => {
                // No need to test permutations, this is the point we just
                // "push" in the keys ourselves, so order doesn't matter, number
                // of presses is always the same.
                permuted_paths[0].len() as u64 - 1 // Don't count starting 'A'.
            }
            _ => {
                // For each possible path permutation, check which one results
                // in the shortest path with the given remaining # of direction
                // keypads.
                permuted_paths
                    .iter()
                    .map(|permuted_path| {
                        // Iterate over each adjacent pair of keys in the path.
                        permuted_path
                            .iter()
                            .tuple_windows()
                            .map(|(&from, &to)| {
                                self._search_shortest_path_on_directional_keypads(
                                    remaining_direction_keypads - 1,
                                    DirectionKeypad::to_coord(from),
                                    DirectionKeypad::to_coord(to),
                                )
                            })
                            .sum()
                    })
                    .min()
                    .unwrap()
            }
        }
    }

    fn _search_shortest_path_on_directional_keypads(
        &mut self,
        remaining_direction_keypads: usize,
        from: util::Coord,
        to: util::Coord,
    ) -> u64 {
        if let Some(solution) = self.solution_cache[remaining_direction_keypads]
            [DirectionKeypad::from_coord(from).0 as usize]
            [DirectionKeypad::from_coord(to).0 as usize]
        {
            return solution;
        }

        // No solution cached, so calculate the best solution path.
        let solution: u64 = self._find_shortest_path_permutation_on_directional_keypads(
            remaining_direction_keypads,
            DirectionKeypad::possible_paths(from, to),
        );

        // Cache result.
        self.solution_cache[remaining_direction_keypads]
            [DirectionKeypad::from_coord(from).0 as usize]
            [DirectionKeypad::from_coord(to).0 as usize] = Some(solution);

        log::debug!(
            "[cached] level: {}, arm pos: {} to {} => length: {}",
            remaining_direction_keypads,
            from,
            to,
            solution
        );

        solution
    }

    fn shortest_sequence_length(
        &mut self,
        num_direction_keypads: usize,
        targets: &[KeypadButton],
    ) -> u64 {
        // Initialize the caches.
        self.solution_cache
            .resize(num_direction_keypads, Default::default());

        // Starting on the 'A' button, for every adjacent pair of keys in the
        // target list, calculate the shortest path between those keys.
        [NumericKeypad::from_ascii(b'A')]
            .iter()
            .chain(targets.iter())
            .tuple_windows()
            .map(|(&from, &to)| {
                // NOTE: Don't bother with cache at top level, only worth it to
                // minimally speed up search for numbers with repeated digits.
                self._find_shortest_path_permutation_on_directional_keypads(
                    num_direction_keypads,
                    NumericKeypad::possible_paths(
                        NumericKeypad::to_coord(from),
                        NumericKeypad::to_coord(to),
                    ),
                )
            })
            .sum()
    }
}

pub fn shortest_chained_sequence(line: &str, num_direction_keypads: u8) -> u64 {
    // Convert ASCII buttons to button indices.
    log::debug!("Line: {}", line);
    let buttons: ArrayVec<KeypadButton, 4> = line
        .as_bytes()
        .iter()
        .map(|ascii| NumericKeypad::from_ascii(*ascii))
        .collect();

    // Chain one path finding operations per keypad. I.e. find the shortest path
    // for the given keypad, then find the shortest path to create that path
    // with the next keypad, etc.
    let mut solver = SequenceFinder::new();

    let solution = solver.shortest_sequence_length(num_direction_keypads as usize, &buttons);
    log::debug!("[{}] shortest path: {}", line, solution);
    solution
}

pub fn solve(input: &str, num_direction_keypads: u8) -> u64 {
    // NOTE: Running this in parallel is slightly slower.
    input
        .lines()
        .map(|line| (line, shortest_chained_sequence(line, num_direction_keypads)))
        .map(|(line, num_presses)| {
            let first_non_zero = line.find(|e| ('0'..='9').contains(&e)).unwrap();
            let first_last_digit = line
                .find(|e| !('0'..='9').contains(&e))
                .unwrap_or(line.len());
            let value: u64 = line[first_non_zero..first_last_digit].parse().unwrap();
            let complexity = value * num_presses;
            log::debug!("[{}] {} * {} = {}", line, value, num_presses, complexity);
            complexity
        })
        .sum()
}

pub fn part_a(input: &str) -> u64 {
    solve(input, 2)
}

pub fn part_b(input: &str) -> u64 {
    solve(input, 25)
}

#[cfg(test)]
mod tests {
    macro_rules! make_example_chain_test {
        ($test_subname: ident, $code: expr, $expected: expr) => {
            make_example_chain_test!(
                part_a_chain,
                $test_subname,
                $code,
                2,
                $expected
            );
        };
        ($test_subname: ident, $code: expr, $num_direction_keypads: expr, $expected: expr) => {
            make_example_chain_test!(partial, $test_subname, $code, $num_direction_keypads, $expected);
        };
        ($test_prefix_name: ident, $test_subname: ident, $code: expr, $num_direction_keypads: expr, $expected: expr) => {
            paste::item! {
                #[test]
                fn [< example_ $test_prefix_name _ $test_subname >] () {
                    util::run_test(|| {
                        // NOTE: There's multiple possible solutions. As long as
                        // the returned solution is minimal, it's OK.
                        let actual = crate::day_21::shortest_chained_sequence($code, $num_direction_keypads);
                        assert_eq!(actual, $expected as u64);
                    });
                }
            }
        };
    }

    make_example_chain_test!(chain_1, "029A", 0, "<A^A>^^AvvvA".len());
    make_example_chain_test!(chain_2, "029A", 1, "v<<A>>^A<A>AvA<^AA>A<vAAA>^A".len());

    make_example_chain_test!(
        code_1,
        "029A",
        "<vA<AA>>^AvAA<^A>A<v<A>>^AvA^A<vA>^A<v<A>^A>AAvA^A<v<A>A>^AAAvA<^A>A".len()
    );

    make_example_chain_test!(
        code_2,
        "980A",
        "<v<A>>^AAAvA^A<vA<AA>>^AvAA<^A>A<v<A>A>^AAAvA<^A>A<vA>^A<A>A".len()
    );

    make_example_chain_test!(
        code_3,
        "179A",
        "<v<A>>^A<vA<A>>^AAvAA<^A>A<v<A>>^AAvA^A<vA>^AA<A>A<v<A>A>^AAAvA<^A>A".len()
    );
    make_example_chain_test!(
        code_4,
        "456A",
        "<v<A>>^AA<vA<A>>^AAvAA<^A>A<vA>^A<A>A<vA>^A<A>A<v<A>A>^AAvA<^A>A".len()
    );

    make_example_chain_test!(
        code_5,
        "379A",
        "<v<A>>^AvA^A<vA<AA>>^AAvA<^A>AAvA^A<vA>^AA<A>A<v<A>A>^AAAvA<^A>A".len()
    );

    // Extra examples from https://www.reddit.com/r/adventofcode/comments/1hj6o1j/comment/m347iul.
    make_example_chain_test!(extra_code_1, "159A", 82);
    make_example_chain_test!(extra_code_2, "375A", 70);
    make_example_chain_test!(extra_code_3, "613A", 62);
    make_example_chain_test!(extra_code_4, "894A", 78);
    make_example_chain_test!(extra_code_5, "080A", 60);

    macro_rules! make_example_partial_chain_test {
        ($test_subname: ident, $num_direction_keypads: expr, $expected: expr) => {
            paste::item! {
                #[test]
                fn [< example_extra_ $test_subname >] () {
                    util::run_test(|| {
                        let actual = crate::day_21::solve(
                            &util::read_resource("example_21.txt").unwrap(),
                            $num_direction_keypads
                        );
                        assert_eq!(actual, $expected as u64);
                    });
                }
            }
        };
    }

    // Extra examples from https://www.reddit.com/r/adventofcode/comments/1hmafd7/comment/m3sn1i1.
    make_example_partial_chain_test!(partial_chain_01, 1, 53772);
    make_example_partial_chain_test!(partial_chain_02, 2, 126384);
    make_example_partial_chain_test!(partial_chain_03, 3, 310188);
    make_example_partial_chain_test!(partial_chain_04, 4, 757754);
    make_example_partial_chain_test!(partial_chain_05, 5, 1881090);
    make_example_partial_chain_test!(partial_chain_06, 6, 4656624);
    make_example_partial_chain_test!(partial_chain_07, 7, 11592556);
    make_example_partial_chain_test!(partial_chain_08, 8, 28805408);
    make_example_partial_chain_test!(partial_chain_09, 9, 71674912);
    make_example_partial_chain_test!(partial_chain_10, 10, 178268300);
    make_example_partial_chain_test!(partial_chain_11, 11, 443466162);
    make_example_partial_chain_test!(partial_chain_12, 12, 1103192296);
    make_example_partial_chain_test!(partial_chain_13, 13, 2744236806);
    make_example_partial_chain_test!(partial_chain_14, 14, 6826789418);
    make_example_partial_chain_test!(partial_chain_15, 15, 16982210284);
    make_example_partial_chain_test!(partial_chain_16, 16, 42245768606);
    make_example_partial_chain_test!(partial_chain_17, 17, 105091166058);
    make_example_partial_chain_test!(partial_chain_18, 18, 261427931594);
    make_example_partial_chain_test!(partial_chain_19, 19, 650334539256);
    make_example_partial_chain_test!(partial_chain_20, 20, 1617788558680);
    make_example_partial_chain_test!(partial_chain_21, 21, 4024453458310);
    make_example_partial_chain_test!(partial_chain_22, 22, 10011330575914);
    make_example_partial_chain_test!(partial_chain_23, 23, 24904446930002);
    make_example_partial_chain_test!(partial_chain_24, 24, 61952932092390);
    make_example_partial_chain_test!(partial_chain_25, 25, 154115708116294);

    macro_rules! make_example_single_code_partial_chain_test {
        ($test_subname: ident, $num_direction_keypads: expr, $expected_length: expr) => {
            paste::item! {
                #[test]
                fn [< single_code_extra_ $test_subname >] () {
                    util::run_test(|| {
                        let keypad_code = 4;
                        let input = format!("{}", keypad_code);
                        let actual_checksum = crate::day_21::solve(&input, $num_direction_keypads);
                        let actual_length : u64 = actual_checksum / keypad_code;
                        assert_eq!(actual_length, $expected_length);
                    });
                }
            }
        };
    }

    make_example_single_code_partial_chain_test!(partial_chain_01, 1, 11);
    make_example_single_code_partial_chain_test!(partial_chain_02, 2, 27);
    make_example_single_code_partial_chain_test!(partial_chain_03, 3, 65);
    make_example_single_code_partial_chain_test!(partial_chain_04, 4, 155);
    make_example_single_code_partial_chain_test!(partial_chain_05, 5, 383);
    make_example_single_code_partial_chain_test!(partial_chain_06, 6, 949);
    make_example_single_code_partial_chain_test!(partial_chain_07, 7, 2361);
    make_example_single_code_partial_chain_test!(partial_chain_08, 8, 5875);
    make_example_single_code_partial_chain_test!(partial_chain_09, 9, 14609);
    make_example_single_code_partial_chain_test!(partial_chain_10, 10, 36351);
    make_example_single_code_partial_chain_test!(partial_chain_11, 11, 90405);
    make_example_single_code_partial_chain_test!(partial_chain_12, 12, 224917);
    make_example_single_code_partial_chain_test!(partial_chain_13, 13, 559473);
    make_example_single_code_partial_chain_test!(partial_chain_14, 14, 1391793);
    make_example_single_code_partial_chain_test!(partial_chain_15, 15, 3462239);
    make_example_single_code_partial_chain_test!(partial_chain_16, 16, 8612739);
    make_example_single_code_partial_chain_test!(partial_chain_17, 17, 21425347);
    make_example_single_code_partial_chain_test!(partial_chain_18, 18, 53298053);
    make_example_single_code_partial_chain_test!(partial_chain_19, 19, 132585927);
    make_example_single_code_partial_chain_test!(partial_chain_20, 20, 329823647);
    make_example_single_code_partial_chain_test!(partial_chain_21, 21, 820478385);
    make_example_single_code_partial_chain_test!(partial_chain_22, 22, 2041042321);
    make_example_single_code_partial_chain_test!(partial_chain_23, 23, 5077349631);
    make_example_single_code_partial_chain_test!(partial_chain_24, 24, 12630544845);
    make_example_single_code_partial_chain_test!(partial_chain_25, 25, 31420065371);

    #[test]
    fn example_a() {
        util::run_test(|| {
            let expected: u64 = 126384;
            assert_eq!(
                crate::day_21::part_a(&util::read_resource("example_21.txt").unwrap()),
                expected
            );
        });
    }

    // No example for part B.
}
