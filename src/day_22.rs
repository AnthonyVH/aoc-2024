use rayon::prelude::*;
use std::{
    simd::{num::SimdUint, Simd},
    sync::{Arc, Mutex},
};

#[derive(Clone, Copy)]
struct Sequence(u32);

impl Sequence {
    fn next(&self) -> Self {
        let mut value = self.0;

        value ^= value << u32::ilog2(64); // Multiply by 64.
        value %= 0x1_00_00_00;

        // No need to reduce after this division, since it can never make the
        // value larger than it was before.
        value ^= value >> u32::ilog2(32); // Divide by 32.

        value ^= value << u32::ilog2(2048); // Multiply by 2048.
        value %= 0x1_00_00_00;

        Sequence { 0: value }
    }

    fn bananas(&self) -> u8 {
        (self.0 % 10) as u8
    }
}

struct SequenceIter {
    value: Sequence,
    count: u32,
}

impl SequenceIter {
    const MAX_COUNT: u32 = 2_000;
}

impl Iterator for SequenceIter {
    type Item = Sequence;

    fn next(&mut self) -> Option<Self::Item> {
        match self.count > Self::MAX_COUNT {
            true => None,
            false => {
                let result = Some(self.value);
                self.value = self.value.next();
                self.count += 1;
                result
            }
        }
    }
}

impl IntoIterator for Sequence {
    type Item = Sequence;
    type IntoIter = SequenceIter;

    fn into_iter(self) -> Self::IntoIter {
        SequenceIter {
            value: self,
            count: 0,
        }
    }
}

struct Window {
    value: u32,
}

impl Window {
    const BASE: usize = 19;
    const LENGTH: usize = 4;
    const NUM_ENCODED_INDICES: usize = Self::BASE.pow(4);

    fn new() -> Window {
        Window { value: 0 }
    }

    /// Push a new value into the window.
    /// Returns the encoding for the window.
    fn push_and_encode(&mut self, diff: i8) -> u32 {
        // Since a difference is between -9 to 9, there's only 19 possible
        // values. Hence we can encode a 4-element window as a 32 bit value.
        // This also ensures all possible windows are encoded into a contiguous
        // range, which means we can use e.g. a Vec to map them. There's approx.
        // 130k, so no issue storage-wise (around half a MiB if storing u32).

        assert!((diff >= -9) && (diff <= 9));
        self.value %= Self::BASE.pow((Self::LENGTH - 1) as u32) as u32; // Remove "MSB".
        self.value *= Self::BASE as u32; // Shift all "digits" one place up.
        self.value += (diff + 9) as u32;

        // NOTE: This assert doesn't affect benchmarked time.
        assert!(self.value < Self::NUM_ENCODED_INDICES as u32);

        self.value
    }
}

#[derive(Debug, Clone, Copy)]
struct WindowState {
    // Since every access of sums also requires an access of last_seen_by at the
    // same index, put them together to improve cache locality.
    sum: u16,
    last_seen_by: u16,
}

#[derive(Debug, Clone)]
struct MarketState {
    state: Vec<WindowState>,
}

impl MarketState {
    fn new() -> MarketState {
        log::trace!("New MarketState created");
        MarketState {
            state: vec![
                WindowState {
                    sum: 0,
                    last_seen_by: 0
                };
                Window::NUM_ENCODED_INDICES
            ],
        }
    }

    fn process_monkey_bidding(&mut self, monkey_idx: u16, secret: Sequence) {
        // Take a 4-entry sliding window over difference of the values. Store the
        // first occurence of each such window together with the amount of bananas
        // offered when this window is seen.

        // Since the window of differences is encoded into a contiguous range, we
        // can keep track of them using a vector instead of a hashmap.

        // NOTE: Summing together an vector for each monkey is extremely slow. It's
        // also extremely inefficient, because less than 2000 out of 130k elements
        // will be set. Hence we sum into a shared vector.

        // NOTE: To avoid having to allocate a "saw index" vector per monkey, we use
        // a shared vector here as well.

        let mut seq_iter = secret.into_iter();
        let mut window = Window::new();
        let mut prev_bananas = seq_iter.next().unwrap().bananas();

        // Prime the window.
        for _ in 0..Window::LENGTH {
            let next_bananas = seq_iter.next().unwrap().bananas();
            let diff: i8 = (next_bananas as i8) - (prev_bananas as i8);

            window.push_and_encode(diff);
            prev_bananas = next_bananas;
        }

        // Store number of bananas for each first-time occurence of a window.
        for value in seq_iter {
            let bananas = value.bananas();
            let diff: i8 = (bananas as i8) - (prev_bananas as i8);
            let window_idx = window.push_and_encode(diff) as usize;

            // We know the index is in range.
            let state = &mut self.state[window_idx];
            let first_occurence = state.last_seen_by != monkey_idx;
            // NOTE: This seems ever so slightly faster than multiplying with bool.
            state.sum += if first_occurence { bananas as u16 } else { 0 };
            state.last_seen_by = monkey_idx;

            prev_bananas = bananas;
        }
    }
}

#[derive(Debug)]
struct MarketStateBuilder {
    states: Vec<Arc<Mutex<MarketState>>>,
}

impl MarketStateBuilder {
    fn new() -> MarketStateBuilder {
        MarketStateBuilder { states: vec![] }
    }

    fn build(&mut self) -> Arc<Mutex<MarketState>> {
        let state = Arc::new(Mutex::new(MarketState::new()));
        self.states.push(state.clone());
        state
    }
}

const SIMD_LANES_PART_A: usize = 16;
struct SecretSimd(Simd<u32, SIMD_LANES_PART_A>);

impl SecretSimd {
    /// Instead of masking after each step (well, the 1st and 3rd one) we pre-
    /// shift the values up such that the useless bits are on the LSB side, and
    /// not on the MSB side. This allows replacing the two mask operations with
    /// a single one after the division instead.
    const VALUE_BIT_W: usize = 24;
    const NUM_SHIFT_POS: usize = 32 - Self::VALUE_BIT_W;

    fn new(start_secrets: &[u32]) -> SecretSimd {
        assert!(start_secrets.len() <= SIMD_LANES_PART_A);
        SecretSimd {
            0: Simd::<u32, SIMD_LANES_PART_A>::load_or_default(start_secrets)
                << (Self::NUM_SHIFT_POS as u32),
        }
    }

    fn values(&self) -> Simd<u32, SIMD_LANES_PART_A> {
        self.0 >> (Self::NUM_SHIFT_POS as u32)
    }

    fn advance_n(&mut self, num_steps: usize) {
        let mask = Simd::<u32, SIMD_LANES_PART_A>::splat(!0xFFu32);

        for _ in 0..num_steps {
            self.0 ^= self.0 << u32::ilog2(64); // Multiply by 64.
            self.0 ^= self.0 >> u32::ilog2(32); // Divide by 32.
            self.0 &= mask; // Clear LSBs before upshifting.
            self.0 ^= self.0 << u32::ilog2(2048); // Multiply by 2048.
        }
    }
}

pub fn part_a(input: &str) -> u64 {
    // Gather all starting seeds in a Vec first, to allow chunking them up in
    // parallel afterwards.
    // NOTE: Pre-allocating a vector with a capacity doesn't improve runtime.
    let seeds: Vec<u32> = input
        .lines()
        .map(|e| -> u32 { e.parse().unwrap() })
        .collect();

    seeds
        .par_chunks(SIMD_LANES_PART_A) // Process N elements simultaneously.
        .map(|chunk| {
            let mut secrets = SecretSimd::new(chunk);
            secrets.advance_n(2_000);
            secrets.values().cast()
        })
        .sum::<Simd<u64, SIMD_LANES_PART_A>>()
        .reduce_sum()
}

pub fn part_b(input: &str) -> u64 {
    // Use a shared buffer to sum into, which is much more efficient than
    // creating a full length buffer for each monkey, storing into that and then
    // unconditionally summing each element into a shared buffer later. This is
    // a waste both due to the allocation per monkey, as well as the fact that
    // less than 2000 elements out of >130k will be non-zero.

    // NOTE: Using a HashMap to store entries per monkey, and then accumulating
    // those into a Vec works well too. But it doesn't parralelize quite as well
    // as the current approach of using thread-local shared state (about a
    // factor 3 slower).

    // NOTE: Doing this in parallel for multiple monkeys at a time using SIMD
    // works. However, it still ends up being slower by a factor of 4, due to
    // the complex logic required to handle gather-scatter to potentially
    // identical indices into the sum & last_seen vectors. This requires
    // detecting whether any of the generated indices across the SIMD lanes are
    // identical, which is very expensive (except on CPUs which support the
    // AVX-512CD instruction set, but those are all very new and high-end).
    // Based on whether there are conflicts (which is the minority of the time),
    // the scatter operation needs to be done sequentially (i.e. via non-SIMD).

    // NOTE: Pre-parsing the input and iterating over it using par_iter() is
    // slower by a factor 6, for some reason...

    // NOTE: Keeping track of the maximum sum seen in process_monkey_bidding()
    // and then selecting the maximum from that is slower by about 15%.

    let state_builder: Mutex<MarketStateBuilder> = Mutex::new(MarketStateBuilder::new());

    input
        .lines()
        .map(|e| Sequence {
            0: e.parse().unwrap(),
        })
        .enumerate()
        .par_bridge()
        .for_each_init(
            // Create threat-local state using ..._init.
            || state_builder.lock().unwrap().build(),
            |local_state, (idx, secret)| {
                {
                    // Only one thread should be accessing this state anyway.
                    let mut guard = local_state.lock().unwrap();
                    guard.process_monkey_bidding((idx + 1) as u16, secret);
                }
            },
        );
    log::trace!("# states: {}", state_builder.lock().unwrap().states.len());

    // Reduce all the sums in the list of shared states. Also do some magic to
    // extract MarketState's "sums" from Arc<Mutex<...>>.
    let state = Arc::try_unwrap(
        state_builder
            .into_inner() // Extract from Mutex<...>.
            .unwrap()
            .states
            .into_iter()
            .reduce(|acc, e| {
                // Sum together all the state's "sums" vectors.
                assert!(!Arc::ptr_eq(&acc, &e));
                {
                    let mut acc_guard = acc.lock().unwrap();
                    let elem_guard = e.lock().unwrap();
                    acc_guard
                        .state
                        .iter_mut()
                        .zip(&elem_guard.state)
                        .for_each(|(lhs, rhs)| lhs.sum += rhs.sum);
                }
                acc
            })
            .unwrap(),
    )
    .unwrap() // Here the extraction magic continues.
    .into_inner() // Extract from the Mutex<...>.
    .unwrap()
    .state;

    state.iter().max_by_key(|e| e.sum).unwrap().sum as u64
}

#[cfg(test)]
mod tests {
    #[test]
    fn example_a() {
        util::run_test(|| {
            let expected: u64 = 37327623;
            assert_eq!(
                crate::day_22::part_a(&util::read_resource("example_22-part_a.txt").unwrap()),
                expected
            );
        });
    }

    #[test]
    fn example_b() {
        util::run_test(|| {
            let expected: u64 = 23;
            assert_eq!(
                crate::day_22::part_b(&util::read_resource("example_22-part_b.txt").unwrap()),
                expected
            );
        });
    }
}
