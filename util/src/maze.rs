use nalgebra as na;

#[derive(Debug, Clone)]
pub struct Maze {
    pub maze: na::DMatrix<char>,
    pub start_pos: crate::Coord,
    pub end_pos: crate::Coord,
}

impl Maze {
    pub fn size(&self) -> crate::Coord {
        crate::Coord {
            row: self.maze.nrows() as isize,
            col: self.maze.ncols() as isize,
        }
    }

    pub fn is_wall(&self, pos: &crate::Coord) -> bool {
        self.maze[pos] == '#'
    }

    pub fn accessible(&self, pos: &crate::Coord) -> bool {
        pos.bounded_by(&self.size()) && !self.is_wall(pos)
    }

    pub fn iter(&self) -> impl Iterator<Item = &char> {
        self.maze.iter()
    }
}

impl std::str::FromStr for Maze {
    type Err = std::string::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let rows = s.lines().count();
        let cols = s.lines().next().unwrap().len();

        let mut start_idx: usize = 0;
        let mut end_idx: usize = 0;
        let mut result = Maze {
            maze: na::DMatrix::from_row_iterator(
                rows,
                cols,
                s.lines()
                    .flat_map(|line| line.chars())
                    .enumerate()
                    .inspect(|(idx, e)| match e {
                        'S' => start_idx = *idx,
                        'E' => end_idx = *idx,
                        _ => (),
                    })
                    .map(|(_, e)| e),
            ),
            start_pos: crate::Coord { row: 0, col: 0 },
            end_pos: crate::Coord { row: 0, col: 0 },
        };

        result.start_pos = crate::Coord::from_row_major_index(start_idx, rows, cols);
        result.end_pos = crate::Coord::from_row_major_index(end_idx, rows, cols);

        Ok(result)
    }
}
