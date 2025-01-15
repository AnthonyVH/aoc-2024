extern crate nalgebra as na;

// Create a wrapper for 3rd party type, so we can implement our own traits, which live
// in the utils crate, for it.
// TODO: Why must this be clonable in order to be able to clone the DirectedSliceIterator?
// That iterator only stores a reference, so why is cloning of the underlying struct required?
#[derive(Clone, Debug)]
struct MatrixWrapper(na::DMatrix<u8>);

impl util::Get<util::Coord> for MatrixWrapper {
    type Item = u8;

    fn get<'a>(&'a self, idx: &util::Coord) -> Option<&'a Self::Item> {
        util::Get::get(&self.0, idx)
    }
}

impl std::ops::Index<util::Coord> for MatrixWrapper {
    type Output = u8;

    fn index(&self, index: util::Coord) -> &Self::Output {
        &self.0[index]
    }
}

impl util::DirectedSlice for MatrixWrapper {}

#[derive(Debug)]
struct WordSearch {
    data: MatrixWrapper,
}

impl WordSearch {
    fn ncols(&self) -> usize {
        self.data.0.ncols()
    }

    fn nrows(&self) -> usize {
        self.data.0.nrows()
    }

    fn slice(
        &self,
        coord_range: util::DirectedCoordRange,
    ) -> util::DirectedSliceIterator<MatrixWrapper> {
        util::DirectedSlice::slice(&self.data, coord_range)
    }
}

impl std::str::FromStr for WordSearch {
    type Err = std::string::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let rows = s.lines().count();
        let cols = s.lines().next().unwrap().len();
        Ok(Self {
            data: MatrixWrapper(na::DMatrix::from_row_iterator(
                rows,
                cols,
                s.lines().flat_map(|e| e.as_bytes().iter().map(|e: &u8| *e)),
            )),
        })
    }
}

pub fn part_a(input: &str) -> usize {
    let word_search: WordSearch = input.parse().unwrap();

    // For every position in the matrix, check all directions for needle.
    const NEEDLE: &'static str = "XMAS";
    const SEARCH_DIRECTIONS: &'static [util::Direction] = &[
        util::Direction::NorthEast,
        util::Direction::East,
        util::Direction::SouthEast,
        util::Direction::South,
    ];

    // TODO: Check if precalculating coordinate offsets speeds things up.
    itertools::Itertools::cartesian_product(0..word_search.nrows(), 0..word_search.ncols())
        .map(|e| e.into())
        .map(|coord| {
            SEARCH_DIRECTIONS
                .iter()
                .map(|&dir| {
                    let range = util::DirectedCoordRange {
                        start: coord,
                        len: NEEDLE.len(),
                        dir: dir,
                    };
                    let slice = word_search.slice(range);
                    slice.clone().eq(NEEDLE.as_bytes()) as usize
                        + slice.eq(NEEDLE.as_bytes().iter().rev()) as usize
                })
                .sum::<usize>()
        })
        .sum()
}

pub fn part_b(input: &str) -> usize {
    let word_search: WordSearch = input.parse().unwrap();

    // A cross of needle "MAS" always requires the A to be in the center. So we can
    // check that first
    const NEEDLE_LEN: usize = 3;
    const NEEDLE_MID: u8 = b'A';
    const NEEDLE_OUTER: [u8; 2] = [b'M', b'S'];

    const OFFSET_OUTERS: [[util::Coord; 2]; 2] = [
        [
            // South-east diagonal
            util::Coord { row: 0, col: 0 },
            util::Coord { row: 2, col: 2 },
        ],
        [
            // North-east diagonal
            util::Coord { row: 2, col: 0 },
            util::Coord { row: 0, col: 2 },
        ],
    ];
    const OFFSET_MID: util::Coord = util::Coord { row: 1, col: 1 };

    itertools::Itertools::cartesian_product(
        0..(word_search.nrows() - (NEEDLE_LEN - 1)),
        0..(word_search.ncols() - (NEEDLE_LEN - 1)),
    )
    .map(|e| e.into())
    .filter(|start_coord: &util::Coord| {
        // Check that all lists of offsets point to elements equal to needle.
        OFFSET_OUTERS.iter().all(|offset_coords| {
            let get_elem = |&offset| util::Get::get(&word_search.data.0, &(*start_coord + offset));

            if get_elem(&OFFSET_MID).is_some_and(|e| *e != NEEDLE_MID) {
                // NOTE: This rechecked for each offset list, which is a bit redundant.
                false
            } else {
                let values = offset_coords.iter().map(|e| get_elem(e));

                if values.clone().any(|e| e.is_none()) {
                    // All coordinates must fall within the matrix.
                    false
                } else {
                    // Values must match either forward or backward.
                    std::iter::zip(values.clone(), NEEDLE_OUTER.iter())
                        .all(|(value, needle)| value.unwrap() == needle)
                        || std::iter::zip(values, NEEDLE_OUTER.iter().rev())
                            .all(|(value, needle)| value.unwrap() == needle)
                }
            }
        })
    })
    .count()
}

#[cfg(test)]
mod tests {
    #[test]
    fn example_a() {
        let expected: usize = 18;
        assert_eq!(
            crate::day_04::part_a(&util::read_resource("example_04.txt").unwrap()),
            expected
        );
    }

    #[test]
    fn example_b() {
        let expected: usize = 9;
        assert_eq!(
            crate::day_04::part_b(&util::read_resource("example_04.txt").unwrap()),
            expected
        );
    }
}
