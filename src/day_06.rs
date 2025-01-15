extern crate nalgebra as na;

trait DirectionUtils {
    const NUM_DIRECTIONS: usize;

    fn turn(self) -> util::Direction;
    fn index(self) -> usize;
    fn from(index: usize) -> util::Direction;
    fn mask(self) -> u8;
}

impl DirectionUtils for util::Direction {
    const NUM_DIRECTIONS: usize = 4;

    fn turn(self) -> util::Direction {
        match self {
            util::Direction::North => util::Direction::East,
            util::Direction::East => util::Direction::South,
            util::Direction::South => util::Direction::West,
            util::Direction::West => util::Direction::North,
            _ => unreachable!(),
        }
    }

    fn index(self) -> usize {
        match self {
            util::Direction::North => 0,
            util::Direction::East => 1,
            util::Direction::South => 2,
            util::Direction::West => 3,
            _ => unreachable!(),
        }
    }

    fn from(index: usize) -> util::Direction {
        match index {
            0 => util::Direction::North,
            1 => util::Direction::East,
            2 => util::Direction::South,
            3 => util::Direction::West,
            _ => unreachable!(),
        }
    }

    fn mask(self) -> u8 {
        match self {
            util::Direction::North => 1 << 0,
            util::Direction::East => 1 << 1,
            util::Direction::South => 1 << 2,
            util::Direction::West => 1 << 3,
            _ => unreachable!(),
        }
    }
}

#[derive(Copy, Clone, Debug)]
struct Guard {
    pos: util::Coord,
    dir: util::Direction,
}

impl std::str::FromStr for Guard {
    type Err = std::string::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let rows = s.lines().count();
        let cols = s.lines().next().unwrap().len();

        const GUARD_SYMBOL: u8 = b'^';
        let (index, _) = s
            .lines()
            .flat_map(|e| e.as_bytes().iter())
            .enumerate()
            .find(|(_, &e)| e == GUARD_SYMBOL)
            .unwrap();

        Ok(Self {
            pos: (index / cols, index % rows).into(),
            dir: util::Direction::North,
        })
    }
}

#[derive(Clone, Debug)]
struct StepTable {
    /// This table stores for each direction the number of steps to take until
    /// either an obstacle is reached, or we're out of bounds.
    steps_to_obstruction: [na::DMatrix<u8>; <util::Direction as DirectionUtils>::NUM_DIRECTIONS],
}

impl StepTable {
    const MARKER: u8 = u8::MAX;

    fn new(rows: usize, cols: usize) -> StepTable {
        // Need to be able to express up to MARKER - 1 steps. Since we want to be able
        // to represent a number of steps that takes us out of the map, that means the
        // step variable needs to hold a value up to max(row, col).
        assert!(rows < Self::MARKER as usize);
        assert!(cols < Self::MARKER as usize);

        let mut result = StepTable {
            steps_to_obstruction: std::array::from_fn(|_| {
                na::DMatrix::from_element(rows, cols, u8::MAX)
            }),
        };

        // Set correct distances to edge in each direction
        for (dir_idx, steps) in result.steps_to_obstruction.iter_mut().enumerate() {
            let dir = <util::Direction as DirectionUtils>::from(dir_idx);
            match dir {
                util::Direction::North => {
                    // Set columns from 1 to max.
                    for mut col in steps.column_iter_mut() {
                        for (row_idx, entry) in col.iter_mut().enumerate() {
                            *entry = (row_idx + 1).try_into().unwrap();
                        }
                    }
                }
                util::Direction::East => {
                    // Set rows from max to 1.
                    for mut row in steps.row_iter_mut() {
                        for (col_idx, entry) in row.iter_mut().enumerate() {
                            *entry = (cols - col_idx).try_into().unwrap();
                        }
                    }
                }
                util::Direction::South => {
                    // Set columns from max to 1.
                    for mut col in steps.column_iter_mut() {
                        for (row_idx, entry) in col.iter_mut().enumerate() {
                            *entry = (rows - row_idx).try_into().unwrap();
                        }
                    }
                }
                util::Direction::West => {
                    // Set rows from 1 to max.
                    for mut row in steps.row_iter_mut() {
                        for (col_idx, entry) in row.iter_mut().enumerate() {
                            *entry = (col_idx + 1).try_into().unwrap();
                        }
                    }
                }
                _ => unreachable!(),
            };
        }

        result
    }

    fn add_obstruction(&mut self, pos: util::Coord) {
        // Update all jump values on squares between new obstacle and previous one.
        // Step counts going forward in the "active" direction don't need to be
        // updated, because each cell already contains the number of steps to
        // the next obstacle.

        // Since we'll update the matrices in place, we first need to read all values
        // from the current state. Otherwise we might use updated values of one
        // direction to update the values of another direction.
        let steps_to_existing: [u8; <util::Direction as DirectionUtils>::NUM_DIRECTIONS] =
            std::array::from_fn(|dir_idx| {
                // Get the number of steps to go in the opposite direction from the
                // square just before the one that is getting an obstruction added.
                let dir = <util::Direction as DirectionUtils>::from(dir_idx);
                let backward_dir = dir.turn().turn();
                let backward_step: util::Coord = backward_dir.into();

                // The previous position indicates how many steps must be taken to
                // stand before the the next obstacle in the other direction, i.e. if this
                // value is 0, then the square after that is an obstacle.
                let prev_pos = pos + backward_step;

                match prev_pos.has_negatives() {
                    true => Self::MARKER, // Out of bounds, nothing to do.
                    false => match self.steps_to_obstruction[backward_dir.index()]
                        .get(prev_pos.as_pair())
                    {
                        None => Self::MARKER,                // Out of bounds, nothing to do.
                        Some(&Self::MARKER) => Self::MARKER, // Another obstacle in the way.
                        Some(&steps) => steps,
                    },
                }
            });

        for dir_idx in 0..self.steps_to_obstruction.len() {
            if steps_to_existing[dir_idx] == Self::MARKER {
                continue; // Nothing to do, we're at the edge.
            }

            // Get the number of steps to go in the opposite direction from the
            // square just before the one that is getting an obstruction added.
            let dir = <util::Direction as DirectionUtils>::from(dir_idx);
            let backward_dir = dir.turn().turn();
            let backward_step: util::Coord = backward_dir.into();

            // Update all squares between the previous obstacle and the new
            // one with the number of steps it requires to reach the new
            // obstacle.
            for step in 0..=steps_to_existing[dir_idx] {
                // We purposely set the number of steps such that if there's no obstacle
                // the walk will go out of bounds. Hence we need to make sure here that
                // we don't write to this out of bounds location.
                let prev_pos = pos + (step + 1) * backward_step;
                match prev_pos.has_negatives() {
                    true => break, // Reached out of bounds position.
                    false => {
                        match self.steps_to_obstruction[dir_idx].get_mut(prev_pos.as_pair()) {
                            None => break, // Out of bounds.
                            Some(prev_steps) => *prev_steps = step,
                        }
                    }
                }
            }
        }

        // Keep track of obstructions.
        for steps_to_obstruction in self.steps_to_obstruction.iter_mut() {
            steps_to_obstruction[pos.as_pair()] = Self::MARKER;
        }
    }

    fn remove_obstruction(&mut self, pos: util::Coord) {
        // Update all jump values on squares between new obstacle and previous one.
        // Step counts going forward in the "active" direction don't need to be
        // updated, because each cell already contains the number of steps to
        // the next obstacle.

        // Since we'll update the matrices in place, we first need to read all values
        // from the current state. Otherwise we might use updated values of one
        // direction to update the values of another direction.
        let update_info: [(u8, u8); <util::Direction as DirectionUtils>::NUM_DIRECTIONS] =
            std::array::from_fn(|dir_idx| {
                assert!(self.steps_to_obstruction[dir_idx][pos.as_pair()] == Self::MARKER);
                // Get the number of steps to go in the opposite direction from the
                // square just before the one that is getting an obstruction removed.
                let dir = <util::Direction as DirectionUtils>::from(dir_idx);
                let backward_dir = dir.turn().turn();

                let step: util::Coord = dir.into();
                let backward_step: util::Coord = backward_dir.into();

                // If the previous position is out of bounds, then we only need to update
                // the step count for the newly unobstructed square.
                let prev_pos = pos + backward_step;
                let cells_to_update = 1 + match prev_pos.has_negatives() {
                    true => 0, // Out of bounds position.
                    false => {
                        match self.steps_to_obstruction[backward_dir.index()]
                            .get(prev_pos.as_pair())
                        {
                            None => 0,                                           // Out of bounds position.
                            Some(&Self::MARKER) => 0, // Another obstacle in the way.
                            Some(&num_cells_backward) => num_cells_backward + 1, // One extra since steps go down to 0.
                        }
                    }
                };

                // Get the number of steps to the next obstacle in the forward direction.
                // If the next position is an out of bounds one, we want to make sure we step onto it.
                let next_pos = pos + step;
                let steps_offset = match next_pos.has_negatives() {
                    true => 1, // Out of bounds position.
                    false => {
                        match self.steps_to_obstruction[dir.index()].get(next_pos.as_pair()) {
                            None => 1,                                         // Out of bounds position.
                            Some(&Self::MARKER) => 0, // Another obstacle in the way.
                            Some(num_cells_forward) => *num_cells_forward + 1, // One extra because we're checking the next square.
                        }
                    }
                };

                (cells_to_update, steps_offset)
            });

        for dir_idx in 0..self.steps_to_obstruction.len() {
            // Update all squares between (and including) the newly unobstructed one and
            // the previous obstacle going backwards.
            let dir = <util::Direction as DirectionUtils>::from(dir_idx);
            let backward_dir = dir.turn().turn();
            let backward_step: util::Coord = backward_dir.into();
            let (cells_to_update, steps_offset) = update_info[dir_idx];

            for step in 0..cells_to_update {
                // We purposely set the number of steps such that if there's no obstacle
                // the walk will go out of bounds. Hence we need to make sure here that
                // we don't write to this out of bounds location.
                let prev_pos = pos + step * backward_step;
                match prev_pos.has_negatives() {
                    true => break, // Reached out of bounds position.
                    false => {
                        match self.steps_to_obstruction[dir_idx].get_mut(prev_pos.as_pair()) {
                            None => break, // Out of bounds.
                            Some(prev_steps) => *prev_steps = steps_offset + step,
                        }
                    }
                }
            }
        }
    }

    fn remaining_steps(&self, pos: util::Coord, dir: util::Direction) -> u8 {
        let result = self.steps_to_obstruction[dir.index()][pos.as_pair()];
        log::debug!("Steps going {:?} from {:?}: {}", dir, pos, result);
        assert!(result != Self::MARKER);
        result
    }

    fn is_obstructed(&self, pos: util::Coord) -> bool {
        // Doesn't matter which direction we check.
        self.steps_to_obstruction[0][pos.as_pair()] == Self::MARKER
    }
}

impl std::str::FromStr for StepTable {
    type Err = std::string::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let rows = s.lines().count();
        let cols = s.lines().next().unwrap().len();

        let mut result = StepTable::new(rows, cols);

        for (row, line) in s.lines().enumerate() {
            for (col, e) in line.as_bytes().iter().enumerate() {
                match e {
                    b'#' => result.add_obstruction((row, col).into()),
                    _ => (),
                }
            }
        }

        log::debug!("Steps:\n{}", result);
        Ok(result)
    }
}

impl std::fmt::Display for StepTable {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for (dir_idx, steps) in self.steps_to_obstruction.iter().enumerate() {
            let dir = <util::Direction as DirectionUtils>::from(dir_idx);
            match write!(f, "Steps {:?}:{}", dir, steps) {
                Ok(_) => (),
                Err(err) => return Err(err),
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
struct Problem {
    step_table: StepTable,
    guard: Guard,
    nrows: usize,
    ncols: usize,
}

impl std::str::FromStr for Problem {
    type Err = std::string::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let rows = s.lines().count();
        let cols = s.lines().next().unwrap().len();

        Ok(Self {
            step_table: s.parse().unwrap(),
            guard: s.parse().unwrap(),
            nrows: rows,
            ncols: cols,
        })
    }
}

struct Patrol {
    is_loop: bool,
    visited: na::DMatrix<u8>,
}

impl Patrol {
    fn new(rows: usize, cols: usize) -> Patrol {
        Patrol {
            is_loop: false,
            visited: na::DMatrix::from_element(rows, cols, 0),
        }
    }
}

impl Problem {
    /// Simulate the guard patrolling the lab. Calculation ends when a loop is
    /// detected, or the guard walks outside the lab.
    ///
    /// Returns a Patrol, indicating whether the patrol is a loop, and all
    /// squares visited by the guard, in the direction it was visited. I.e. it's
    /// possible the same square is visited from multiple directions.
    fn patrol_slow(&self) -> Patrol {
        let mut result = Patrol::new(self.nrows, self.ncols);
        let mut guard = self.guard;

        // Iterate until guard loops or goes out of bounds.
        loop {
            // Take a step in the current direction.
            match self.step_table.remaining_steps(guard.pos, guard.dir) {
                StepTable::MARKER => unreachable!(), // Somehow ended up on an obstruction.
                0 => {
                    // No more steps allowed in this direction, just turn.
                    guard.dir = guard.dir.turn();
                }
                _ => {
                    // Take a single step, so we can properly track all the visited squares.
                    guard.pos += util::Coord::from(guard.dir);
                }
            }

            match guard.pos.has_negatives() {
                true => break, // Went out of bound.
                false => {
                    match result.visited.get_mut(guard.pos.as_pair()) {
                        None => break, // Out of bounds.
                        Some(square_visited) => {
                            if (*square_visited & guard.dir.mask()) != 0 {
                                result.is_loop = true;
                                break; // Stop, guard was here before.
                            }

                            *square_visited |= guard.dir.mask();
                        }
                    }
                }
            }
        }

        result
    }

    /// Simulate the guard patrolling the lab by jumping around between the
    /// obstacles. Due to the jumping, we can't keep track of which intermediate
    /// squares have been jumped over, since that would slow us down.
    ///
    /// Returns whether the patrol is a loop.
    fn patrol_fast(&self) -> bool {
        let mut result = Patrol::new(self.nrows, self.ncols);
        let mut guard = self.guard;

        // Iterate until guard loops or goes out of bounds.
        loop {
            // Look up the number of steps to the next obstruction.
            match self.step_table.remaining_steps(guard.pos, guard.dir) {
                StepTable::MARKER => unreachable!(), // Somehow ended up on an obstruction.
                steps => {
                    // Jump to the next obstruction, and then already turn in
                    // preparation for the next jump. Note that the jump can
                    // have a length of zero.
                    guard.pos += steps * util::Coord::from(guard.dir);
                    guard.dir = guard.dir.turn();
                }
            }

            // Check if our new position is still in bounds.
            match guard.pos.has_negatives() {
                true => break, // Out of bounds.
                false => {
                    match result.visited.get_mut(guard.pos.as_pair()) {
                        None => break, // Out of bounds.
                        Some(square_visited) => {
                            if (*square_visited & guard.dir.mask()) != 0 {
                                result.is_loop = true;
                                break; // Stop, guard was here before.
                            }

                            *square_visited |= guard.dir.mask();
                        }
                    }
                }
            }
        }

        result.is_loop
    }
}

pub fn part_a(input: &str) -> usize {
    let problem: Problem = input.parse().unwrap();

    problem
        .patrol_slow()
        .visited
        .iter()
        .filter(|&&was_visited| was_visited != 0)
        .count()
}

pub fn part_b(input: &str) -> usize {
    let mut problem: Problem = input.parse().unwrap();

    // Find all squares visited during the original patrol.
    let orig_patrol = problem.patrol_slow();

    // For each square visited on the original patrol, consider placing an obstacle.
    let mut sum = 0;

    for (square_idx, _) in orig_patrol
        .visited
        .iter()
        .enumerate()
        .filter(|(_, &was_visited)| was_visited != 0)
    {
        // WARN: nalgebra's iter() is column-major! So must adapt coord calculation accordingly.
        let pos = util::Coord::from_column_major_index(square_idx, problem.nrows, problem.ncols);

        // Don't block the starting square.
        if pos == problem.guard.pos {
            continue;
        }

        // Block the current square.
        assert!(!problem.step_table.is_obstructed(pos));
        problem.step_table.add_obstruction(pos);
        log::debug!("Obstructed {:?}:\n{:}", pos, problem.step_table);

        sum += problem.patrol_fast() as usize;

        problem.step_table.remove_obstruction(pos);
        log::debug!("Unobstructed {:?}:\n{:}", pos, problem.step_table);
    }

    sum
}

#[cfg(test)]
mod tests {
    #[test]
    fn example_a() {
        util::run_test(|| {
            let expected: usize = 41;
            assert_eq!(
                crate::day_06::part_a(&util::read_resource("example_06.txt").unwrap()),
                expected
            );
        });
    }

    #[test]
    fn example_b() {
        util::run_test(|| {
            let expected: usize = 6;
            assert_eq!(
                crate::day_06::part_b(&util::read_resource("example_06.txt").unwrap()),
                expected
            );
        });
    }
}
