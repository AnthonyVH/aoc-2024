extern crate nalgebra as na;

#[derive(Clone, Copy, Debug, PartialEq, strum_macros::EnumIter)]
pub enum Direction {
    East,
    West,
    North,
    South,
    NorthEast,
    NorthWest,
    SouthEast,
    SouthWest,
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct Coord {
    pub row: isize,
    pub col: isize,
}

impl Coord {
    pub fn as_pair(&self) -> (usize, usize) {
        self.into()
    }

    pub fn has_negatives(&self) -> bool {
        (self.row < 0) || (self.col < 0)
    }

    pub fn bounded_by(&self, bound: &Coord) -> bool {
        (self.row < bound.row) && (self.col < bound.col)
    }

    pub fn from_row_major_index(idx: usize, nrows: usize, ncols: usize) -> Coord {
        Coord::from((idx / ncols, idx % nrows))
    }

    pub fn from_column_major_index(idx: usize, nrows: usize, ncols: usize) -> Coord {
        Coord::from((idx % nrows, idx / ncols))
    }
}

impl std::ops::Add for Coord {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            row: self.row + other.row,
            col: self.col + other.col,
        }
    }
}

impl std::ops::Add for &Coord {
    type Output = Coord;

    fn add(self, other: Self) -> Coord {
        Coord {
            row: self.row + other.row,
            col: self.col + other.col,
        }
    }
}

impl std::ops::AddAssign for Coord {
    fn add_assign(&mut self, rhs: Coord) {
        self.row += rhs.row;
        self.col += rhs.col;
    }
}

impl std::ops::Sub for Coord {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            row: self.row - other.row,
            col: self.col - other.col,
        }
    }
}

impl std::ops::Sub for &Coord {
    type Output = Coord;

    fn sub(self, other: Self) -> Coord {
        Coord {
            row: self.row - other.row,
            col: self.col - other.col,
        }
    }
}

impl std::ops::SubAssign for Coord {
    fn sub_assign(&mut self, rhs: Coord) {
        self.row -= rhs.row;
        self.col -= rhs.col;
    }
}

impl std::ops::Mul<Coord> for isize {
    type Output = Coord;

    fn mul(self, other: Coord) -> Coord {
        Coord {
            row: self * other.row,
            col: self * other.col,
        }
    }
}

impl std::ops::Mul<isize> for Coord {
    type Output = Coord;

    fn mul(self, other: isize) -> Coord {
        other * self
    }
}

impl std::ops::Mul<u8> for Coord {
    type Output = Coord;

    fn mul(self, other: u8) -> Coord {
        other as isize * self
    }
}

impl std::ops::Mul<Coord> for u8 {
    type Output = Coord;

    fn mul(self, other: Coord) -> Coord {
        self as isize * other
    }
}

impl From<Direction> for Coord {
    fn from(item: Direction) -> Coord {
        match item {
            Direction::East => Coord { row: 0, col: 1 },
            Direction::West => Coord { row: 0, col: -1 },
            Direction::North => Coord { row: -1, col: 0 },
            Direction::South => Coord { row: 1, col: 0 },
            Direction::NorthEast => Coord::from(Direction::North) + Coord::from(Direction::East),
            Direction::NorthWest => Coord::from(Direction::North) + Coord::from(Direction::West),
            Direction::SouthEast => Coord::from(Direction::South) + Coord::from(Direction::East),
            Direction::SouthWest => Coord::from(Direction::South) + Coord::from(Direction::West),
        }
    }
}

impl From<(usize, usize)> for Coord {
    fn from((row, col): (usize, usize)) -> Coord {
        Coord {
            row: row.try_into().unwrap(),
            col: col.try_into().unwrap(),
        }
    }
}

impl From<Coord> for (usize, usize) {
    fn from(coord: Coord) -> (usize, usize) {
        (&coord).into()
    }
}

impl From<&Coord> for (usize, usize) {
    fn from(coord: &Coord) -> (usize, usize) {
        // This assert causes a signification slowdown and is only used to catch
        // e.g. one-off errors during debugging anyway.
        #[cfg(test)]
        assert!(!coord.has_negatives());
        (coord.row as usize, coord.col as usize)
    }
}

// Need to implement traits for Get here, since it's not allowed to implement non-crate
// traits for non-crate types.
impl<T> std::ops::Index<Coord> for na::DMatrix<T> {
    type Output = T;

    fn index(&self, index: Coord) -> &Self::Output {
        &self[Into::<(usize, usize)>::into(index)]
    }
}

impl<T> std::ops::IndexMut<Coord> for na::DMatrix<T> {
    fn index_mut(&mut self, index: Coord) -> &mut Self::Output {
        &mut self[Into::<(usize, usize)>::into(index)]
    }
}

impl<'a, T: 'a, R, C, S> na::base::indexing::MatrixIndex<'a, T, R, C, S> for Coord
where
    R: na::Dim,
    C: na::Dim,
    S: na::RawStorage<T, R, C>,
{
    type Output = &'a T;

    fn contained_by(&self, matrix: &na::Matrix<T, R, C, S>) -> bool {
        let pair = Into::<(usize, usize)>::into(self);
        pair.contained_by(matrix)
    }

    unsafe fn get_unchecked(self, matrix: &'a na::Matrix<T, R, C, S>) -> Self::Output {
        let pair = Into::<(usize, usize)>::into(self);
        pair.get_unchecked(matrix)
    }
}

impl<'a, T: 'a, R, C, S> na::base::indexing::MatrixIndexMut<'a, T, R, C, S> for Coord
where
    R: na::Dim,
    C: na::Dim,
    S: na::RawStorageMut<T, R, C>,
{
    type OutputMut = &'a mut T;

    unsafe fn get_unchecked_mut(
        self,
        matrix: &'a mut nalgebra::Matrix<T, R, C, S>,
    ) -> Self::OutputMut {
        let pair = Into::<(usize, usize)>::into(self);
        pair.get_unchecked_mut(matrix)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct DirectedCoordRange {
    pub start: Coord,
    pub len: usize,
    pub dir: Direction,
}

impl DirectedCoordRange {
    pub fn iter(&self) -> DirectedCoordRangeIterator {
        DirectedCoordRangeIterator {
            range: self.clone(),
            offset: 0,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct DirectedCoordRangeIterator {
    // Store the range inside the iterator, such that the original range doesn't need to
    // remain alive. This allows storing this struct as a member of another struct,
    // without having to store it's range separately. Which wouldn't be possible, since
    // then that struct couldn't be moved.
    range: DirectedCoordRange,
    offset: usize,
}

impl Iterator for DirectedCoordRangeIterator {
    type Item = Coord;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset < self.range.len {
            let dir_offset: Coord = self.range.dir.clone().into();
            let result =
                Some(self.range.start + isize::try_from(self.offset.clone()).unwrap() * dir_offset);
            self.offset += 1;
            result
        } else {
            None
        }
    }

    fn last(self) -> Option<Self::Item> {
        let dir_offset: Coord = self.range.dir.into();
        let max_steps = isize::try_from(self.range.len).unwrap() - 1;
        Some(self.range.start + max_steps * dir_offset)
    }
}

impl ExactSizeIterator for DirectedCoordRangeIterator {
    fn len(&self) -> usize {
        self.range.len - self.offset
    }
}
