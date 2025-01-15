use itertools::Itertools;
use nalgebra as na;
use std::collections::VecDeque;

struct Problem {
    maze: util::Maze,
}

impl std::str::FromStr for Problem {
    type Err = std::string::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Problem {
            maze: s.parse().unwrap(),
        })
    }
}

// NOTE: Rust doesn't allow statics in a struct's impl, so here we are.
static SEARCH_DIRS: [util::Coord; 4] = [
    util::Direction::North.to_coord(),
    util::Direction::East.to_coord(),
    util::Direction::South.to_coord(),
    util::Direction::West.to_coord(),
];

impl Problem {
    /// Calculate distance from given cell to every other cell in the maze.
    fn flood_fill(&self, from: util::Coord) -> na::DMatrix<u64> {
        let maze_size = self.maze.size();
        let mut result =
            na::DMatrix::from_element(maze_size.row as usize, maze_size.col as usize, u64::MAX);
        result[from] = 0;

        // NOTE: This capacity is about pi / 4 too large, since the maximum
        // cell we can reach is bounded by a circle, but whatever.
        let mut to_visit: VecDeque<util::Coord> =
            VecDeque::with_capacity((self.maze.size().row * self.maze.size().col) as usize);
        to_visit.push_back(from);

        // Do a BFS until each (reachable) cell has been visited.
        while let Some(pos) = to_visit.pop_front() {
            let next_dist = result[pos] + 1;
            for offset_dir in SEARCH_DIRS.iter() {
                let next_pos = pos + offset_dir;
                if self.maze.accessible(&next_pos) && (result[next_pos] == u64::MAX) {
                    result[next_pos] = next_dist;
                    to_visit.push_back(next_pos);
                }
            }
        }

        result
    }

    fn min_cheat_start_to_end_dist(
        &self,
        dist_from_start: &na::DMatrix<u64>,
        cheat_start: &util::Coord,
    ) -> u64 {
        // The minimum cheat path requires N steps to get to its start. Then it
        // takes another M steps (Manhattan distance) to get to the end, at
        // minimum. Don't bother trying any points for which this minimum
        // distance is too large.
        dist_from_start[cheat_start] + (cheat_start.manhattan_distance(&self.maze.end_pos) as u64)
    }

    /// Find all cheats that improve on the non-cheat distance. For each cheat,
    /// which is identified by it's start and end coordinate, only the minimum
    /// distance path is returned.
    fn num_cheat_paths(&self, min_required_improvement: u64, max_cheat_distance: u64) -> u64 {
        // NOTE: It's possible to speed this up by at least a factor 2 for
        // problem A, by making the implementation less generic. But eh...

        // First find all the shortest distances from the start to any point,
        // and from the end to any point. The distance from the start is used
        // to determine the distance required to get to a cheat's start. The
        // distance from the end is used to determine the distance required to
        // get to the end from a cheat's end.
        let dist_from_start = self.flood_fill(self.maze.start_pos);
        let dist_from_end = self.flood_fill(self.maze.end_pos);

        // A cheat should always improve on a non-cheat path.
        let max_allowed_cheat_dist = dist_from_start[self.maze.end_pos] - min_required_improvement;

        // Generate all offsets from a given point, up to a distance equal to
        // the maximum cheat distance. We can then use these offsets to find
        // possible cheats, instead of each time finding all possible paths
        // from a given start (which ends up generating the same offsets).
        let cheat_path_offsets = Self::_path_offsets(max_cheat_distance);

        // Visit every accessible cell. For each one, determine all possible
        // cheats. Since each of these cheats starts at that cell, there's no
        // other cheats starting at another cell that can be the same. Hence,
        // any cheat found starting at a given cell, is unique to that cell.
        // So there's no need to compare to cheats from other cells.

        // NOTE: Parallelizing the next chain speeds it up by a factor 5 for
        // part B. It slows it down by a factor 2.5 for part A though. So we
        // run it conditionally in parallel.
        const RUN_PARALLEL_THRESHOLD: u64 = 10;
        let run_parallel = max_cheat_distance >= RUN_PARALLEL_THRESHOLD;

        // Since Itertools::cartesian_product isn't compatible with rayon, just
        // generate a single range and convert it as an index into a Coord.
        let num_cells = (self.maze.size().row * self.maze.size().col) as usize;

        rayon_cond::CondIterator::new(0..num_cells, run_parallel)
            .map(|idx| {
                util::Coord::from_column_major_index(
                    idx,
                    self.maze.size().row as usize,
                    self.maze.size().col as usize,
                )
            })
            .filter(|pos| match self.maze.is_wall(pos) {
                true => false,
                false => {
                    self.min_cheat_start_to_end_dist(&dist_from_start, pos)
                        <= max_allowed_cheat_dist
                }
            })
            .map(|start_pos| {
                // Evaluate all cheat paths starting at this position.
                let dist_to_cheat_start = dist_from_start[start_pos];

                cheat_path_offsets
                    .iter()
                    .filter(|(offset, dist_to_cheat_end)| {
                        let cheat_end_pos = start_pos + offset;

                        // Only count cheats ending on a non-wall cell.
                        match self.maze.accessible(&cheat_end_pos) {
                            false => false,
                            true => {
                                let cheat_dist = dist_to_cheat_start
                                    + dist_to_cheat_end
                                    + dist_from_end[cheat_end_pos];
                                cheat_dist <= max_allowed_cheat_dist
                            }
                        }
                    })
                    .count() as u64
            })
            .sum()
    }

    /// Generate all offsets from a given point, up to a given distance.
    fn _path_offsets(max_distance: u64) -> Vec<(util::Coord, u64)> {
        // NOTE: The generated offsets are generated for all quadrants. Although
        // most offsets can simply be mirrored to get their "equivalents" in
        // other quadrants, this is not true when either the row or col offset
        // is zero. Hence, to keep iterating over these offsets as fast as
        // possible, all offsets are generated here.
        let mut result: Vec<(util::Coord, u64)> = (0..=max_distance)
            .cartesian_product(0..=max_distance)
            .map(|(row, col)| {
                const ZERO: util::Coord = util::Coord { row: 0, col: 0 };
                let pos = util::Coord {
                    row: row as isize,
                    col: col as isize,
                };
                let dist = ZERO.manhattan_distance(&pos) as u64;
                (pos, dist)
            })
            .filter(|(_, dist)| (*dist > 0) && (*dist <= max_distance))
            .map(|(pos, dist)| {
                // Generate all mirrored versions.
                [(1, 1), (-1, 1), (1, -1), (-1, -1)].map(|(row_mult, col_mult)| {
                    (
                        util::Coord {
                            row: pos.row * row_mult,
                            col: pos.col * col_mult,
                        },
                        dist,
                    )
                })
            })
            .flatten()
            .collect();

        result.sort_by_key(|&(pos, dist)| (dist, pos));
        result.dedup();

        result
    }
}

fn solve_configurable(input: &str, min_time_saving: u64, max_cheat_time: u64) -> u64 {
    let problem: Problem = input.parse().unwrap();
    problem.num_cheat_paths(min_time_saving, max_cheat_time)
}

pub fn part_a(input: &str) -> u64 {
    const MIN_TIME_SAVING: u64 = 100;
    const MAX_CHEAT_TIME: u64 = 2;
    solve_configurable(input, MIN_TIME_SAVING, MAX_CHEAT_TIME)
}

pub fn part_b(input: &str) -> u64 {
    const MIN_TIME_SAVING: u64 = 100;
    const MAX_CHEAT_TIME: u64 = 20;
    solve_configurable(input, MIN_TIME_SAVING, MAX_CHEAT_TIME)
}

#[cfg(test)]
mod tests {
    #[test]
    fn example_a_distance() {
        util::run_test(|| {
            let expected: u64 = 84;
            let input = util::read_resource("example_20.txt").unwrap();
            let problem: crate::day_20::Problem = input.as_str().parse().unwrap();
            let shortest_paths = problem.flood_fill(problem.maze.start_pos);
            assert_eq!(shortest_paths[problem.maze.end_pos], expected);
        });
    }

    #[test]
    fn example_a_num_faster() {
        util::run_test(|| {
            const MIN_TIME_SAVING: u64 = 20;
            const MAX_CHEAT_TIME: u64 = 2;
            let expected: u64 = 5;
            assert_eq!(
                crate::day_20::solve_configurable(
                    &util::read_resource("example_20.txt").unwrap(),
                    MIN_TIME_SAVING,
                    MAX_CHEAT_TIME
                ),
                expected
            );
        });
    }

    macro_rules! make_example_b_test {
        ($test_subname: ident, $min_time_saving: expr, $expected: expr) => {
            paste::item! {
                #[test]
                fn [< example_b_ $test_subname >] () {
                    util::run_test(|| {
                        const MIN_TIME_SAVING: u64 = $min_time_saving;
                        const MAX_CHEAT_TIME: u64 = 20;
                        let expected: u64 = $expected;
                        assert_eq!(
                            crate::day_20::solve_configurable(
                                &util::read_resource("example_20.txt").unwrap(),
                                MIN_TIME_SAVING,
                                MAX_CHEAT_TIME
                            ),
                            expected
                        );
                    });
                }
            }
        };
    }

    make_example_b_test!(subset_1, 76, 3);
    make_example_b_test!(subset_2, 74, 7);
    make_example_b_test!(subset_3, 72, 29);
    make_example_b_test!(subset_4, 50, 285);
}
