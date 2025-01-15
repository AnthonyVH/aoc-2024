use std::collections::VecDeque;

extern crate nalgebra as na;

struct TopographicMap {
    nrows: usize,
    ncols: usize,
    heights: na::DMatrix<u8>,
}

impl TopographicMap {
    const HEIGHT_PATH_START: u8 = 0;
    const HEIGHT_PATH_END: u8 = 9;

    const SEARCH_DIRS: [util::Direction; 4] = [
        util::Direction::North,
        util::Direction::East,
        util::Direction::South,
        util::Direction::West,
    ];

    fn _find_trails(&self, start_pos: util::Coord, allow_revisits: bool) -> usize {
        // Only start trail searches from height 0.
        let start_height = self.heights[start_pos.as_pair()];
        if start_height != Self::HEIGHT_PATH_START {
            return 0;
        }

        // Do DFS to find the number of trails that end at height 9. A DFS
        // instead of BFS is used for improved cache locality. Might be able to
        // speed this up even more if "cache" is reused.
        let mut score = 0;
        let mut to_visit: VecDeque<util::Coord> = VecDeque::with_capacity(self.heights.len());
        let mut marked = na::DMatrix::from_element(self.nrows, self.ncols, false);

        // Prime queue.
        to_visit.push_back(start_pos);

        while let Some(pos) = to_visit.pop_back() {
            // Don't visit marked squares multiple times.
            if marked[pos.as_pair()] {
                continue;
            }
            marked[pos.as_pair()] = !allow_revisits;

            let cur_height = *self.heights.get(pos.as_pair()).unwrap();

            if cur_height == Self::HEIGHT_PATH_END {
                // If we're at the end of a path, increase count.
                score += 1;
            } else {
                // Otherwise keep searching by visiting all children that have a
                // height one larger than current position.
                for &offset_dir in Self::SEARCH_DIRS.iter() {
                    let coord: util::Coord = pos + offset_dir;

                    if coord.has_negatives() {
                        continue; // Don't bother with invalid coordinates.
                    } else if let Some(&height) = self.heights.get(coord.as_pair()) {
                        if height == (cur_height + 1) {
                            to_visit.push_back(coord);
                        }
                    }
                }
            }
        }

        score
    }

    fn _sum_trails(&self, allow_revisits: bool) -> usize {
        (0..self.heights.len())
            .map(|pos_idx| {
                // na::DMatrix stores data in column major order, so iterating this
                // way results in faster code.
                let start_pos =
                    util::Coord::from_column_major_index(pos_idx, self.nrows, self.ncols);
                self._find_trails(start_pos, allow_revisits)
            })
            .sum()
    }
}

impl std::str::FromStr for TopographicMap {
    type Err = std::string::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let rows = s.lines().count();
        let cols = s.lines().next().unwrap().len();
        Ok(Self {
            nrows: rows,
            ncols: cols,
            heights: na::DMatrix::from_row_iterator(
                rows,
                cols,
                s.lines()
                    .flat_map(|e| e.as_bytes().iter().map(|e| *e - b'0')),
            ),
        })
    }
}

pub fn part_a(input: &str) -> usize {
    let map: TopographicMap = input.parse().unwrap();
    // Count the number of distinct trail ends reached by not allowing
    // revisiting the same cells. That will prevent following multiple trails
    // reusing the same subsections of a trail.
    map._sum_trails(false)
}

pub fn part_b(input: &str) -> usize {
    let map: TopographicMap = input.parse().unwrap();
    // Allow revisiting cells, suchs that multiple trails can reuse the same
    // trail subsections.
    map._sum_trails(true)
}

#[cfg(test)]
mod tests {
    #[test]
    fn example_a() {
        util::run_test(|| {
            let expected: usize = 36;
            assert_eq!(
                crate::day_10::part_a(&util::read_resource("example_10.txt").unwrap()),
                expected
            );
        });
    }

    #[test]
    fn example_b() {
        util::run_test(|| {
            let expected: usize = 81;
            assert_eq!(
                crate::day_10::part_b(&util::read_resource("example_10.txt").unwrap()),
                expected
            );
        });
    }
}
