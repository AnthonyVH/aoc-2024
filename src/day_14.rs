use nalgebra as na;
use std::simd::num::*;
use std::simd::*;

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

fn parse_robot_data(line: &str, room_size: util::Coord) -> ((u8, u8), (u8, u8)) {
    let ascii = line.as_bytes();

    let mut start_pos = ascii.iter().position(|&e| e == b'=').unwrap() + 1;
    let (pos_col, offset_next): (u8, _) = atoi_simd::parse_any_pos(&ascii[start_pos..]).unwrap();
    start_pos += offset_next + 1;
    let (pos_row, offset_next): (u8, _) = atoi_simd::parse_any_pos(&ascii[start_pos..]).unwrap();

    start_pos += offset_next + 1;
    start_pos += ascii[start_pos..].iter().position(|&e| e == b'=').unwrap() + 1;
    let (mut vel_col, offset_next): (i8, _) = atoi_simd::parse_any(&ascii[start_pos..]).unwrap();
    start_pos += offset_next + 1;
    let (mut vel_row, _): (i8, _) = atoi_simd::parse_any(&ascii[start_pos..]).unwrap();

    // Ensure all velocities are positive.
    vel_col += (vel_col < 0) as i8 * room_size.col as i8;
    vel_row += (vel_row < 0) as i8 * room_size.row as i8;

    ((pos_col, pos_row), (vel_col as u8, vel_row as u8))
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

const fn variance_swizzle_indices<const SIMD_LANES: usize, const OFFSET: usize>(
) -> [usize; SIMD_LANES] {
    let mut result = [0; SIMD_LANES];
    let mut i = 0; // Can't use a for-loop as that depends on traits, which can't be used in const fn.
    while i < SIMD_LANES {
        result[i] = i;
        i += 1;
    }
    result
}

fn calculate_dispersion_coefficient<const MODULO: u8>(
    positions: &[u8],
    velocities: &[u8],
    step: u8,
) -> f32 {
    // NOTE: Need to use const generics here, because no SIMD instruction exists
    // to divide/modulo with an integer. Falling back on floating point SIMD
    // doesn't speed up the calculations. However, by providing the modulus as a
    // constant, the compiler can convert the modulo operation in a sequence of
    // multiplications/shifts/subtractions, which does result in a massive
    // speed-up.
    assert!(step < MODULO);

    const SIMD_LANES: usize = 16;
    let simd_mod: Simd<u16, SIMD_LANES> = Simd::splat(MODULO as u16);
    let simd_step: Simd<u16, SIMD_LANES> = Simd::splat(step as u16);

    let mut simd_mean: Simd<u16, SIMD_LANES> = Simd::splat(0_u16);

    // The data in the variable array doesn't fit in 16-bit words. So store the
    // result in two 32-bit word arrays. This allows to still process twice as
    // many elements using 16-bit arithmetic.
    let mut simd_variance_lo: Simd<u32, { SIMD_LANES / 2 }> = Simd::splat(0_u32);
    let mut simd_variance_hi: Simd<u32, { SIMD_LANES / 2 }> = Simd::splat(0_u32);

    assert_eq!(positions.len(), velocities.len());
    let num_chunks = positions.len() / SIMD_LANES;

    for (pos, vel) in positions
        .chunks_exact(SIMD_LANES)
        .zip(velocities.chunks_exact(SIMD_LANES))
    {
        // If the chunk is not full sized, then the default elements are all 0,
        // which means they won't change the mean nor variance result.
        let simd_pos = Simd::from_slice(&pos).cast();
        let simd_vel = Simd::from_slice(&vel).cast();
        let simd_loc = (simd_pos + simd_step * simd_vel) % simd_mod;

        simd_mean += simd_loc;

        // Extract the 16-bit arrays into 32-bit ones and accumulate.
        let simd_loc_lo: Simd<u32, { SIMD_LANES / 2 }> = simd_swizzle!(
            simd_loc,
            variance_swizzle_indices::<{ SIMD_LANES / 2 }, { 0 * (SIMD_LANES / 2) }>()
        )
        .cast();
        simd_variance_lo += simd_loc_lo * simd_loc_lo;

        let simd_loc_hi: Simd<u32, { SIMD_LANES / 2 }> = simd_swizzle!(
            simd_loc,
            variance_swizzle_indices::<{ SIMD_LANES / 2 }, { 1 * (SIMD_LANES / 2) }>()
        )
        .cast();
        simd_variance_hi += simd_loc_hi * simd_loc_hi;
    }

    // Process elements that did not fit neatly into a chunked slice.
    let remaining_pos = &positions[num_chunks * SIMD_LANES..];
    let remaining_vel = &velocities[num_chunks * SIMD_LANES..];

    for (&pos, &vel) in remaining_pos.iter().zip(remaining_vel.iter()) {
        let loc = (pos as u16 + step as u16 * vel as u16) % MODULO as u16;
        simd_mean.as_mut_array()[0] += loc;
        simd_variance_lo.as_mut_array()[0] += loc as u32 * loc as u32;
    }

    let num_samples = positions.len() as u16;
    let mean = simd_mean.reduce_sum() / num_samples;
    let variance = ((simd_variance_lo.reduce_sum() + simd_variance_hi.reduce_sum())
        / num_samples as u32)
        - (mean as u32).pow(2);

    // Calculate statistical dispersion as variance divided by mean.
    variance as f32 / mean as f32
}

fn find_step_with_min_dispersion<const MODULO: u8>(positions: &[u8], velocities: &[u8]) -> u8 {
    // NOTE: Parallelization this makes things much slower.
    let num_steps = (0..MODULO)
        .map(|step| {
            (
                step,
                calculate_dispersion_coefficient::<MODULO>(&positions, &velocities, step),
            )
        })
        // Floats don't implement Ord, so we have to do this whole dance.
        .min_by(|lhs, rhs| lhs.1.total_cmp(&rhs.1))
        .unwrap()
        .0;
    log::debug!("min dispersion @ step {}", num_steps);
    num_steps
}

pub fn part_b(input: &str) -> usize {
    // NOTE: This solution is inspired by a comment on Reddit: the repetition of
    // the X- and Y-locations is independent. Everything else follows from this.
    // I.e. clustering can be detected in X & Y direction independently. The
    // solution is then the first step where lcm(x_step, y_step). For a ~5 ms
    // solution without external inspiration, check the Git commit history.

    // Store X & Y position & velocity separately, so they can be loaded faster
    // in SIMD structs later on.
    let ((robot_pos_col, robot_pos_row), (robot_vel_col, robot_vel_row)): (
        (Vec<u8>, Vec<u8>),
        (Vec<u8>, Vec<u8>),
    ) = input
        .lines()
        .map(|e| parse_robot_data(e, ROOM_SIZE))
        .unzip();

    // Detect step with maximum row and column clustering independently. The
    // robot locations repeat at most every respectively ROOM_SIZE.row or
    // ROOM_SIZE.col steps.
    // NOTE: Running this in parallel with rayon::join slows things down by a
    // factor of 2.
    let row_steps_remainder =
        find_step_with_min_dispersion::<{ ROOM_SIZE.row as u8 }>(&robot_pos_row, &robot_vel_row);
    let col_steps_remainder =
        find_step_with_min_dispersion::<{ ROOM_SIZE.col as u8 }>(&robot_pos_col, &robot_vel_col);

    // Now we know that given a solution of N steps, N modulo respectively the
    // room's number of rows or columns must equal one of the two values found.
    // To solve, use the Chinese remainder theorem:
    //   N == (row_steps_remainder + N * ROOM_SIZE.row)
    //      iif N % ROOM_SIZE.col == col_steps_remainder.
    // NOTE: The runtime of this loop is utterly negligible compared to the rest
    // of the code, no point in optimizing it.
    let num_steps = (0..ROOM_SIZE.col as u8)
        .map(|step| (row_steps_remainder as u16 + step as u16 * ROOM_SIZE.row as u16) as u16)
        .find(|e| *e % ROOM_SIZE.col as u16 == col_steps_remainder as u16)
        .unwrap();

    log::debug!("num steps => {}{}", num_steps, {
        let mut map =
            na::DMatrix::from_element(ROOM_SIZE.row as usize, ROOM_SIZE.col as usize, '.');
        let robots: Vec<Robot> = input.lines().map(|e| e.parse().unwrap()).collect();
        for robot in robots.iter() {
            map[robot.step(&ROOM_SIZE, num_steps as isize)] = '#'
        }
        map
    });

    num_steps as usize
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
