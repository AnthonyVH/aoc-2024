use nalgebra as na;
use rayon::prelude::*;
use std::simd::{cmp::SimdPartialOrd, num::SimdInt, Simd};

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
    const SIMD_SIZE: usize = 16;

    /// Calculate distance from any point on the race track to the end point.
    /// Returns the coordinates of the race track (in reverse order, i.e. from
    /// end to start!), and a map with distances to the end.
    fn calculate_race_path(maze: &util::Maze) -> (Vec<util::Coord>, na::DMatrix<u16>) {
        // Use a value for unreachable cells that can be added together with
        // another distance within the map without overflowing. This is used
        // later on to avoid having to call saturating_add() on SIMD elements.
        let unreachable_value = (maze.size().row * maze.size().col + 1) as u16;

        let mut reversed_path = Vec::new();
        let mut distances = na::DMatrix::from_element(
            maze.size().row as usize,
            maze.size().col as usize,
            unreachable_value,
        );

        distances[maze.end_pos] = 0;
        reversed_path.push(maze.end_pos);

        // NOTE: The race track is exactly that: a track, i.e. there's no side
        // branches or anything, just a single path.
        let mut cur_pos = maze.end_pos;

        // Do a BFS until each (reachable) cell has been visited.
        while cur_pos != maze.start_pos {
            // Only one of these directions is possible. It's the cell that's
            // not a wall, and not yet visited.
            cur_pos = SEARCH_DIRS
                .iter()
                .map(|offset| cur_pos + offset)
                .filter(|pos| maze.accessible(pos) && (distances[pos] == unreachable_value))
                .next()
                .unwrap();

            distances[cur_pos] = reversed_path.len() as u16;
            reversed_path.push(cur_pos);
        }

        assert_eq!(reversed_path.len() - 1, distances[maze.start_pos] as usize);
        (reversed_path, distances)
    }

    fn _num_masks_per_column(max_cheat_distance: u16) -> usize {
        (2 * max_cheat_distance + 1) as usize
    }

    fn _num_simd_words_per_column(max_cheat_distance: u16) -> usize {
        Self::_num_masks_per_column(max_cheat_distance).div_ceil(Self::SIMD_SIZE)
    }

    fn _expand_maze(prev_maze: &util::Maze, max_cheat_distance: u16) -> util::Maze {
        // Expand maze matrix, such that we never have to check for bounds.
        let maze_offset = util::Coord {
            row: max_cheat_distance as isize,
            col: max_cheat_distance as isize,
        };

        // nalgebra matrices are stored in column-major order. Hence when we
        // load multiple elements in a SIMD element, this happens in the row
        // direction (column per column). However, the number of words per SIMD
        // element might not nicely divide the max_cheat_distance. Which means
        // the "bottom" needs to be expanded more, to ensure that even for the
        // bottom-most cell in the original maze there is guaranteed no
        // out-of-bounds access when loading all "south" cells in SIMD elements.
        let maze_expansion: (usize, usize) = (
            Self::SIMD_SIZE * Self::_num_simd_words_per_column(max_cheat_distance),
            2 * max_cheat_distance as usize,
        );
        let mut expanded_maze = util::Maze {
            maze: na::DMatrix::from_element(
                prev_maze.maze.nrows() + maze_expansion.0,
                prev_maze.maze.ncols() + maze_expansion.1,
                false,
            ),
            start_pos: prev_maze.start_pos + maze_offset,
            end_pos: prev_maze.end_pos + maze_offset,
        };

        // Assign existing maze to the expanded one.
        expanded_maze
            .maze
            .view_mut(maze_offset.as_pair(), prev_maze.maze.shape())
            .copy_from(&prev_maze.maze);

        expanded_maze
    }

    fn _calculate_simd_masks(
        &self,
        max_cheat_distance: u16,
    ) -> Vec<Vec<Simd<u16, { Self::SIMD_SIZE }>>> {
        // Calculate SIMD masks for all the accessible cheat endpoints from a
        // given start point. Each entry in the mask contains its distance from
        // the origin point (Manhattan distance). Unreachable points are set to
        // one more than the maximum length of the path. Do this column-wise
        // because of the column-major storage of na::Matrix.
        let max_path_length = (self.maze.maze.nrows() * self.maze.maze.ncols()) as u16;
        let origin: util::Coord = (0, 0).into();

        let mut simd_masks = Vec::new();

        for col in -(max_cheat_distance as i16)..=(max_cheat_distance as i16) {
            // The first column (left-most) is only reachable by moving only to
            // the left. For the next columns, an extra cell is reachable above
            // and below the previously "highest" & "lowest" reachable cell. At
            // column 0, all cells in cheat distance are reachable.
            let mut column_masks = vec![
                Simd::splat(max_path_length);
                Self::_num_simd_words_per_column(max_cheat_distance)
            ];

            for row in -(max_cheat_distance as i16)..=(max_cheat_distance as i16) {
                let offset = util::Coord {
                    row: row.into(),
                    col: col.into(),
                };
                let dist_from_center = origin.manhattan_distance(&offset) as u16;

                if dist_from_center > max_cheat_distance {
                    continue;
                }

                let offset_row = (row + max_cheat_distance as i16) as usize;
                let element_idx = offset_row / Self::SIMD_SIZE;
                let word_idx = offset_row % Self::SIMD_SIZE;

                column_masks[element_idx][word_idx] = dist_from_center;
            }

            simd_masks.push(column_masks);
        }

        log::debug!("masks: {:?}", simd_masks);
        simd_masks
    }

    fn num_cheat_paths(&self, min_required_improvement: u16, max_cheat_distance: u16) -> u64 {
        assert!(self.maze.maze.nrows() < 255);
        assert!(self.maze.maze.ncols() < 255);

        // Expand maze matrix, such that we never have to check for bounds.
        let expanded_maze = Self::_expand_maze(&self.maze, max_cheat_distance);

        let (reversed_path, dist_from_end) = Self::calculate_race_path(&expanded_maze);
        let distance_masks = self._calculate_simd_masks(max_cheat_distance);

        // Loop over every step of the race path.
        // NOTE: Paths closer to the end than the minimum required improvement
        // can't improve enough on the solution, so skip those.
        reversed_path[min_required_improvement as usize..]
            .into_par_iter()
            .map(|pos| {
                let max_dist_to_end = Simd::splat(dist_from_end[pos] - min_required_improvement);
                let cheat_start_row = pos.row as usize - max_cheat_distance as usize;

                let mut num_valid_cheats = Simd::splat(0);

                // Process every column in the jump masks table.
                let columns = dist_from_end.columns(
                    (pos.col - max_cheat_distance as isize) as usize,
                    (2 * max_cheat_distance + 1) as usize,
                );
                for (column_masks, column) in distance_masks.iter().zip(columns.column_iter()) {
                    let col_slice = &column.as_slice()[cheat_start_row..];

                    for (cheat_distances, dists_chunk) in
                        column_masks.iter().zip(col_slice.chunks(Self::SIMD_SIZE))
                    {
                        let dists_to_end = Simd::from_slice(&dists_chunk);
                        let cheated_dist_to_end = dists_to_end + cheat_distances;
                        let is_valid_cheat = cheated_dist_to_end.simd_le(max_dist_to_end);

                        // An SIMD Mask converted to Simd has 0 for false and -1
                        // for true. So subtract to get the equivalent of adding
                        // a 1 for each matching comparison.
                        // NOTE: This is a loop-carried dependency. Trying to get
                        // rid of it by summing into an intermediate array slows
                        // things down by a factor of 30% though.
                        // The compiler seems to unroll this loop by 2 such that
                        // the loop-carried dependency doesn't slow things down.
                        num_valid_cheats -= is_valid_cheat.to_int();
                    }
                }

                num_valid_cheats.reduce_sum() as u64
            })
            .sum()
    }
}

fn solve_configurable(input: &str, min_time_saving: u16, max_cheat_time: u16) -> u64 {
    let problem: Problem = input.parse().unwrap();
    problem.num_cheat_paths(min_time_saving, max_cheat_time)
}

pub fn part_a(input: &str) -> u64 {
    const MIN_TIME_SAVING: u16 = 100;
    const MAX_CHEAT_TIME: u16 = 2;
    solve_configurable(input, MIN_TIME_SAVING, MAX_CHEAT_TIME)
}

pub fn part_b(input: &str) -> u64 {
    const MIN_TIME_SAVING: u16 = 100;
    const MAX_CHEAT_TIME: u16 = 20;
    solve_configurable(input, MIN_TIME_SAVING, MAX_CHEAT_TIME)
}

#[cfg(test)]
mod tests {
    #[test]
    fn example_a_distance() {
        util::run_test(|| {
            let expected: u16 = 84;
            let input = util::read_resource("example_20.txt").unwrap();
            let problem: crate::day_20::Problem = input.as_str().parse().unwrap();
            let (_, dist_to_end) = crate::day_20::Problem::calculate_race_path(&problem.maze);
            assert_eq!(dist_to_end[problem.maze.start_pos], expected);
        });
    }

    #[test]
    fn example_a() {
        util::run_test(|| {
            const MIN_TIME_SAVING: u16 = 20;
            const MAX_CHEAT_TIME: u16 = 2;
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
                        const MIN_TIME_SAVING: u16 = $min_time_saving;
                        const MAX_CHEAT_TIME: u16 = 20;
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
