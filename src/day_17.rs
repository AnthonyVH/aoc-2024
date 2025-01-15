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

    fn run(&self, mut state: State) -> Vec<u8> {
        let mut result = Vec::new();

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
                Instruction::Out(operand) => result.push((operand.value(&state) % 8) as u8),
                Instruction::Bdv(operand) => do_div!(A, B, operand),
                Instruction::Cdv(operand) => do_div!(A, C, operand),
            }

            state.program_counter += 2;
            log::trace!("Advancing to PC {}", state.program_counter);
        }

        result
    }

    /// Run the instruction in reverse to determine which value is required in
    /// register A in order to have the program replicate its own instructions
    /// in the output.
    fn run_reversed(&self) -> usize {
        // Only tested to work with a single jump at the end of the program.
        assert!(self.instructions.len() % 2 == 0);
        assert!(
            self.read_instruction(self.instructions.len() - 2)
                == Instruction::Jnz(LiteralOperand(0))
        );

        // Never execute the jump instruction at the end.
        let last_instruction_idx = self.instructions.len() - 2 - 2;

        // The A register must be zero at the end of the program. We assume the
        // other registers are as well.
        let mut state = State {
            program_counter: last_instruction_idx,
            registers: [0, 0, 0],
        };
        let mut num_generated_outputs = 0;

        while (num_generated_outputs != self.instructions.len())
            || (state.program_counter != last_instruction_idx)
        {
            const THREE_LSB_MASK: usize = 7;

            // Compute the inverse of each function.
            log::trace!(
                "Executing {:?} (PC: {})",
                self.read_instruction(state.program_counter),
                state.program_counter
            );
            if state.program_counter == last_instruction_idx {
                log::debug!("Starting from end, A = {}", state.get(Register::A));
            }

            let instruction = self.read_instruction(state.program_counter);

            macro_rules! invert_div {
                ($reg_src: ident, $reg_dst: ident, $operand: ident) => {{
                    assert!($operand.mapped_register() != Some(Register::$reg_dst));
                    let denominator = usize::pow(2, $operand.value(&state) as u32);
                    let reversed =
                    match Register::$reg_src == Register::$reg_dst {
                        true => {
                            // If source and destination are the same, then we can't know
                            // the LSBs so set them to zero.
                            state.get(Register::$reg_src) * denominator
                        },
                        false => {
                            // We might know LSBs about the source register. Keep them.
                            (state.get(Register::$reg_dst) * denominator)
                                + (state.get(Register::$reg_src) & !(denominator - 1))
                        }
                    };

                    log::debug!(
                        "[{:?}] Reversing {:?} = {:?} / {} = {} / {}, set {:?} = {}",
                        instruction,
                        Register::$reg_dst,
                        Register::$reg_src,
                        denominator,
                        state.get(Register::$reg_src),
                        denominator,
                        Register::$reg_src,
                        reversed
                    );

                    *state.get_mut(Register::$reg_src) = reversed;

                    // TODO: This is all wrong...

                    // Shift "dst" back by given amount and store back in "src".
                    // If the amount to shift is determined by the "dst" register,
                    // then we can't find the original value.
                    //let reversed =
                    //    (state.get(Register::$reg_dst) * div) | (state.get(Register::$reg_src) & !mask);
                }};
            }

            match instruction {
                Instruction::Adv(operand) => invert_div!(A, A, operand),
                Instruction::Bxl(operand) => {
                    // Inverse of XOR is XOR.
                    log::debug!(
                        "[{:?}] Set {:?} to {} ^ {} = {}",
                        instruction,
                        Register::B,
                        state.get(Register::B),
                        operand.value(),
                        state.get(Register::B) ^ operand.value()
                    );
                    *state.get_mut(Register::B) ^= operand.value();
                }
                Instruction::Bst(operand) => {
                    // The value of B should be at most 3 bits, since in a
                    // forward run it gets set to an operand modulo 8.
                    let value_b = state.get(Register::B);
                    assert!(value_b & !THREE_LSB_MASK == 0);

                    // Set the bottom 3 bits of the target operand to the
                    // value stored in B.
                    match operand.mapped_register() {
                        None => assert!(operand.value(&state) & THREE_LSB_MASK == value_b),
                        Some(reg) => {
                            log::debug!(
                                "[{:?}] Set {:?} to {:?} & {} | {:?} = {} & {} | {} = {}",
                                instruction,
                                reg,
                                reg,
                                THREE_LSB_MASK,
                                Register::B,
                                state.get(reg),
                                THREE_LSB_MASK,
                                value_b,
                                (state.get(reg) & !THREE_LSB_MASK) | value_b
                            );
                            *state.get_mut(reg) = (state.get(reg) & !THREE_LSB_MASK) | value_b;
                        }
                    }

                    // We can't possibly know what value B had before this
                    // though. Just set it to 0 for now.
                    // TODO: For sure this is going to cause issues...
                    *state.get_mut(Register::B) = 0;
                }
                Instruction::Jnz(_) => unreachable!(),
                Instruction::Bxc => {
                    // Inverse of XOR is XOR.
                    log::debug!(
                        "[{:?}] Set {:?} to {:?} ^ {:?} = {} ^ {} = {}",
                        instruction,
                        Register::B,
                        Register::B,
                        Register::C,
                        state.get(Register::B),
                        state.get(Register::C),
                        state.get(Register::B) ^ state.get(Register::C),
                    );
                    *state.get_mut(Register::B) ^= state.get(Register::C);
                }
                Instruction::Out(operand) => {
                    // Set the bottom bits of the operand to the expected output.
                    num_generated_outputs += 1;
                    let idx = self.instructions.len() - num_generated_outputs;

                    match operand.mapped_register() {
                        Some(reg) => {
                            log::debug!(
                                "[{:?}] Output {}, {:?} was {}, now {}",
                                instruction,
                                self.instructions[idx],
                                reg,
                                state.get(reg),
                                (state.get(reg) & !THREE_LSB_MASK)
                                    | self.instructions[idx] as usize
                            );
                            *state.get_mut(reg) = (state.get(reg) & !THREE_LSB_MASK)
                                | self.instructions[idx] as usize;
                        }
                        None => {
                            let value = (operand.value(&state) % 8) as u8;
                            assert!(value == self.instructions[idx]);
                        }
                    }
                }
                Instruction::Bdv(operand) => invert_div!(A, B, operand),
                Instruction::Cdv(operand) => invert_div!(A, C, operand),
            }

            state.program_counter = state
                .program_counter
                .checked_sub(2)
                .unwrap_or(last_instruction_idx);
            log::trace!("Jumping back to PC {}", state.program_counter);
        }

        // NOTE: When we get here B and C should be 0.
        log::debug!("Registers: {:?}", state.registers);

        let forward_output = self.run(State {
            program_counter: 0,
            registers: state.registers,
        });
        log::debug!(
            "Found {:?} = {}, instructions: {:?}, forward run: {:?}",
            Register::A,
            state.get(Register::A),
            self.instructions,
            forward_output
        );
        assert!(forward_output == self.instructions);
        state.get(Register::A)
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
    computer.run_reversed()
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
