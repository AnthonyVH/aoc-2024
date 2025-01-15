use std::{cell::RefCell, collections::VecDeque};

extern crate nalgebra as na;

struct MatrixDfsSearcher {
    marked: na::DMatrix<bool>,
    to_visit: VecDeque<util::Coord>,
}

impl MatrixDfsSearcher {
    // TODO: Make this configurable.
    const SEARCH_DIRS: [util::Direction; 4] = [
        util::Direction::North,
        util::Direction::East,
        util::Direction::South,
        util::Direction::West,
    ];

    fn new(nrows: usize, ncols: usize) -> MatrixDfsSearcher {
        MatrixDfsSearcher {
            marked: na::DMatrix::from_element(nrows, ncols, false),
            to_visit: VecDeque::default(),
        }
    }

    fn dfs<FnFirstVisit, FnNoNeighbor, FnNeighbor, FnIsNeighborOk>(
        &mut self,
        start_pos: util::Coord,
        mut first_visit_fn: FnFirstVisit,
        mut no_neighbor_fn: FnNoNeighbor,
        mut neighbor_fn: FnNeighbor,
        is_neighbor_ok_fn: FnIsNeighborOk,
    ) where
        FnFirstVisit: FnMut(util::Coord),
        FnNoNeighbor: FnMut(util::Coord, util::Direction),
        FnNeighbor: FnMut(util::Coord, util::Coord, util::Direction),
        FnIsNeighborOk: Fn(util::Coord, util::Coord) -> bool,
    {
        // Don't bother with plots we've already seen.
        if self.marked[start_pos] {
            return;
        }

        assert!(self.to_visit.len() == 0);
        self.to_visit.push_back(start_pos);

        self.marked[start_pos] = true;

        while let Some(visit_pos) = self.to_visit.pop_front() {
            // Notify caller of first visit to this coordinate.
            (first_visit_fn)(visit_pos);

            // Try to visit all neighbors.
            for &offset_dir in Self::SEARCH_DIRS.iter() {
                let neighbor_pos: util::Coord = visit_pos + offset_dir.into();

                if neighbor_pos.has_negatives() {
                    (no_neighbor_fn)(visit_pos, offset_dir);
                } else {
                    match self.marked.get_mut(neighbor_pos) {
                        None => (no_neighbor_fn)(visit_pos, offset_dir),
                        Some(marked) => {
                            (neighbor_fn)(visit_pos, neighbor_pos, offset_dir);
                            if !*marked && (is_neighbor_ok_fn)(visit_pos, neighbor_pos) {
                                *marked = true;
                                self.to_visit.push_back(neighbor_pos);
                            }
                        }
                    }
                }
            }
        }
    }
}

trait DirectionProperties {
    fn from_index(idx: usize) -> util::Direction;
    fn to_index(self) -> usize;
    fn is_horizontal_edge(self) -> bool;
}

impl DirectionProperties for util::Direction {
    fn from_index(idx: usize) -> util::Direction {
        match idx {
            0 => util::Direction::North,
            1 => util::Direction::East,
            2 => util::Direction::South,
            3 => util::Direction::West,
            _ => unreachable!(),
        }
    }

    fn to_index(self) -> usize {
        match self {
            util::Direction::North => 0,
            util::Direction::East => 1,
            util::Direction::South => 2,
            util::Direction::West => 3,
            _ => unreachable!(),
        }
    }

    fn is_horizontal_edge(self) -> bool {
        match self {
            util::Direction::North | util::Direction::South => true,
            util::Direction::East | util::Direction::West => false,
            _ => unreachable!(),
        }
    }
}

fn parse_input(input: &str) -> na::DMatrix<u8> {
    na::DMatrix::from_row_iterator(
        input.lines().count(),
        input.lines().next().unwrap().len(),
        input.lines().flat_map(|e| e.as_bytes().iter().copied()),
    )
}

#[derive(Debug)]
struct PlotProperties {
    area: usize,
    perimeter: usize,
    perimeter_coords: [Vec<util::Coord>; MatrixDfsSearcher::SEARCH_DIRS.len()],
}

impl PlotProperties {
    fn new() -> PlotProperties {
        PlotProperties {
            area: 0,
            perimeter: 0,
            perimeter_coords: [(); MatrixDfsSearcher::SEARCH_DIRS.len()]
                .map(|_| Vec::<util::Coord>::default()),
        }
    }

    fn reset(&mut self) {
        self.area = 0;
        self.perimeter = 0;
        self.perimeter_coords.iter_mut().for_each(|e| e.clear());
    }
}

pub fn part_a(input: &str) -> usize {
    let plots = parse_input(input);
    let mut result = 0;

    // Reuse storage for a minor speed-up.
    let properties = RefCell::new(PlotProperties::new());
    let mut searcher = MatrixDfsSearcher::new(plots.nrows(), plots.ncols());

    // Go over each plot and gather neighboring plots of the same type.
    for (plot_idx, plot_type) in plots.iter().enumerate() {
        let start_pos =
            util::Coord::from_column_major_index(plot_idx, plots.nrows(), plots.ncols());

        // Increment a region's perimeter if for a given side a plot does either
        // not have a neighbor, or its neighbor has a different type.
        searcher.dfs(
            start_pos,
            |_| properties.borrow_mut().area += 1,
            |_, _| properties.borrow_mut().perimeter += 1,
            |lhs, rhs, _| {
                if plots[lhs] != plots[rhs] {
                    properties.borrow_mut().perimeter += 1;
                }
            },
            |lhs, rhs| plots[lhs] == plots[rhs],
        );

        // Update result.
        if properties.borrow().area != 0 {
            let cost = properties.borrow().area * properties.borrow().perimeter;
            log::debug!(
                "Region {} (start @ {:?}): {} * {} => {}",
                *plot_type as char,
                start_pos,
                properties.borrow().area,
                properties.borrow().perimeter,
                cost
            );
            result += cost;

            // Reset properties for next iteration.
            properties.borrow_mut().reset();
        }
    }

    result
}

fn update_plot_properties(
    plots: &na::DMatrix<u8>,
    searcher: &mut MatrixDfsSearcher,
    properties: &RefCell<PlotProperties>,
    start_pos: util::Coord,
) {
    // For each of the edges of a plot that is on the perimeter, store that
    // plot's coordinates in a list.
    searcher.dfs(
        start_pos,
        |_| properties.borrow_mut().area += 1,
        |coord, dir| properties.borrow_mut().perimeter_coords[dir.to_index()].push(coord),
        |lhs, rhs, dir| {
            if plots[lhs] != plots[rhs] {
                properties.borrow_mut().perimeter_coords[dir.to_index()].push(lhs);
            }
        },
        |lhs, rhs| plots[lhs] == plots[rhs],
    );
}

fn count_num_edges(coords: &mut [util::Coord], edge_position: util::Direction) -> usize {
    // If edge is vertical, sort by column first, otherwise sort by row first.
    // This will ensure all coordinates are sorted such that we can check whether
    // consecutive coordinates have a gap between them or not.
    let sort_fn = match edge_position.is_horizontal_edge() {
        true => |e: &util::Coord| (e.row, e.col),
        false => |e: &util::Coord| (e.col, e.row),
    };
    coords.sort_by_key(sort_fn);

    // All the coordinates are now sorted, but we need to group them either by
    // row or column.
    let chunk_fn = match edge_position.is_horizontal_edge() {
        true => |lhs: &util::Coord, rhs: &util::Coord| lhs.row == rhs.row,
        false => |lhs: &util::Coord, rhs: &util::Coord| lhs.col == rhs.col,
    };

    // To detect if a pair of coordinates has a gap between them, we need to
    // select the correct property. I.e. for an edge in the horizontal direction
    // there's a gap when the difference between the rhs' and lhs' column is not
    // exactly one.
    let gap_fn = match edge_position.is_horizontal_edge() {
        true => |e: &util::Coord| e.col,
        false => |e: &util::Coord| e.row,
    };

    // A pair of coordinates are part of the same edge if the difference
    // between their relevant coordinate property (i.e. row or col) is
    // exactly 1. If the difference is not 1, then there's a gap and
    // they belong to different edges. Furthermore, to be on the same
    // edge, they must also have the same col, respectively row.
    1 + coords
        .windows(2)
        .filter(|window| {
            // Keep all the pairs that represent a gap to a next edge.
            match chunk_fn(&window[0], &window[1]) {
                false => true,
                true => gap_fn(&window[1]) - gap_fn(&window[0]) != 1,
            }
        })
        .count()
}

pub fn part_b(input: &str) -> usize {
    let plots = parse_input(input);
    let mut result = 0;

    // Reuse storage for a minor speed-up.
    let properties = RefCell::new(PlotProperties::new());
    let mut searcher = MatrixDfsSearcher::new(plots.nrows(), plots.ncols());

    // Go over each plot and gather neighboring plots of the same type.
    for (plot_idx, plot_type) in plots.iter().enumerate() {
        let start_pos =
            util::Coord::from_column_major_index(plot_idx, plots.nrows(), plots.ncols());

        // Get properties for current area.
        update_plot_properties(&plots, &mut searcher, &properties, start_pos);

        // If area was already visited, skip the remainder.
        if properties.borrow().area == 0 {
            continue;
        }

        // All of the connected plot's perimeter coordinates are now in the
        // lists. For each edge orientation, calculate the number of edges.
        let num_edges: usize = properties
            .borrow_mut()
            .perimeter_coords
            .iter_mut()
            .enumerate()
            .map(|(idx, coords)| {
                count_num_edges(
                    coords,
                    <util::Direction as DirectionProperties>::from_index(idx),
                )
            })
            .sum();

        if num_edges > 0 {
            let cost = properties.borrow().area * num_edges;
            log::debug!(
                "Region {} (start @ {:?}): {} * {} => {}",
                *plot_type as char,
                start_pos,
                properties.borrow().area,
                num_edges,
                cost
            );
            result += cost;

            // Reset properties for next iteration.
            properties.borrow_mut().reset();
        }
    }

    result
}

#[cfg(test)]
mod tests {
    #[test]
    fn example_a_1() {
        util::run_test(|| {
            let expected: usize = 140;
            assert_eq!(
                crate::day_12::part_a(&util::read_resource("example_12-part_1.txt").unwrap()),
                expected
            );
        });
    }

    #[test]
    fn example_a_2() {
        util::run_test(|| {
            let expected: usize = 772;
            assert_eq!(
                crate::day_12::part_a(&util::read_resource("example_12-part_2.txt").unwrap()),
                expected
            );
        });
    }

    #[test]
    fn example_a_3() {
        util::run_test(|| {
            let expected: usize = 1930;
            assert_eq!(
                crate::day_12::part_a(&util::read_resource("example_12-part_3.txt").unwrap()),
                expected
            );
        });
    }

    #[test]
    fn example_b_1() {
        util::run_test(|| {
            let expected: usize = 80;
            assert_eq!(
                crate::day_12::part_b(&util::read_resource("example_12-part_1.txt").unwrap()),
                expected
            );
        });
    }

    #[test]
    fn example_b_2() {
        util::run_test(|| {
            let expected: usize = 436;
            assert_eq!(
                crate::day_12::part_b(&util::read_resource("example_12-part_2.txt").unwrap()),
                expected
            );
        });
    }

    #[test]
    fn example_b_3() {
        util::run_test(|| {
            let expected: usize = 1206;
            assert_eq!(
                crate::day_12::part_b(&util::read_resource("example_12-part_3.txt").unwrap()),
                expected
            );
        });
    }

    #[test]
    fn example_b_4() {
        util::run_test(|| {
            let expected: usize = 236;
            assert_eq!(
                crate::day_12::part_b(&util::read_resource("example_12-part_4.txt").unwrap()),
                expected
            );
        });
    }

    #[test]
    fn example_b_5() {
        util::run_test(|| {
            let expected: usize = 368;
            assert_eq!(
                crate::day_12::part_b(&util::read_resource("example_12-part_5.txt").unwrap()),
                expected
            );
        });
    }
}
