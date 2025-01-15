use nalgebra as na;
use rayon::prelude::*;

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

#[derive(Clone)]
struct ClusteringCache {
    set: util::DisjointSetWithMaxSize,
    occupied: bit_vec::BitVec,
}

impl ClusteringCache {
    fn new() -> ClusteringCache {
        // Create a disjoint set for the whole grid, it's only 10k elements.
        // This way we don't have to keep track of which robot occupies which
        // cell, which would be necessary if we would join a set of robots
        // together instead.
        ClusteringCache {
            set: util::DisjointSetWithMaxSize::new((ROOM_SIZE.row * ROOM_SIZE.col) as u16),
            occupied: bit_vec::BitVec::from_elem((ROOM_SIZE.row * ROOM_SIZE.col) as usize, false),
        }
    }

    #[allow(dead_code)]
    fn reset(&mut self) {
        self.set.reset();
        self.occupied = bit_vec::BitVec::from_elem((ROOM_SIZE.row * ROOM_SIZE.col) as usize, false);
    }

    /// Helper function to convert a position to its index in the disjoint set.
    fn to_index(pos: util::Coord) -> u16 {
        (pos.row * ROOM_SIZE.col + pos.col) as u16
    }
}

fn are_robots_clustered(robots: &[Robot], num_steps: usize, required_set_size: usize) -> bool {
    // NOTE: It's slower to check for neighbors while calculating robot
    // positions, because in that case, the neighbors need to be checked in 4
    // directions instead of 2.

    // NOTE: Passing in a cache when running in parallel with rayon slows things
    // down a lot.
    let mut clustering_cache = ClusteringCache::new();
    let mut idx_and_pos: Vec<(u16, (u8, u8))> = vec![(0, (0, 0)); robots.len()];

    for (idx, robot) in robots.iter().enumerate() {
        let pos = robot.step(&ROOM_SIZE, num_steps as isize);
        let pos_idx = ClusteringCache::to_index(pos);
        idx_and_pos[idx] = (pos_idx as u16, (pos.row as u8, pos.col as u8));
        clustering_cache.occupied.set(pos_idx as usize, true);
    }

    static SEARCH_DIRS: [util::Coord; 2] = [
        util::Direction::East.to_coord(),
        util::Direction::South.to_coord(),
    ];

    for (pos_idx, (row, col)) in idx_and_pos.into_iter() {
        let pos = util::Coord {
            row: row as isize,
            col: col as isize,
        };

        for offset in SEARCH_DIRS.iter() {
            let neighbor_pos = pos + offset;
            if !neighbor_pos.bounded_by(&ROOM_SIZE) {
                continue;
            }

            let neighbor_idx = ClusteringCache::to_index(neighbor_pos);
            if clustering_cache
                .occupied
                .get(neighbor_idx as usize)
                .unwrap()
            {
                clustering_cache.set.union(pos_idx, neighbor_idx);
            }
        }
    }

    // Find the largest set.
    let is_clustered = clustering_cache.set.max_set_size() as usize > required_set_size;
    if is_clustered {
        log::debug!(
            "max cluster size: {}{}",
            clustering_cache.set.max_set_size(),
            na::DMatrix::<bool>::from_row_iterator(
                ROOM_SIZE.row as usize,
                ROOM_SIZE.col as usize,
                clustering_cache.occupied.iter()
            )
            .map(|e| if e { '#' } else { '.' })
        );
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
                are_robots_clustered(&robots, num_steps + offset, required_set_size)
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
