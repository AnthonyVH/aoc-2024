#[derive(Debug)]
struct ClawMachine {
    button_moves: [util::Coord; 2],
    prize: util::Coord,
}

impl ClawMachine {
    fn num_button_presses_to_win(&self) -> Option<[usize; 2]> {
        let divisor = (self.button_moves[1].col * self.button_moves[0].row
            - self.button_moves[0].col * self.button_moves[1].row) as f64;
        let num_presses: [f64; 2] = [
            (self.button_moves[1].col * self.prize.row - self.prize.col * self.button_moves[1].row),
            (self.prize.col * self.button_moves[0].row - self.button_moves[0].col * self.prize.row),
        ]
        .map(|e| (e as f64) / divisor);
        log::debug!("# presses for {:?}: {:?}", self, num_presses);

        let are_presses_integer = num_presses.iter().all(|e| e.fract() == 0.);
        match are_presses_integer {
            false => None,
            true => Some(num_presses.map(|e| e as usize)),
        }
    }

    fn num_tokens_to_win(&self) -> Option<usize> {
        match self.num_button_presses_to_win() {
            None => None,
            Some([press_a, press_b]) => Some(3 * press_a + 1 * press_b),
        }
    }
}

impl std::str::FromStr for ClawMachine {
    type Err = std::string::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parse_coord = |s: &str| -> util::Coord {
            let start_x = s.find('X').unwrap() + 2;
            let end_x = start_x + s[start_x..].find(',').unwrap();
            let start_y = end_x + s[end_x..].find('Y').unwrap() + 2;
            util::Coord {
                row: s[start_y..].parse().unwrap(),
                col: s[start_x..end_x].parse().unwrap(),
            }
        };

        let mut lines = s.lines();

        Ok(ClawMachine {
            button_moves: [
                parse_coord(lines.next().unwrap()),
                parse_coord(lines.next().unwrap()),
            ],
            prize: parse_coord(lines.next().unwrap()),
        })
    }
}

pub fn part_a(input: &str) -> usize {
    input
        .split("\n\n")
        .map(|sub| sub.parse::<ClawMachine>().unwrap())
        .filter_map(|e| e.num_tokens_to_win())
        .sum()
}

pub fn part_b(input: &str) -> usize {
    input
        .split("\n\n")
        .map(|sub| sub.parse::<ClawMachine>().unwrap())
        .map(|mut machine| {
            const OFFSET: isize = 10000000000000;
            machine.prize.row += OFFSET;
            machine.prize.col += OFFSET;
            machine
        })
        .filter_map(|e| e.num_tokens_to_win())
        .sum()
}

#[cfg(test)]
mod tests {
    #[test]
    fn example_a() {
        util::run_test(|| {
            let expected: usize = 480;
            assert_eq!(
                crate::day_13::part_a(&util::read_resource("example_13.txt").unwrap()),
                expected
            );
        });
    }

    // No example for part B.
}
