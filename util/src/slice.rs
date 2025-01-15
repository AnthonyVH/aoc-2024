pub trait DirectedSlice
where
    Self: crate::Get<crate::Coord>,
{
    fn slice<'a>(
        &'a self,
        coord_range: crate::DirectedCoordRange,
    ) -> DirectedSliceIterator<'a, Self> {
        DirectedSliceIterator::new(self, coord_range)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct DirectedSliceIterator<'a, T>
where
    T: ?Sized,
{
    data: &'a T,
    iter: crate::DirectedCoordRangeIterator,
}

impl<'a, T> DirectedSliceIterator<'a, T>
where
    T: ?Sized,
{
    // Private, so only DirectedSlice can construct this.
    fn new(data: &'a T, mut coord_range: crate::DirectedCoordRange) -> DirectedSliceIterator<'a, T>
    where
        T: crate::Get<crate::Coord>,
    {
        // Ensure that range is either empty, or that all access will succeed.

        // If range is empty, then no bounds checking is required.
        if coord_range.len > 0 {
            // Ensure that whole range is accessible on data.
            let first = &coord_range.start;
            let last = coord_range.iter().last().unwrap();

            // Assume most iterators are created at a valid starting point. I.e. fail as
            // fast as possible by checking validity of last coordinate first.
            if data.get(&last).is_none() || data.get(first).is_none() {
                // Change the range to a null range.
                coord_range.len = 0;
            }
        }

        DirectedSliceIterator {
            data: data,
            iter: coord_range.iter(), // Possibly changed length, so must recreate iterator.
        }
    }
}

impl<'a, T> Iterator for DirectedSliceIterator<'a, T>
where
    T: std::ops::Index<crate::Coord>,
{
    type Item = &'a <T as std::ops::Index<crate::Coord>>::Output;

    fn next(&mut self) -> Option<Self::Item> {
        // Since DirectedSlice implementation ensures that all coordinates are in range,
        // there's no need to check here. I.e. the Index operator should never panic.
        self.iter.next().and_then(|e| Some(&self.data[e]))
    }
}
