use nalgebra as na;
use rayon::prelude::*;
use rustc_hash::FxHashMap as HashMap;

#[derive(Debug)]
struct Robot {
    position: util::Coord,
    velocity: util::Coord,
}

impl std::str::FromStr for Robot {
    type Err = std::string::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parse_coord = |s: &str| -> util::Coord {
            let start_x = s.find('=').unwrap() + 1;
            let end_x = start_x + s[start_x..].find(',').unwrap();
            let start_y = end_x + 1;
            util::Coord {
                row: s[start_y..].parse().unwrap(),
                col: s[start_x..end_x].parse().unwrap(),
            }
        };

        let mut coords = s.split_whitespace().map(|e| parse_coord(e));

        Ok(Robot {
            position: coords.next().unwrap(),
            velocity: coords.next().unwrap(),
        })
    }
}

impl Robot {
    fn step(&self, room_size: &util::Coord, num_steps: isize) -> util::Coord {
        let mut result = self.position + num_steps * self.velocity;
        // The % operation is remainer, we need modulo (i.e. always positive).
        result.row = result.row.rem_euclid(room_size.row);
        result.col = result.col.rem_euclid(room_size.col);
        result
    }
}

fn coord_to_quadrant(pos: util::Coord, room_size: util::Coord) -> Option<usize> {
    let calc_side = |e, room| match ((room - 1) as f64) / (e as f64) {
        value if value < 2. => Some(0),
        value if value > 2. => Some(1),
        _ => None,
    };

    let side_col = calc_side(pos.col, room_size.col);
    let side_row = calc_side(pos.row, room_size.row);
    log::debug!("{:?} => quadrant: ({:?}, {:?})", pos, side_col, side_row);

    match (side_col, side_row) {
        (Some(col), Some(row)) => Some(col + 2 * row),
        _ => None,
    }
}

pub static ROOM_SIZE: util::Coord = util::Coord { row: 103, col: 101 };

pub fn part_a_configurable(input: &str, room_size: util::Coord) -> usize {
    let robots: Vec<Robot> = input.lines().map(|e| e.parse().unwrap()).collect();

    const NUM_STEPS: isize = 100;
    let mut quadrant_count: [usize; 4] = [0; 4];

    robots
        .iter()
        .map(|e| e.step(&room_size, NUM_STEPS))
        .filter_map(|e| coord_to_quadrant(e, room_size))
        .for_each(|e| quadrant_count[e] += 1);
    log::debug!("Quadrant count: {:?}", quadrant_count);

    quadrant_count
        .iter()
        .copied()
        .reduce(|acc, e| acc * e)
        .unwrap()
}

pub fn part_a(input: &str) -> usize {
    part_a_configurable(input, ROOM_SIZE)
}

fn are_robots_clustered(robots: &[Robot], num_steps: usize, required_set_size: usize) -> bool {
    // Create a disjoint set to keep track of adjacent positions.
    // NOTE: We don't need/want to store any extra data in this set, so make
    // the type as short as possible.
    let mut set: partitions::PartitionVec<_> = partitions::partition_vec![(); robots.len()];
    let mut is_occupied: HashMap<util::Coord, usize> =
        HashMap::with_capacity_and_hasher(robots.len(), rustc_hash::FxBuildHasher);

    // NOTE: It would be great to do an early exit if we know there will be
    // more sets than required, i.e. the iteration won't be succesfull.
    // Unfortunately querying the number of sets is O(n), so this would just
    // massively slow things down.
    static SEARCH_DIRS: [util::Coord; 4] = [
        util::Direction::North.to_coord(),
        util::Direction::East.to_coord(),
        util::Direction::South.to_coord(),
        util::Direction::West.to_coord(),
    ];

    for (idx, robot) in robots.iter().enumerate() {
        let pos = robot.step(&ROOM_SIZE, num_steps as isize);
        is_occupied.insert(pos, idx);

        // Find occupied neighbors.
        // NOTE: This counts every robot, i.e. if multiple robots are on the
        // same position, they will all be counted individually.
        for offset in SEARCH_DIRS.iter() {
            let neighbor_pos = pos + offset;
            if let Some(neighbor_idx) = is_occupied.get(&neighbor_pos) {
                set.union(idx, *neighbor_idx);
            }
        }
    }

    // Find the largest set.
    let max_set_size = set.all_sets().map(|e| e.count()).max().unwrap();
    let is_clustered = max_set_size > required_set_size;
    if is_clustered {
        let mut positions =
            na::DMatrix::from_element(ROOM_SIZE.row as usize, ROOM_SIZE.col as usize, '.');
        for pos in is_occupied.keys() {
            positions[pos] = '#';
        }
        log::debug!("{}", positions);
    }

    is_clustered
}

pub fn part_b(input: &str) -> usize {
    // NOTE: This question is bullshit.
    let robots: Vec<Robot> = input.lines().map(|e| e.parse().unwrap()).collect();

    // Iterate steps until we find one for which the given percentage of robots
    // are in adjacent positions.
    // NOTE: This threshold is pretty random... Putting it at 50% doesn't work,
    // even though the problem statement says "most of the robots" should be
    // arranged in a picture of a Christmas tree...
    const REQUIRED_CLUSTER_PERCENTAGE: usize = 30;
    const NUM_PARALLEL_STEPS: usize = 128;

    let required_set_size = REQUIRED_CLUSTER_PERCENTAGE * robots.len() / 100;
    let mut num_steps: usize = 0;

    // NOTE: Iterating in chunks is faster than iterating over an endless range.
    // By almost a factor 6...
    loop {
        let first_match = (0..NUM_PARALLEL_STEPS)
            .par_bridge() // NOTE: This is somehow faster than into_par_iter().
            .find_first(|offset| {
                are_robots_clustered(&robots, num_steps + *offset, required_set_size)
            });

        match first_match {
            Some(offset) => return num_steps + offset,
            None => num_steps += NUM_PARALLEL_STEPS,
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn example_a() {
        util::run_test(|| {
            let example_room_size = util::Coord { row: 7, col: 11 };
            let expected: usize = 12;
            assert_eq!(
                crate::day_14::part_a_configurable(
                    &util::read_resource("example_14.txt").unwrap(),
                    example_room_size
                ),
                expected
            );
        });
    }

    // No example for part B.
}
