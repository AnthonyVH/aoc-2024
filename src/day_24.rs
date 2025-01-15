use arrayvec::ArrayVec;
use itertools::Itertools;
use rustc_hash::FxHashMap as HashMap;
use std::collections::VecDeque;

#[derive(Debug, PartialEq, Eq)]
enum GateKind {
    AND,
    OR,
    XOR,
}

impl GateKind {
    fn evaluate(&self, inputs: &[bool; 2]) -> bool {
        match self {
            GateKind::AND => inputs[0] & inputs[1],
            GateKind::OR => inputs[0] | inputs[1],
            GateKind::XOR => inputs[0] ^ inputs[1],
        }
    }
}

// NOTE: It's a bit wasteful to store the value of an input, but it makes things
// faster when evaluating. And obviously we're optimizing for speed, not memory.
#[derive(Debug)]
struct Gate {
    kind: GateKind,
    inputs: [Option<bool>; 2],
}

impl Gate {
    fn new(kind: GateKind) -> Gate {
        Gate {
            kind,
            inputs: [None; 2],
        }
    }

    fn set_input(&mut self, port_idx: usize, value: bool) {
        assert!(self.inputs[port_idx].is_none());
        self.inputs[port_idx] = Some(value);
    }

    fn evaluate(&self) -> Option<bool> {
        match self.inputs.iter().any(|e| e.is_none()) {
            true => None,
            false => Some(self.kind.evaluate(&self.inputs.map(|e| e.unwrap()))),
        }
    }
}

#[derive(Debug)]
struct GateInput {
    index: usize,
    port: u8,
}

#[derive(Debug)]
struct Problem<'a> {
    gates: HashMap<usize, Gate>,
    connections: HashMap<usize, Vec<GateInput>>,
    output_gates: Vec<usize>,
    initial_values: Vec<(usize, bool)>,
    gate_inputs: HashMap<usize, [usize; 2]>,
    name_to_idx: HashMap<&'a str, usize>,
    idx_to_name: HashMap<usize, &'a str>,
}

impl<'a> TryFrom<&'a str> for Problem<'a> {
    type Error = std::string::ParseError;

    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        let mut lines = s.lines();
        let mut index_next: usize = 0;

        let mut result = Problem {
            gates: HashMap::default(),
            connections: HashMap::default(),
            output_gates: Vec::default(),
            initial_values: Vec::default(),
            gate_inputs: HashMap::default(),
            name_to_idx: HashMap::default(),
            idx_to_name: HashMap::default(),
        };

        let mut get_index = |name: &'a str| -> usize {
            let idx = *result.name_to_idx.entry(name).or_insert_with_key(|_| {
                let index = index_next;
                index_next += 1;
                index
            });
            result.idx_to_name.entry(idx).or_insert(name);
            idx
        };

        // Extract initial states.
        for line in lines.by_ref().take_while(|e| !e.is_empty()) {
            let name = &line[..3];
            let value: bool = line[5..=5].parse::<u8>().unwrap() != 0;
            let idx = get_index(name);
            result.initial_values.push((idx, value));
        }

        // Parse gates.
        for line in lines.skip_while(|e| e.is_empty()) {
            let gate_kind_end = 4 + line[4..].find(' ').unwrap();
            let gate_kind = match &line[4..gate_kind_end] {
                "OR" => GateKind::OR,
                "AND" => GateKind::AND,
                "XOR" => GateKind::XOR,
                _ => unreachable!(),
            };

            let in_a = get_index(&line[0..3]);
            let in_b = get_index(&line[gate_kind_end + 1..gate_kind_end + 4]);
            let out_name = &line[gate_kind_end + 8..];
            let out = get_index(out_name);

            result.gates.insert(out, Gate::new(gate_kind));

            // Name a gate after the output.
            result.connections.entry(in_a).or_default().push(GateInput {
                index: out,
                port: 0,
            });
            result.connections.entry(in_b).or_default().push(GateInput {
                index: out,
                port: 1,
            });

            // Keep track of inputs per gate.
            result
                .gate_inputs
                .entry(out)
                .or_insert([in_a, in_b])
                .sort_unstable();

            // Remember which gates are output ones.
            if out_name.starts_with('z') {
                let number_idx: usize = out_name[1..].parse().unwrap();

                // Indices start at 0, so store at least one more entry.
                if result.output_gates.len() < number_idx + 1 {
                    result.output_gates.resize(number_idx + 1, 0);
                }
                result.output_gates[number_idx] = out;
            }
        }

        assert!(result.output_gates.iter().all(|e| *e != 0));

        Ok(result)
    }
}

impl<'a> Problem<'a> {
    // NOTE: These functions are only implemented as far as was necessary to
    // solve the given input. They might not work on someone else's input.

    fn find_gate_with_input(&self, in_idx: usize, gate_kind: GateKind) -> usize {
        let mut gates = self.connections[&in_idx]
            .iter()
            .filter(|gate_input| self.gates[&gate_input.index].kind == gate_kind);
        assert_eq!(gates.clone().count(), 1);
        gates.next().and_then(|e| Some(e.index)).unwrap()
    }

    /// Check that a half adder has the correct connections, and return the
    /// index of the carry output.
    fn check_half_adder(
        &self,
        _wrong_conns: &mut Vec<usize>,
        input_idx: [usize; 2],
        output_idx: usize,
    ) -> usize {
        // Both inputs should go to the same XOR gate generating the output.
        let xor_idx = self.find_gate_with_input(input_idx[0], GateKind::XOR);
        assert_eq!(self.gate_inputs[&xor_idx], input_idx);
        assert_eq!(xor_idx, output_idx);

        // The carry is generated by both inputs going to an AND gate.
        let carry_idx = self.find_gate_with_input(input_idx[0], GateKind::AND);
        assert_eq!(self.gate_inputs[&carry_idx], input_idx);
        carry_idx
    }

    /// Check that a full adder has the correct connections, and return the
    /// index of the carry output.
    fn check_full_adder(
        &self,
        wrong_conns: &mut Vec<usize>,
        input_idx: [usize; 3],
        output_idx: usize,
    ) -> usize {
        let mut push_swap = |idxes: [usize; 2]| {
            idxes.iter().for_each(|e| wrong_conns.push(*e));
            log::debug!(
                "Found swap for {}",
                idxes.iter().map(|e| self.idx_to_name[e]).join(" & ")
            );
        };

        // Two of the given inputs should go to one XOR gate. Assume it's the
        // two non-carry ones (i.e. first two inputs in the list).
        let input_xor = self.find_gate_with_input(input_idx[0], GateKind::XOR);
        assert_eq!(self.gate_inputs[&input_xor], input_idx[0..2]);

        // The output should be connected to a XOR gate to which one of the
        // inputs is connected.
        match self.gates[&output_idx].kind {
            GateKind::AND | GateKind::OR => {
                // Find out which output is generated by the inputs. That one is
                // swapped as well.
                let output_xor = self.find_gate_with_input(input_xor, GateKind::XOR);
                push_swap([output_idx, output_xor]);

                match self.gates[&output_idx].kind {
                    GateKind::XOR => unreachable!(),
                    GateKind::AND => return self.find_gate_with_input(output_xor, GateKind::OR),
                    GateKind::OR => return output_xor,
                }
            }
            GateKind::XOR => {
                // At least one of the inputs should be an input to the output's
                // XOR gate. If this is not the case, the output is swapped.

                // Find which inputs generate the non-matched output XOR input.
                let non_input_to_output_xor: ArrayVec<usize, 1> = self.gate_inputs[&output_idx]
                    .iter()
                    .filter(|&xor_in_idx| !input_idx.contains(xor_in_idx))
                    .copied()
                    .collect();
                assert_eq!(non_input_to_output_xor.len(), 1);

                match non_input_to_output_xor[0] == input_xor {
                    true => {
                        // All seems well, just return the carry output.
                        let carry_and = self.find_gate_with_input(input_idx[2], GateKind::AND);
                        return self.find_gate_with_input(carry_and, GateKind::OR);
                    }
                    false => {
                        // Input XOR output and input to output XOR are swapped.
                        push_swap([non_input_to_output_xor[0], input_xor]);

                        // Swapped input XOR must be connected to carry OR.
                        return self.find_gate_with_input(input_xor, GateKind::OR);
                    }
                }
            }
        }
    }
}

pub fn part_a(input: &str) -> u64 {
    let mut problem = Problem::try_from(input).unwrap();
    log::trace!("{:#?}", problem);

    // Propagate values until there's nothing left to be done.
    let mut values: VecDeque<(usize, bool)> = problem.initial_values.iter().copied().collect();

    // Set all initial values.
    while !values.is_empty() {
        let (out_idx, value) = values.pop_front().unwrap();

        match problem.connections.get(&out_idx) {
            None => (),
            Some(conns) => {
                for conn in conns {
                    let gate = problem.gates.get_mut(&(conn.index as usize)).unwrap();
                    gate.set_input(conn.port as usize, value);

                    if let Some(gate_value) = gate.evaluate() {
                        values.push_back((conn.index as usize, gate_value));
                    }
                }
            }
        }
    }

    problem
        .output_gates
        .iter()
        .enumerate()
        .map(|(output_pos, gate_idx)| {
            (problem.gates[gate_idx].evaluate().unwrap() as u64) << output_pos
        })
        .sum()
}

pub fn part_b(input: &str) -> String {
    const NUM_SWAPPED_WIRES: usize = 4 * 2;

    let problem = Problem::try_from(input).unwrap();

    // Check that the gates represent a ripple-carry adder. This requires a full
    // adder (5 gates), except for the first bit, which requires only a half
    // adder (2 gates). Furthermore, the last output is the carry of the MSB's
    // full adder, so the number of full adders is equal to the number of output
    // bits minus two.
    assert_eq!(
        2 + (problem.output_gates.len() - 2) * 5,
        problem.gates.len()
    );

    // Note that we don't need to figure out how to fix the gates. We only need
    // to find which ones are wrong. Since we know exactly which kind of circuit
    // we're dealing with, we can simply go through the whole circuit and see
    // which connections are wrong. Because this is a ripple-carry adder, we
    // start at the first bit and work our way to the end.
    let mut wrong_conns: Vec<usize> = Vec::new();
    let max_input_idx = problem.output_gates.len() - 1;

    // Check half adder.
    let mut carry_idx: usize = problem.check_half_adder(
        &mut wrong_conns,
        [problem.name_to_idx["x00"], problem.name_to_idx["y00"]],
        problem.name_to_idx["z00"],
    );

    // Check all the full adders.
    for in_idx in 1..max_input_idx {
        carry_idx = problem.check_full_adder(
            &mut wrong_conns,
            [
                problem.name_to_idx[format!("x{:02}", in_idx).as_str()],
                problem.name_to_idx[format!("y{:02}", in_idx).as_str()],
                carry_idx,
            ],
            problem.name_to_idx[format!("z{:02}", in_idx).as_str()],
        );

        if wrong_conns.len() >= NUM_SWAPPED_WIRES {
            break;
        }
    }
    assert_eq!(wrong_conns.len(), NUM_SWAPPED_WIRES);

    wrong_conns.sort_unstable_by_key(|e| problem.idx_to_name[e]);
    wrong_conns.iter().map(|e| problem.idx_to_name[e]).join(",")
}

#[cfg(test)]
mod tests {
    #[test]
    fn example_a_part_1() {
        util::run_test(|| {
            let expected: u64 = 4;
            assert_eq!(
                crate::day_24::part_a(&util::read_resource("example_24-part_1.txt").unwrap()),
                expected
            );
        });
    }

    #[test]
    fn example_a_part_2() {
        util::run_test(|| {
            let expected: u64 = 2024;
            assert_eq!(
                crate::day_24::part_a(&util::read_resource("example_24-part_2.txt").unwrap()),
                expected
            );
        });
    }

    // Part B is written explicitly to check a carry-chain adder, so won't work
    // for the example.
}
