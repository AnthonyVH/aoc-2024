use rustc_hash::FxHashMap as HashMap;
use rustc_hash::FxHashSet as HashSet;

pub type Vertex = u32;

#[derive(Debug, Clone)]
pub struct Graph {
    pub neighbours: HashMap<Vertex, HashSet<Vertex>>,
}

impl Graph {
    pub fn new() -> Graph {
        Graph {
            neighbours: HashMap::default(),
        }
    }

    pub fn add_vertex(&mut self, index: Vertex) -> &mut HashSet<Vertex> {
        self.neighbours.entry(index).or_insert(HashSet::default())
    }

    pub fn add_neighbours(&mut self, index: Vertex, neighbours: &[Vertex]) {
        let existing_neighbours = self.add_vertex(index);
        neighbours.iter().for_each(|&idx| {
            existing_neighbours.insert(idx);
        });
    }

    fn bron_kerbosh<T>(
        &self,
        on_clique_fn: &mut T,
        required_vertices: &mut Vec<Vertex>,
        mut possible_vertices: HashSet<Vertex>,
        mut excluded_vertices: HashSet<Vertex>,
    ) where
        T: FnMut(&[Vertex]),
    {
        // NOTE: Using sorted Vec instead of HashSet made things slower.
        log::trace!(
            "[bron_kerbosh] required: {:?}, possible: {:?}, excluded: {:?}",
            required_vertices,
            possible_vertices,
            excluded_vertices
        );

        if possible_vertices.is_empty() && excluded_vertices.is_empty() {
            log::debug!("[bron_kerbosh] clique: {:?}", required_vertices);
            (on_clique_fn)(required_vertices.as_slice());
        } else if !possible_vertices.is_empty() {
            // Select a pivot vertex whose neighbors won't be recursed in.
            let pivot = self.bron_kerbosh_pivot(&possible_vertices, &excluded_vertices);
            let pivot_neighbours = &self.neighbours[&pivot];

            let vertices_to_recurse: Vec<Vertex> = possible_vertices
                .difference(pivot_neighbours)
                .copied()
                .collect();

            for &vertex in vertices_to_recurse.iter() {
                // Update vertices lists.
                required_vertices.push(vertex);

                let neighbours = &self.neighbours[&vertex];
                let possible_vertices_recurse: HashSet<Vertex> = neighbours
                    .intersection(&possible_vertices)
                    .copied()
                    .collect();
                let excluded_vertices_recurse: HashSet<Vertex> = neighbours
                    .intersection(&excluded_vertices)
                    .copied()
                    .collect();

                self.bron_kerbosh(
                    on_clique_fn,
                    required_vertices,
                    possible_vertices_recurse,
                    excluded_vertices_recurse,
                );

                // Remove vertex from the list again.
                required_vertices.pop();

                possible_vertices.remove(&vertex);
                excluded_vertices.insert(vertex);
            }
        }
    }

    fn bron_kerbosh_pivot(
        &self,
        possible_vertices: &HashSet<Vertex>,
        excluded_vertices: &HashSet<Vertex>,
    ) -> Vertex {
        // Pick the vertex with the largest amount of neighbors to avoid the
        // maximum amount of recursions.
        *possible_vertices
            .iter()
            .chain(excluded_vertices.iter())
            .max_by_key(|&idx| self.neighbours[idx].len())
            .unwrap()
    }
}

pub trait BronKerbosh {
    fn maximal_cliques<T>(&self, on_clique_fn: T)
    where
        T: FnMut(&[Vertex]);
}

impl BronKerbosh for Graph {
    fn maximal_cliques<T>(&self, mut on_clique_fn: T)
    where
        T: FnMut(&[Vertex]),
    {
        let mut required_vertices: Vec<Vertex> = Vec::new();
        let possible_vertices: HashSet<Vertex> = self.neighbours.keys().copied().collect();
        let empty_vec = HashSet::default();

        self.bron_kerbosh(
            &mut on_clique_fn,
            &mut required_vertices,
            possible_vertices,
            empty_vec,
        );
    }
}
