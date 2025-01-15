extern crate nalgebra as na;

pub trait Get<T> {
    type Item;

    fn get<'a>(&'a self, idx: &T) -> Option<&'a Self::Item>;
}

// Need to implement traits for Get here, since it's not allowed to implement non-crate
// traits for non-crate types.
impl<T> Get<crate::Coord> for na::DMatrix<T> {
    type Item = T;

    fn get<'a>(&'a self, idx: &crate::Coord) -> Option<&'a Self::Item> {
        self.get((idx.row as usize, idx.col as usize))
    }
}
