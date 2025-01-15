extern crate nalgebra as na;

#[derive(Clone, Copy, Debug, strum_macros::EnumIter)]
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

#[derive(Copy, Clone, Debug)]
pub struct Coord {
    pub row: isize,
    pub col: isize,
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
    type Output = Self;

    fn mul(self, other: isize) -> Coord {
        other * self
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

// Need to implement traits for Get here, since it's not allowed to implement non-crate
// traits for non-crate types.
impl<T> std::ops::Index<crate::Coord> for na::DMatrix<T> {
    type Output = T;

    fn index(&self, index: crate::Coord) -> &Self::Output {
        assert!(index.row >= 0);
        assert!(index.col >= 0);
        &self[(index.row as usize, index.col as usize)]
    }
}

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
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
