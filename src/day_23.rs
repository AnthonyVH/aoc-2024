use itertools::Itertools;
use rustc_hash::FxHashMap as HashMap;
use rustc_hash::FxHashSet as HashSet;
use std::convert::TryFrom;
use util::BronKerbosh;

struct Problem<'a> {
    names: Vec<&'a str>,
    graph: util::Graph,
}

impl<'a> TryFrom<&'a str> for Problem<'a> {
    type Error = std::string::ParseError;

    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        let mut computers: HashMap<&'a str, util::Vertex> = HashMap::default();
        let mut names: Vec<&'a str> = Vec::new();
        let mut graph = util::Graph::new();

        for line in s.lines() {
            assert_eq!(line.chars().nth(2).unwrap(), '-');
            let lhs = &line[..2];
            let rhs = &line[3..];

            let mut insert_vertex = |name: &&'a str| -> util::Vertex {
                names.push(name);
                (names.len() - 1) as util::Vertex
            };

            let lhs_idx = *computers.entry(lhs).or_insert_with_key(&mut insert_vertex);
            let rhs_idx = *computers.entry(rhs).or_insert_with_key(&mut insert_vertex);

            // Edges are not directed.
            graph.add_neighbours(lhs_idx, &[rhs_idx]);
            graph.add_neighbours(rhs_idx, &[lhs_idx]);
        }

        Ok(Problem { names, graph })
    }
}

pub fn part_a(input: &str) -> u64 {
    let problem: Problem = Problem::try_from(input).unwrap();

    // Need to keep track of all cliques, since it's possible that two big
    // maximal cliques are found with many of the same vertices. In that case
    // generating 3-combinations from them will generate a lot of the same
    // results.
    let mut cliques: HashSet<(util::Vertex, util::Vertex, util::Vertex)> = HashSet::default();

    let process_clique = |vertices: &[util::Vertex]| {
        vertices
            .iter()
            .copied()
            // NOTE: Don't use combinations() since it allocates.
            .tuple_combinations()
            .map(
                |tuple: (util::Vertex, util::Vertex, util::Vertex)| -> [util::Vertex; 3] {
                    tuple.into()
                },
            )
            .filter(|component| {
                // Only accept cliques which contain a computer with a name
                // starting with 't'.
                component
                    .iter()
                    .any(|&idx| problem.names[idx as usize].starts_with('t'))
            })
            .for_each(|mut component| {
                component.sort_unstable();
                cliques.insert(component.iter().copied().collect_tuple().unwrap());
            });
    };

    problem.graph.maximal_cliques(process_clique);
    cliques.len() as u64
}

pub fn part_b(input: &str) -> String {
    let problem: Problem = Problem::try_from(input).unwrap();

    let mut largest_clique = Vec::new();
    let process_clique = |clique: &[util::Vertex]| {
        if largest_clique.len() < clique.len() {
            largest_clique = clique.to_vec();
        }
    };
    problem.graph.maximal_cliques(process_clique);

    let mut named_clique: Vec<&str> = largest_clique
        .iter()
        .map(|idx| problem.names[*idx as usize])
        .collect();
    named_clique.sort_unstable();
    named_clique.iter().join(",").to_string()
}

#[cfg(test)]
mod tests {
    #[test]
    fn example_a() {
        util::run_test(|| {
            let expected: u64 = 7;
            assert_eq!(
                crate::day_23::part_a(&util::read_resource("example_23.txt").unwrap(),),
                expected
            );
        });
    }

    #[test]
    fn example_b() {
        util::run_test(|| {
            let expected = String::from("co,de,ka,ta");
            assert_eq!(
                crate::day_23::part_b(&util::read_resource("example_23.txt").unwrap(),),
                expected
            );
        });
    }
}
