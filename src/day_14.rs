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
        .map(|e| {
            let mut end_pos = e.position + NUM_STEPS * e.velocity;
            // The % operation is remainer, we need modulo (i.e. always positive).
            end_pos.row = end_pos.row.rem_euclid(room_size.row);
            end_pos.col = end_pos.col.rem_euclid(room_size.col);
            end_pos
        })
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

pub fn part_b(_input: &str) -> usize {
    0
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
