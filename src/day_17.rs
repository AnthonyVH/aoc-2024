use itertools::Itertools;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Register {
    A,
    B,
    C,
}

impl Into<usize> for Register {
    fn into(self) -> usize {
        match self {
            Register::A => 0,
            Register::B => 1,
            Register::C => 2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct LiteralOperand(u8);

impl LiteralOperand {
    fn value(&self) -> usize {
        self.0 as usize
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ComboOperand(u8);

impl ComboOperand {
    fn value(&self, state: &State) -> usize {
        match self.0 {
            0..=3 => self.0 as usize,
            4 => state.get(Register::A),
            5 => state.get(Register::B),
            6 => state.get(Register::C),
            _ => unreachable!(),
        }
    }

    fn mapped_register(&self) -> Option<Register> {
        match self.0 {
            0..=3 => None,
            4 => Some(Register::A),
            5 => Some(Register::B),
            6 => Some(Register::C),
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Instruction {
    Adv(ComboOperand),
    Bxl(LiteralOperand),
    Bst(ComboOperand),
    Jnz(LiteralOperand),
    Bxc,
    Out(ComboOperand),
    Bdv(ComboOperand),
    Cdv(ComboOperand),
}

impl Into<Instruction> for &[u8; 2] {
    fn into(self) -> Instruction {
        match self[0] {
            0 => Instruction::Adv(ComboOperand { 0: self[1] }),
            1 => Instruction::Bxl(LiteralOperand { 0: self[1] }),
            2 => Instruction::Bst(ComboOperand { 0: self[1] }),
            3 => Instruction::Jnz(LiteralOperand { 0: self[1] }),
            4 => Instruction::Bxc,
            5 => Instruction::Out(ComboOperand { 0: self[1] }),
            6 => Instruction::Bdv(ComboOperand { 0: self[1] }),
            7 => Instruction::Cdv(ComboOperand { 0: self[1] }),
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct State {
    program_counter: usize,
    registers: [usize; 3],
}

impl State {
    fn get(&self, reg: Register) -> usize {
        match reg {
            Register::A => self.registers[0],
            Register::B => self.registers[1],
            Register::C => self.registers[2],
        }
    }

    fn get_mut(&mut self, reg: Register) -> &mut usize {
        match reg {
            Register::A => &mut self.registers[0],
            Register::B => &mut self.registers[1],
            Register::C => &mut self.registers[2],
        }
    }
}

#[derive(Debug)]
struct Computer {
    state: State,
    instructions: Vec<u8>,
}

impl Computer {
    fn is_done(&self, program_counter: usize) -> bool {
        // Always need to be able to read two words.
        program_counter + 1 >= self.instructions.len()
    }

    fn read_instruction(&self, program_counter: usize) -> Instruction {
        let view: &[u8; 2] = &self.instructions[program_counter..=program_counter + 1]
            .try_into()
            .unwrap();
        view.into()
    }

    fn run(&self, state: State) -> Vec<u8> {
        let mut output = Vec::new();
        let mut push_to_output = |out: u8| -> bool {
            output.push(out);
            true
        };
        self._run_with_callback(state, &mut push_to_output);
        output
    }

    fn _run_with_callback<FnOutput>(&self, mut state: State, mut fn_output: FnOutput) -> bool
    where
        FnOutput: FnMut(u8) -> bool,
    {
        macro_rules! do_div {
            ($reg_src: ident, $reg_dst: ident, $operand: ident) => {{
                *state.get_mut(Register::$reg_dst) =
                    state.get(Register::$reg_src) / usize::pow(2, $operand.value(&state) as u32);
            }};
        }

        while !self.is_done(state.program_counter) {
            match self.read_instruction(state.program_counter) {
                Instruction::Adv(operand) => do_div!(A, A, operand),
                Instruction::Bxl(operand) => *state.get_mut(Register::B) ^= operand.value(),
                Instruction::Bst(operand) => {
                    *state.get_mut(Register::B) = operand.value(&state) % 8
                }
                Instruction::Jnz(operand) => {
                    if state.get(Register::A) != 0 {
                        state.program_counter = operand.value();
                        continue; // Don't modify program counter anymore.
                    }
                }
                Instruction::Bxc => *state.get_mut(Register::B) ^= state.get(Register::C),
                Instruction::Out(operand) => {
                    let keep_running = (fn_output)((operand.value(&state) % 8) as u8);
                    if !keep_running {
                        return false;
                    }
                }
                Instruction::Bdv(operand) => do_div!(A, B, operand),
                Instruction::Cdv(operand) => do_div!(A, C, operand),
            }

            state.program_counter += 2;
            log::trace!("Advancing to PC {}", state.program_counter);
        }

        true
    }

    fn reversed_backtracking(&self) -> usize {
        // NOTE: This is a crappy implementation that only works for a very
        // specific input, because I couldn't get a reverse running
        // implementation to work properly.
        let a_shifts: Vec<_> = (0..self.instructions.len())
            .step_by(2)
            .filter_map(|ctr| {
                let instruction = self.read_instruction(ctr);
                match instruction {
                    Instruction::Adv(operant) => Some(operant),
                    _ => None,
                }
            })
            .collect();
        assert_eq!(a_shifts.len(), 1);

        // We need A to be shifted by a fixed amount.
        assert_eq!(a_shifts[0].mapped_register(), None);

        // Find solution backwards, assuming that B & C registers are zero.
        let state = State {
            program_counter: 0,
            registers: [0, 0, 0],
        };
        let num_bit_shifts = a_shifts[0].value(&state) as u32;
        let mut output = Vec::new();
        self._reversed_backtracking_recurse(
            num_bit_shifts,
            state,
            self.instructions.len(),
            &mut output,
        )
        .unwrap()
    }

    fn _reversed_backtracking_recurse(
        &self,
        num_bit_shifts: u32,
        mut state: State,
        num_outputs_remaining: usize,
        output: &mut Vec<u8>,
    ) -> Option<usize> {
        if num_outputs_remaining == 0 {
            return Some(state.get(Register::A));
        }

        let prev_a_shifted: usize = state.get(Register::A) << num_bit_shifts;
        for a_lsbs in 0..2usize.pow(num_bit_shifts) {
            *state.get_mut(Register::A) = prev_a_shifted | a_lsbs;

            // If this state results in the wanted output, then recurse, if not
            // try the next option.
            let mut output_idx = num_outputs_remaining - 1;
            let check_ouput = |output: u8| -> bool {
                let output_correct = self.instructions[output_idx] == output;
                output_idx += 1;
                output_correct
            };

            let output_ok = self._run_with_callback(state, check_ouput);
            log::debug!(
                "# outputs remaining: {:2}, reg A: {:16} => output {}",
                num_outputs_remaining,
                state.get(Register::A),
                match output_ok {
                    true => String::from("ok"),
                    false => format!("# {} wrong", num_outputs_remaining - output_idx),
                }
            );

            if !output_ok {
                continue;
            }

            let next = self._reversed_backtracking_recurse(
                num_bit_shifts,
                state,
                num_outputs_remaining - 1,
                output,
            );
            if next.is_some() {
                return next;
            }
        }

        None
    }
}

impl std::str::FromStr for Computer {
    type Err = std::string::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.lines().filter(|e| !e.is_empty()).map(|e| {
            let pos = e.find(':').unwrap();
            &e[pos + 2..]
        });

        Ok(Computer {
            state: State {
                program_counter: 0,
                registers: [
                    lines.next().unwrap().parse().unwrap(),
                    lines.next().unwrap().parse().unwrap(),
                    lines.next().unwrap().parse().unwrap(),
                ],
            },
            instructions: lines
                .next()
                .unwrap()
                .as_bytes()
                .iter()
                .copied()
                .filter(|&e| e != b',')
                .map(|e| e - b'0')
                .collect(),
        })
    }
}

pub fn part_a(input: &str) -> String {
    let computer: Computer = input.parse().unwrap();
    let output = computer.run(computer.state);
    itertools::join(output.iter().map(|e| format!("{}", e)), ",")
}

pub fn part_b(input: &str) -> usize {
    let computer: Computer = input.parse().unwrap();
    log::debug!(
        "Instructions:\n{}",
        (0..computer.instructions.len())
            .step_by(2)
            .map(|idx| format!("{:?}", computer.read_instruction(idx)))
            .join("\n")
    );
    computer.reversed_backtracking()
}

#[cfg(test)]
mod tests {
    #[test]
    fn example_a() {
        util::run_test(|| {
            let expected: &str = "4,6,3,5,6,3,5,2,1,0";
            assert_eq!(
                crate::day_17::part_a(&util::read_resource("example_17-part_1.txt").unwrap()),
                expected
            );
        });
    }

    #[test]
    fn example_b() {
        util::run_test(|| {
            let expected: usize = 117440;
            assert_eq!(
                crate::day_17::part_b(&util::read_resource("example_17-part_2.txt").unwrap()),
                expected
            );
        });
    }
}
