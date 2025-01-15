use rayon::prelude::*;
use rustc_hash::FxHashMap as HashMap; // 37% speed-up vs default HashMap
use std::{cell::RefCell, sync::RwLock};

type Stone = u64;

#[derive(Debug, Copy, Clone)]
enum StoneEvolution {
    Single(Stone),
    Split((Stone, Stone)),
}

impl StoneEvolution {
    fn blink(stone: Stone) -> StoneEvolution {
        const MULTIPLIER: u64 = 2024;

        match stone {
            0 => StoneEvolution::Single(1),
            e => {
                let num_digits = util::digit_width_base10(e);
                if num_digits % 2 == 0 {
                    let power = 10u64.pow(num_digits / 2);
                    StoneEvolution::Split((e / power, e % power))
                } else {
                    StoneEvolution::Single(e * MULTIPLIER)
                }
            }
        }
    }
}

trait SolverCache {
    fn init(&mut self, max_num_blinks: u8);
    fn num_stones(&self, stone: Stone, num_blinks: u8) -> Option<usize>;
    fn try_store(&self, stone: Stone, num_blinks: u8, num_stones: usize) -> bool;
}

struct SingleThreadedSolverCache {
    // NOTE: Performance cost to using RefCell here. Required to make the
    // SolverCache trait work for multi-threaded implementations as well (which
    // require that self is always pass non-mutable).
    cache: RefCell<Vec<HashMap<u64, usize>>>,
    granularity: u8,
}

impl SingleThreadedSolverCache {
    fn new(granularity: u8) -> SingleThreadedSolverCache {
        // Allow for some granularity on when to cache, so the cache doesn't
        // become massive.
        SingleThreadedSolverCache {
            cache: RefCell::new(Vec::new()),
            granularity: granularity,
        }
    }

    fn _cache_index(&self, num_blinks: u8) -> usize {
        assert!(self._is_cached_num_blinks(num_blinks));
        (num_blinks / self.granularity) as usize
    }

    fn _is_cached_num_blinks(&self, num_blinks: u8) -> bool {
        num_blinks % self.granularity == 0
    }
}

impl SolverCache for SingleThreadedSolverCache {
    fn init(&mut self, max_num_blinks: u8) {
        let num_entries = (max_num_blinks / self.granularity) as usize + 1;
        self.cache
            .borrow_mut()
            .resize(num_entries, HashMap::default());
    }

    fn num_stones(&self, stone: Stone, num_blinks: u8) -> Option<usize> {
        match self._is_cached_num_blinks(num_blinks) {
            false => None,
            true => {
                let map_idx = self._cache_index(num_blinks);
                self.cache.borrow()[map_idx].get(&stone).copied()
            }
        }
    }

    fn try_store(&self, stone: Stone, num_blinks: u8, num_stones: usize) -> bool {
        match self._is_cached_num_blinks(num_blinks) {
            false => false,
            true => {
                let map_idx = self._cache_index(num_blinks);
                self.cache.borrow_mut()[map_idx].insert(stone, num_stones);
                true
            }
        }
    }
}

impl Drop for SingleThreadedSolverCache {
    fn drop(&mut self) {
        use std::fmt::Write;

        let mut output = String::new();
        writeln!(output, "# cache entries:").unwrap();
        for (num_blinks, val) in self
            .cache
            .borrow()
            .iter()
            .enumerate()
            .map(|(idx, val)| (idx * self.granularity as usize, val))
        {
            writeln!(output, "  {:3} => {}", num_blinks, val.len()).unwrap();
        }
        log::debug!("{}", output);
    }
}

struct MultiThreadedSolverCache {
    // Can't wrap a SingleThreadedSolverCache in an RwLock, because that would
    // force always locking even to call functions that don't require the lock.
    cache: Vec<RwLock<HashMap<u64, usize>>>,
    granularity: u8,
}

impl MultiThreadedSolverCache {
    fn new(granularity: u8) -> MultiThreadedSolverCache {
        MultiThreadedSolverCache {
            cache: Vec::new(),
            granularity: granularity,
        }
    }

    fn _cache_index(&self, num_blinks: u8) -> usize {
        assert!(self._is_cached_num_blinks(num_blinks));
        (num_blinks / self.granularity) as usize
    }

    fn _is_cached_num_blinks(&self, num_blinks: u8) -> bool {
        num_blinks % self.granularity == 0
    }
}

impl SolverCache for MultiThreadedSolverCache {
    fn init(&mut self, max_num_blinks: u8) {
        let num_entries = (max_num_blinks / self.granularity) as usize + 1;
        self.cache = (0..num_entries)
            .map(|_| RwLock::new(HashMap::default()))
            .collect();
    }

    fn num_stones(&self, stone: Stone, num_blinks: u8) -> Option<usize> {
        match self._is_cached_num_blinks(num_blinks) {
            false => None,
            true => {
                let idx = self._cache_index(num_blinks);
                self.cache[idx].read().unwrap().get(&stone).copied()
            }
        }
    }

    fn try_store(&self, stone: Stone, num_blinks: u8, num_stones: usize) -> bool {
        match self._is_cached_num_blinks(num_blinks) {
            false => false,
            true => {
                let idx = self._cache_index(num_blinks);
                self.cache[idx].write().unwrap().insert(stone, num_stones);
                true
            }
        }
    }
}

impl Drop for MultiThreadedSolverCache {
    fn drop(&mut self) {
        use std::fmt::Write;

        let mut output = String::new();
        writeln!(output, "# cache entries:").unwrap();
        for (num_blinks, val) in self
            .cache
            .iter()
            .enumerate()
            .map(|(idx, val)| (idx * self.granularity as usize, val))
        {
            writeln!(
                output,
                "  {:3} => {}",
                num_blinks,
                val.read().unwrap().len()
            )
            .unwrap();
        }
        log::debug!("{}", output);
    }
}

struct Solver<T>
where
    T: SolverCache,
{
    cache: T,
}

impl<T> Solver<T>
where
    T: SolverCache,
{
    fn new(cache: T) -> Solver<T> {
        Solver { cache: cache }
    }

    fn _num_stones_recursive(cache: &T, stone: Stone, num_blinks: u8) -> usize {
        // TODO: Convert to a manual stack-based "iterative" version.
        match cache.num_stones(stone, num_blinks) {
            Some(x) => x,
            None => {
                match num_blinks {
                    0 => 1,
                    _ => {
                        // Blink once to "evolve" the stone and return the number of
                        // stones generated by the evolved stone(s).
                        let num_stones = match StoneEvolution::blink(stone) {
                            StoneEvolution::Single(x) => {
                                Solver::_num_stones_recursive(cache, x, num_blinks - 1)
                            }
                            StoneEvolution::Split((x, y)) => {
                                // NOTE: Parrallelizing this with rayon::join
                                // significantly increases runtime.
                                Self::_num_stones_recursive(cache, x, num_blinks - 1)
                                    + Self::_num_stones_recursive(cache, y, num_blinks - 1)
                            }
                        };
                        cache.try_store(stone, num_blinks, num_stones);
                        num_stones
                    }
                }
            }
        }
    }

    fn _solve_sequential(&self, stones: &[Stone], num_blinks: u8) -> usize {
        // Count the number of stones each of the starting stones evolve into.
        stones
            .iter()
            .map(|e| {
                let num_stones = Self::_num_stones_recursive(&self.cache, *e, num_blinks);
                log::debug!("Stone({:7}) => # stones: {}", e, num_stones);
                num_stones
            })
            .sum()
    }

    fn _solve_parallel(&self, stones: &[Stone], num_blinks: u8) -> usize
    where
        T: Sync,
    {
        // Count the number of stones each of the starting stones evolve into.
        stones
            .par_iter()
            .map(|e| {
                let num_stones = Self::_num_stones_recursive(&self.cache, *e, num_blinks);
                log::debug!("Stone({:7}) => # stones: {}", e, num_stones);
                num_stones
            })
            .sum()
    }

    fn solve(&mut self, stones: &[Stone], num_blinks: u8, parallelize: bool) -> usize
    where
        T: ParallelSolverForwarder<T>,
    {
        // Pre-calculate when the cache would store data.
        self.cache.init(num_blinks);

        if parallelize {
            T::forward_parallel_solve(self, stones, num_blinks)
        } else {
            self._solve_sequential(stones, num_blinks)
        }
    }
}

// Do incredibly disgusting things to allow only the MultiThreadedSolverCache to
// call the parallel solving function.
trait ParallelSolverForwarder<T>
where
    T: SolverCache,
{
    fn forward_parallel_solve(_solver: &Solver<T>, _stones: &[Stone], _num_blinks: u8) -> usize {
        unreachable!()
    }
}

impl ParallelSolverForwarder<SingleThreadedSolverCache> for SingleThreadedSolverCache {}

impl ParallelSolverForwarder<MultiThreadedSolverCache> for MultiThreadedSolverCache {
    fn forward_parallel_solve(solver: &Solver<Self>, stones: &[Stone], num_blinks: u8) -> usize {
        solver._solve_parallel(stones, num_blinks)
    }
}

fn parse_and_solve(input: &str, num_blinks: u8, parallelize: bool) -> usize {
    let stones: Vec<Stone> = input
        .lines()
        .next()
        .unwrap()
        .split_whitespace()
        .map(|e| e.parse().unwrap())
        .collect();

    // NOTE: Solver is generic, so it's actual type differs depending on
    // whether we parallelize or not.
    if parallelize {
        let mut solver = Solver::new(MultiThreadedSolverCache::new(3));
        solver.solve(&stones, num_blinks, parallelize)
    } else {
        let mut solver = Solver::new(SingleThreadedSolverCache::new(3));
        solver.solve(&stones, num_blinks, parallelize)
    }
}

pub fn part_a(input: &str) -> usize {
    const NUM_BLINKS: u8 = 25;
    parse_and_solve(input, NUM_BLINKS, false)
}

pub fn part_b(input: &str, parallelize: bool) -> usize {
    const NUM_BLINKS: u8 = 75;
    parse_and_solve(input, NUM_BLINKS, parallelize)
}

#[cfg(test)]
mod tests {
    #[test]
    fn example_a() {
        util::run_test(|| {
            let expected: usize = 55312;
            assert_eq!(
                crate::day_11::part_a(&util::read_resource("example_11.txt").unwrap()),
                expected
            );
        });
    }

    // There is no example B for this day.
}
