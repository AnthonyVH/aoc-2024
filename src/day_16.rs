use bit_vec::BitVec;
use nalgebra as na;
use radix_heap::RadixHeapMap;
use std::{array, collections::VecDeque};

#[derive(Debug)]
struct Problem {
    // NOTE: Replacing char with u8 somehow caused a slowdown.
    maze: na::DMatrix<char>,
    start_pos: util::Coord,
    end_pos: util::Coord,
}

#[derive(Debug)]
struct PathCell {
    pos: util::Coord,
    dir: util::Direction,
}

// Keep this around in case a standard BinaryHeap is required.
#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq)]
struct State {
    pos: util::Coord,
    dir: util::Direction,
    cost: usize,
}

impl Ord for State {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Invert ordering, so we get a min-heap.
        other
            .cost
            .cmp(&self.cost)
            .then(self.pos.cmp(&other.pos))
            .then(self.dir.cmp(&other.dir))
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

trait DirectionProperties {
    fn to_idx(&self) -> usize;
    fn from_idx(idx: usize) -> util::Direction;
    fn turns(&self) -> &[util::Direction; 2];
    fn reverse(&self) -> util::Direction;
}

impl DirectionProperties for util::Direction {
    fn to_idx(&self) -> usize {
        match self {
            util::Direction::North => 0,
            util::Direction::East => 1,
            util::Direction::South => 2,
            util::Direction::West => 3,
            _ => unreachable!(),
        }
    }

    fn from_idx(idx: usize) -> util::Direction {
        match idx {
            0 => util::Direction::North,
            1 => util::Direction::East,
            2 => util::Direction::South,
            3 => util::Direction::West,
            _ => unreachable!(),
        }
    }

    fn turns(&self) -> &[util::Direction; 2] {
        match self {
            util::Direction::North | util::Direction::South => {
                &[util::Direction::East, util::Direction::West]
            }
            util::Direction::East | util::Direction::West => {
                &[util::Direction::North, util::Direction::South]
            }
            _ => unreachable!(),
        }
    }

    fn reverse(&self) -> util::Direction {
        match self {
            util::Direction::North => util::Direction::South,
            util::Direction::East => util::Direction::West,
            util::Direction::South => util::Direction::North,
            util::Direction::West => util::Direction::East,
            _ => unreachable!(),
        }
    }
}

impl Problem {
    fn _find_cheapest_paths(&self) -> [na::DMatrix<usize>; 4] {
        // Just Dijkstra, keeping track from which direction a cell was visited.

        // NOTE: This priority queue requires that a key pushed to the heap must
        // be smaller than or equal to the previously popped key. So we simply
        // negate the cost when pushing new entries.
        let mut to_visit: RadixHeapMap<isize, PathCell> = RadixHeapMap::new();
        let mut costs: [na::DMatrix<usize>; 4] = array::from_fn(|_| {
            na::DMatrix::from_element(self.maze.nrows(), self.maze.ncols(), usize::MAX)
        });

        let maze_size: util::Coord = (self.maze.nrows(), self.maze.ncols()).into();

        // Prime priority queue.
        to_visit.push(
            0,
            PathCell {
                pos: self.start_pos,
                dir: util::Direction::East,
            },
        );

        while !to_visit.is_empty() {
            // Pop off cheapest path.
            let (cur_neg_cost, cur) = to_visit.pop().unwrap();
            let cur_cost = -cur_neg_cost as usize;
            log::debug!("Visiting {:?}", cur);

            if cur_cost >= costs[cur.dir.to_idx()][cur.pos] {
                continue; // Found better path for position & direction.
            }

            costs[cur.dir.to_idx()][cur.pos] = cur_cost;
            if cur.pos == self.end_pos {
                // If target is reached, bail out.
                log::debug!("Found end: {:?}", cur);
                break;
            }

            // Option: moving forward.
            {
                let next_pos = cur.pos + cur.dir;
                let next_cost = cur_cost + 1;

                let skip_next = next_pos.has_negatives()
                    || !next_pos.bounded_by(&maze_size)
                    || (self.maze[next_pos] == '#')
                    || (next_cost >= costs[cur.dir.to_idx()][next_pos]);

                if !skip_next {
                    to_visit.push(
                        -(next_cost as isize),
                        PathCell {
                            pos: next_pos,
                            dir: cur.dir,
                        },
                    );
                }
            }

            // Option: turn 90 degrees.
            for turn in cur.dir.turns() {
                let next_cost = cur_cost + 1000;

                if next_cost >= costs[turn.to_idx()][cur.pos] {
                    continue;
                }

                to_visit.push(
                    -(next_cost as isize),
                    PathCell {
                        pos: cur.pos,
                        dir: *turn,
                    },
                );
            }
        }

        costs
    }

    fn find_cheapest_path(&self) -> usize {
        itertools::min(self._find_cheapest_paths().map(|e| e[self.end_pos])).unwrap()
    }

    fn to_idx(&self, pos: &util::Coord) -> usize {
        (pos.row as usize) * self.maze.ncols() + (pos.col as usize)
    }

    fn _extract_num_paths_cells(&self, costs: &[na::DMatrix<usize>]) -> usize {
        // Walk from end position back to start and keep track of all possible
        // cheapest ways to get there.
        let mut path_cells: [BitVec; 4] =
            array::from_fn(|_| BitVec::from_elem(self.maze.nrows() * self.maze.ncols(), false));
        let mut to_visit: VecDeque<PathCell> = VecDeque::default();

        // Prime structs.
        let min_cost = itertools::min(costs.iter().map(|e| e[self.end_pos])).unwrap();
        to_visit.extend(costs.iter().enumerate().filter_map(|(idx, e)| {
            match e[self.end_pos] == min_cost {
                false => None,
                true => Some(PathCell {
                    pos: self.end_pos,
                    dir: <util::Direction as DirectionProperties>::from_idx(idx),
                }),
            }
        }));
        path_cells[to_visit[0].dir.to_idx()].set(self.to_idx(&to_visit[0].pos), true);

        while !to_visit.is_empty() {
            // Find all connected cells which were reached either with:
            //  - A cost of 1 less than the current cost, i.e. a forward step.
            //  - A cost of 1000 less than the current cost, i.e. a turn.
            let cur = to_visit.pop_front().unwrap();
            let cur_cost = costs[cur.dir.to_idx()][cur.pos];

            let mut add_if_match = |wanted_cost, to_push: PathCell| {
                if !path_cells[to_push.dir.to_idx()][self.to_idx(&to_push.pos)]
                    && (costs[to_push.dir.to_idx()][to_push.pos] == wanted_cost)
                {
                    log::debug!("Marking {:?}", to_push);
                    path_cells[to_push.dir.to_idx()].set(self.to_idx(&to_push.pos), true);
                    to_visit.push_back(to_push);
                }
            };

            // Option: step backwards.
            if cur_cost >= 1 {
                add_if_match(
                    cur_cost - 1,
                    PathCell {
                        pos: cur.pos + cur.dir.reverse(),
                        dir: cur.dir,
                    },
                );
            }

            // Option: turn 90 degrees.
            if cur_cost >= 1000 {
                for turn in cur.dir.turns() {
                    add_if_match(
                        cur_cost - 1000,
                        PathCell {
                            pos: cur.pos,
                            dir: *turn,
                        },
                    );
                }
            }
        }

        // Count all set bits over all directions.
        path_cells
            .iter_mut()
            .reduce(|acc, e| {
                acc.or(e);
                acc
            })
            .unwrap()
            .count_ones() as usize
    }

    fn find_num_path_cells(&self) -> usize {
        let costs = self._find_cheapest_paths();
        self._extract_num_paths_cells(&costs)
    }
}

impl std::str::FromStr for Problem {
    type Err = std::string::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let rows = s.lines().count();
        let cols = s.lines().next().unwrap().len();

        let mut start_idx: usize = 0;
        let mut end_idx: usize = 0;
        let mut result = Problem {
            maze: na::DMatrix::from_row_iterator(
                rows,
                cols,
                s.lines()
                    .flat_map(|line| line.chars())
                    .enumerate()
                    .inspect(|(idx, e)| match e {
                        'S' => start_idx = *idx,
                        'E' => end_idx = *idx,
                        _ => (),
                    })
                    .map(|(_, e)| e),
            ),
            start_pos: util::Coord { row: 0, col: 0 },
            end_pos: util::Coord { row: 0, col: 0 },
        };

        result.start_pos = util::Coord::from_row_major_index(start_idx, rows, cols);
        result.end_pos = util::Coord::from_row_major_index(end_idx, rows, cols);

        Ok(result)
    }
}

pub fn part_a(input: &str) -> usize {
    let problem: Problem = input.parse().unwrap();
    problem.find_cheapest_path()
}

pub fn part_b(input: &str) -> usize {
    let problem: Problem = input.parse().unwrap();
    problem.find_num_path_cells()
}

#[cfg(test)]
mod tests {
    #[test]
    fn example_a_part_1() {
        util::run_test(|| {
            let expected: usize = 7036;
            assert_eq!(
                crate::day_16::part_a(&util::read_resource("example_16-part_1.txt").unwrap()),
                expected
            );
        });
    }

    #[test]
    fn example_a_part_2() {
        util::run_test(|| {
            let expected: usize = 11048;
            assert_eq!(
                crate::day_16::part_a(&util::read_resource("example_16-part_2.txt").unwrap()),
                expected
            );
        });
    }

    #[test]
    fn example_b_part_1() {
        util::run_test(|| {
            let expected: usize = 45;
            assert_eq!(
                crate::day_16::part_b(&util::read_resource("example_16-part_1.txt").unwrap()),
                expected
            );
        });
    }

    #[test]
    fn example_b_part_2() {
        util::run_test(|| {
            let expected: usize = 64;
            assert_eq!(
                crate::day_16::part_b(&util::read_resource("example_16-part_2.txt").unwrap()),
                expected
            );
        });
    }
}
