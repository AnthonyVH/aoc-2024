use nalgebra as na;
use std::collections::VecDeque;

struct Problem {
    byte_pos: Vec<util::Coord>,
}

fn from_line(line: &str) -> util::Coord {
    let comma_pos = line.find(',').unwrap();
    util::Coord {
        row: line[0..comma_pos].parse().unwrap(),
        col: line[comma_pos + 1..].parse().unwrap(),
    }
}

impl std::str::FromStr for Problem {
    type Err = std::string::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Problem {
            byte_pos: s.lines().map(|e| from_line(e)).collect(),
        })
    }
}

impl Problem {
    fn path_length(&self, map_size: util::Coord, cur_time: usize) -> Option<usize> {
        let start_pos = util::Coord { row: 0, col: 0 };
        let end_pos = map_size - util::Coord { row: 1, col: 1 };

        // Just mark the obstacles, so we don't revisit them.
        let mut marked =
            na::DMatrix::from_element(map_size.row as usize, map_size.col as usize, false);

        for pos in self.byte_pos.iter().take(cur_time) {
            marked[pos] = true;
        }
        marked[start_pos] = true;

        // Just do BFS, no need for fancy stuff. Since each step costs the same,
        // this is the same as Dijkstra.
        let mut to_visit: VecDeque<(util::Coord, usize)> =
            VecDeque::with_capacity((map_size.row * map_size.col) as usize);
        to_visit.push_back((start_pos, 0));

        while let Some((pos, cost)) = to_visit.pop_front() {
            if pos == end_pos {
                return Some(cost);
            }

            // Visit all neighbors.
            const SEARCH_DIRS: [util::Direction; 4] = [
                util::Direction::North,
                util::Direction::East,
                util::Direction::South,
                util::Direction::West,
            ];

            for &offset_dir in SEARCH_DIRS.iter() {
                let next_pos: util::Coord = pos + offset_dir;
                if next_pos.bounded_by(&map_size) && !marked[next_pos] {
                    marked[next_pos] = true;
                    to_visit.push_back((next_pos, cost + 1));
                }
            }
        }

        None
    }
}

fn part_a_configurable(input: &str, map_size: util::Coord, cur_time: usize) -> usize {
    let problem: Problem = input.parse().unwrap();
    problem.path_length(map_size, cur_time).unwrap()
}

fn part_b_configurable(input: &str, map_size: util::Coord) -> String {
    let problem: Problem = input.parse().unwrap();

    // Binary search for the first time at which no more path is possible. Since
    // there's no way to binary search on a range, collect into a vector first,
    // because I'm lazy. Use partition_point() so the first match gets returned.
    let blocking_coord_idx = Vec::from_iter(0..problem.byte_pos.len())
        .as_slice()
        .partition_point(|idx| {
            let time = idx + 1;
            // The partition_point function expects true, true, ..., false. It's
            // the first element causing false we're looking for.
            let solution = problem.path_length(map_size, time);
            log::debug!("time: {}, solution: {:?}", time, solution,);
            solution.is_some()
        });
    let coord = &problem.byte_pos[blocking_coord_idx];
    format!("{},{}", coord.row, coord.col)
}

pub fn part_a(input: &str) -> usize {
    const MAP_SIZE: util::Coord = util::Coord { row: 71, col: 71 };
    const CUR_TIME: usize = 1024;
    part_a_configurable(input, MAP_SIZE, CUR_TIME)
}

pub fn part_b(input: &str) -> String {
    const MAP_SIZE: util::Coord = util::Coord { row: 71, col: 71 };
    part_b_configurable(input, MAP_SIZE)
}

#[cfg(test)]
mod tests {
    const MAP_SIZE: util::Coord = util::Coord { row: 7, col: 7 };

    #[test]
    fn example_a() {
        util::run_test(|| {
            const CUR_TIME: usize = 12;
            let expected: usize = 22;
            assert_eq!(
                crate::day_18::part_a_configurable(
                    &util::read_resource("example_18.txt").unwrap(),
                    MAP_SIZE,
                    CUR_TIME
                ),
                expected
            );
        });
    }

    #[test]
    fn example_b() {
        util::run_test(|| {
            let expected: &str = "6,1";
            assert_eq!(
                crate::day_18::part_b_configurable(
                    &util::read_resource("example_18.txt").unwrap(),
                    MAP_SIZE
                ),
                expected
            );
        });
    }
}
