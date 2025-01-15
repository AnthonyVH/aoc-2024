extern crate nalgebra as na;

#[derive(Debug)]
struct Problem {
    // Using char instead of u8 is a small performance penalty (approx. 3%).
    warehouse: na::DMatrix<char>,
    moves: Vec<util::Direction>,
    robot_pos: util::Coord,
}

impl Problem {
    fn gps_coord(&self, coord: &util::Coord) -> usize {
        100 * (coord.row as usize) + (coord.col as usize)
    }

    fn gps_coord_sum(&self) -> usize {
        log::debug!("Summing warehouse: {}", self.warehouse);
        self.warehouse
            .iter()
            .enumerate()
            .filter_map(|(idx, e)| {
                let coord = util::Coord::from_column_major_index(
                    idx,
                    self.warehouse.nrows(),
                    self.warehouse.ncols(),
                );
                match e {
                    'O' | '[' => Some(self.gps_coord(&coord)),
                    _ => None,
                }
            })
            .sum()
    }

    fn widen(&mut self) {
        let widened_warehouse = na::DMatrix::from_row_iterator(
            self.warehouse.nrows(),
            2 * self.warehouse.ncols(),
            self.warehouse.row_iter().flatten().flat_map(|e| {
                match e {
                    '.' => "..",
                    '#' => "##",
                    'O' => "[]",
                    '@' => "@.",
                    _ => unreachable!(),
                }
                .chars()
            }),
        );
        self.warehouse = widened_warehouse;
        self.robot_pos.col *= 2;

        log::debug!("Widened:\n{}", self.warehouse);
    }

    fn move_robot<T>(&mut self, gather_to_move: T)
    where
        T: Fn(&mut Vec<util::Coord>, &na::DMatrix<char>, &util::Coord, &util::Coord),
    {
        let mut to_move: Vec<util::Coord> = Vec::new();

        for dir in &self.moves {
            log::debug!("Trying to move {:?} from {:?}", dir, self.robot_pos);

            // Try and move in the given direction. First gather up the robot
            // and all the boxes between the robot's position and the next empty
            // space or wall. If there's a wall, don't do anything. Else move
            // everything one step in the given direction.
            let offset: util::Coord = (*dir).into();

            to_move.clear();
            gather_to_move(&mut to_move, &self.warehouse, &self.robot_pos, &offset);

            if !to_move.is_empty() {
                // Move each cell one step, start at the back to prevent overwriting
                // values that still need to be read out.
                for pos in to_move.iter().rev() {
                    self.warehouse[pos + &offset] = self.warehouse[*pos];
                    self.warehouse[*pos] = '.';
                }

                // Update robot position.
                self.robot_pos += offset;

                log::debug!(
                    "Moved robot and {} boxes:\n{}",
                    to_move.len() - 1,
                    self.warehouse
                );
            }
        }
    }
}

impl std::str::FromStr for Problem {
    type Err = std::string::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let cols = s.lines().next().unwrap().len();
        let rows = s
            .lines()
            .enumerate()
            .skip_while(|(_, e)| e.len() != 0)
            .map(|(idx, _)| idx)
            .next()
            .unwrap();

        let mut result = Problem {
            warehouse: na::DMatrix::from_row_iterator(
                rows,
                cols,
                s.lines()
                    .take_while(|e| e.len() != 0)
                    .flat_map(|line| line.chars()),
            ),
            moves: s
                .lines()
                .skip_while(|e| e.len() != 0)
                .flat_map(|line| {
                    line.as_bytes().iter().map(|e| match e {
                        b'^' => util::Direction::North,
                        b'>' => util::Direction::East,
                        b'v' => util::Direction::South,
                        b'<' => util::Direction::West,
                        _ => unreachable!(),
                    })
                })
                .collect(),
            robot_pos: util::Coord { row: 0, col: 0 },
        };

        result.robot_pos = util::Coord::from_column_major_index(
            result
                .warehouse
                .iter()
                .enumerate()
                .filter_map(|(idx, e)| match *e == '@' {
                    true => Some(idx),
                    false => None,
                })
                .next()
                .unwrap(),
            rows,
            cols,
        );

        // Don't track robot position on map.
        //result.warehouse[result.robot_pos] = '.';

        Ok(result)
    }
}

pub fn part_a(input: &str) -> usize {
    let mut problem: Problem = input.parse().unwrap();
    log::debug!("{:?}", problem);

    let gather_to_move = |result: &mut Vec<util::Coord>,
                          warehouse: &na::DMatrix<char>,
                          robot_pos: &util::Coord,
                          offset: &util::Coord| {
        result.push(*robot_pos);
        let mut found_wall = false;

        loop {
            let next_coord = *robot_pos + (result.len() as isize) * *offset;
            log::debug!("Checking {:?}", next_coord);
            if next_coord.has_negatives() {
                break; // Out of range.
            } else {
                match warehouse.get(next_coord) {
                    None => break, // Out of range.
                    Some(e) => match e {
                        '#' => {
                            found_wall = true;
                            break;
                        }
                        '.' => break,
                        'O' => result.push(next_coord),
                        '@' => unreachable!(),
                        _ => unreachable!(),
                    },
                }
            }
        }

        if found_wall {
            result.clear();
        }
    };

    problem.move_robot(gather_to_move);
    problem.gps_coord_sum()
}

pub fn part_b(input: &str) -> usize {
    let mut problem: Problem = input.parse().unwrap();
    log::debug!("{:?}", problem);

    // Expand problem by making it twice as wide.
    problem.widen();

    let gather_to_move = |result: &mut Vec<util::Coord>,
                          warehouse: &na::DMatrix<char>,
                          robot_pos: &util::Coord,
                          offset: &util::Coord| {
        // For each step in the given direction, keep track of which boxes
        // were added. Then for each of those boxes, check the next row. If
        // a wall is found for any of them, abort.

        // Start by adding the robot position. It doesn't matter if we later
        // on insert a box there, because later in the same loop that box
        // gets removed again.

        result.push(*robot_pos);
        let mut added_boxes: &[util::Coord] = &result[0..1];
        let mut found_wall = false;

        // Keep track of whether the move is a horizontal one. If so, there
        // is no need to add the other half of a box explicitly. It will be
        // seen automatically in the next loop. Doing things this way allows
        // storing all boxes in a Vec (i.e. in contiguous memory), which
        // means the added_boxes variable can simply be a slice. If both
        // parts of a box would always be added, no matter the direction of
        // the move, then we'd have to make sure no part of the box was
        // added before and some kind of HashSet would be required.
        let is_horizontal_move = (offset == &Into::<util::Coord>::into(util::Direction::East))
            || (offset == &Into::<util::Coord>::into(util::Direction::West));

        loop {
            let mut new_boxes: Vec<util::Coord> = Vec::default();
            let mut all_spaces = true;

            for added_box in added_boxes {
                let next_coord = *added_box + *offset;
                log::trace!("Checking {:?}", next_coord);

                if next_coord.has_negatives() {
                    break; // Out of range.
                } else {
                    match warehouse.get(next_coord) {
                        None => break, // Out of range.
                        Some(e) => match e {
                            '#' => {
                                found_wall = true;
                                break;
                            }
                            '.' => continue,
                            '[' => {
                                new_boxes.push(next_coord);
                                if !is_horizontal_move {
                                    new_boxes.push(next_coord + util::Direction::East.into());
                                }
                            }
                            ']' => {
                                new_boxes.push(next_coord);
                                if !is_horizontal_move {
                                    new_boxes.push(next_coord + util::Direction::West.into());
                                }
                            }
                            '@' => unreachable!(),
                            _ => unreachable!(),
                        },
                    }
                }

                // Remember if a non-space was found for any added box.
                all_spaces = false;
            }

            // Remove duplicate elements. Since there's never a ton of
            // elements added, this is faster than using a HashSet.
            new_boxes.sort();
            new_boxes.dedup();

            let num_new_boxes = new_boxes.len();
            log::trace!("Adding {} new boxes: {:?}", num_new_boxes, new_boxes);
            result.extend(new_boxes);
            added_boxes = &result[result.len() - num_new_boxes..];
            log::trace!("Added boxes: {:?}", added_boxes);

            if found_wall || all_spaces {
                break;
            }
        }

        if found_wall {
            result.clear();
        }
    };

    problem.move_robot(gather_to_move);
    problem.gps_coord_sum()
}

#[cfg(test)]
mod tests {
    #[test]
    fn example_a_part_1() {
        util::run_test(|| {
            let expected: usize = 10092;
            assert_eq!(
                crate::day_15::part_a(&util::read_resource("example_15-part_1.txt").unwrap()),
                expected
            );
        });
    }

    #[test]
    fn example_a_part_2() {
        util::run_test(|| {
            let expected: usize = 2028;
            assert_eq!(
                crate::day_15::part_a(&util::read_resource("example_15-part_2.txt").unwrap()),
                expected
            );
        });
    }

    #[test]
    fn example_b() {
        util::run_test(|| {
            let expected: usize = 9021;
            assert_eq!(
                crate::day_15::part_b(&util::read_resource("example_15-part_1.txt").unwrap()),
                expected
            );
        });
    }

    #[test]
    fn example_b_no_answer() {
        util::run_test(|| {
            crate::day_15::part_b(&util::read_resource("example_15-part_3.txt").unwrap());
        });
    }
}
