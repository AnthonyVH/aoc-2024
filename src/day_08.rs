#[derive(Debug)]
struct Problem {
    antennas: std::collections::HashMap<char, Vec<util::Coord>>,
    city_bounds: util::Coord,
}

impl std::str::FromStr for Problem {
    type Err = std::string::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut result = Problem {
            antennas: std::collections::HashMap::new(),
            city_bounds: (s.lines().count(), s.lines().next().unwrap().len()).into(),
        };

        for (row, line) in s.lines().enumerate() {
            for (col, value) in line.chars().enumerate() {
                match value {
                    'A'..='Z' | 'a'..='z' | '0'..='9' => result
                        .antennas
                        .entry(value)
                        .or_insert(Vec::new())
                        .push((row, col).into()),
                    _ => (),
                }
            }
        }

        Ok(result)
    }
}

pub fn part_a(input: &str) -> usize {
    let problem: Problem = input.parse().unwrap();

    let mut antinodes = std::collections::HashSet::<util::Coord>::new();

    for (_, coords) in problem.antennas.iter() {
        use itertools::Itertools;
        for (coord_a, coord_b) in coords.iter().tuple_combinations() {
            let offset = coord_b - coord_a;
            for coord in [coord_a - &offset, coord_b + &offset] {
                if !coord.has_negatives() && coord.bounded_by(&problem.city_bounds) {
                    antinodes.insert(coord);
                }
            }
        }
    }

    antinodes.len()
}

pub fn part_b(input: &str) -> usize {
    let problem: Problem = input.parse().unwrap();

    // NOTE: Parallellizing this makes it slower.
    problem
        .antennas
        .iter()
        .flat_map(|(_, coords)| {
            use itertools::Itertools;
            coords.iter().tuple_combinations::<(_, _)>()
        })
        .map(|coord_pair| -> [&util::Coord; 2] { coord_pair.into() })
        .flat_map(|coord_pair| {
            let (&&coord_min, &&coord_max) = itertools::Itertools::minmax(coord_pair.iter())
                .into_option()
                .unwrap();
            let offset = coord_max - coord_min;

            let forward_iter = (0isize..)
                .map(move |step| coord_max + step * offset)
                .take_while(|coord| {
                    !coord.has_negatives() && coord.bounded_by(&problem.city_bounds)
                });

            let backward_iter = (0isize..)
                .map(move |step| coord_min - step * offset)
                .take_while(|coord| {
                    !coord.has_negatives() && coord.bounded_by(&problem.city_bounds)
                });

            forward_iter.chain(backward_iter)
        })
        .collect::<std::collections::HashSet<util::Coord>>()
        .len()
}

#[cfg(test)]
mod tests {
    #[test]
    fn example_a() {
        util::run_test(|| {
            let expected: usize = 14;
            assert_eq!(
                crate::day_08::part_a(&util::read_resource("example_08.txt").unwrap()),
                expected
            );
        });
    }

    #[test]
    fn example_b() {
        util::run_test(|| {
            let expected: usize = 34;
            assert_eq!(
                crate::day_08::part_b(&util::read_resource("example_08.txt").unwrap()),
                expected
            );
        });
    }
}
