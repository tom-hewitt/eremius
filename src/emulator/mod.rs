use std::{
    collections::HashMap,
    ops::{Index, IndexMut},
};

use crate::{
    assembler::AssemblyError,
    decoder::{Bits, InvalidInstructionError},
    encoder::Encode,
    ir::{Condition, Imm, RegisterIdentifier, Rs, ShiftedRegister, ShifterOperandCode},
    lexer::Lexer,
    parser::{
        AddressingOffset, AddressingOffsetValue, BranchKind, CalculationKind, ComparisonKind,
        DataProcessingKind, InstructionKind, LoadStoreAddressCode, LoadStoreKind,
        LoadStoreQuantity, MoveKind, Register, SetFlags, Shift,
    },
    preprocessor::PreProcessResult,
    resolver::{ResolvedStatement, SymbolTable},
};

struct Registers([u32; 16]);

impl RegisterIdentifier for u8 {
    fn number(&self) -> u8 {
        *self
    }
}

impl<R: RegisterIdentifier> Index<R> for Registers {
    type Output = u32;

    fn index(&self, index: R) -> &Self::Output {
        &self.0[index.number() as usize]
    }
}

impl<R: RegisterIdentifier> IndexMut<R> for Registers {
    fn index_mut(&mut self, index: R) -> &mut Self::Output {
        &mut self.0[index.number() as usize]
    }
}

struct Emulator {
    memory: [u8; 0xFFFF],
    registers: Registers,
    cpsr: CPSR,
    entry_point: u32,
    symbol_table: SymbolTable<u32>,
    source_map: HashMap<usize, usize>,
}

struct CPSR {
    n: bool,
    z: bool,
    c: bool,
    v: bool,
}

impl Emulator {
    pub fn assemble(&mut self, input: &str) -> Result<(), AssemblyError> {
        let PreProcessResult {
            entry_point,
            statements,
            symbol_table,
            source_map,
        } = Lexer::new(input).parse().preprocess()?;

        let symbol_table = symbol_table.resolve()?;

        // write the statements to memory
        for (mut address, statement) in statements {
            let statement = statement.resolve(&symbol_table, address)?;

            match statement {
                ResolvedStatement::Instructions(instructions) => {
                    for instruction in instructions {
                        for byte in instruction.encode().to_be_bytes() {
                            self.memory[address] = byte;
                            address += 1;
                        }
                    }
                }

                ResolvedStatement::Data(data) => {
                    for byte in data {
                        self.memory[address] = byte;
                        address += 1;
                    }
                }
            }
        }

        // set the PC to the entry point
        self.registers[15] = entry_point as u32;

        Ok(())
    }

    pub fn step(&mut self) -> Result<(), InvalidInstructionError> {
        // get the address from the PC
        let address = self.registers[15] as usize;

        // fetch the instruction
        let instruction: [u8; 4] = self.memory[address..address + 4].try_into().unwrap(); // try_into converts the slice to a fixed size 4-byte array

        // increment the PC
        // TODO: check branch implementation for this?
        self.registers[15] += 4;

        // decode the instruction
        let instruction = InstructionKind::decode(&Bits(u32::from_be_bytes(instruction)))?;

        // execute the instruction
        self.execute(instruction);

        // check docs for cpsr etc, to get correct behaviours

        Ok(())
    }

    fn execute(&mut self, instruction: InstructionKind) {
        match instruction {
            InstructionKind::Branch {
                condition,
                kind,
                target,
            } => {
                if self.cpsr.condition_passed(condition) {
                    if let BranchKind::BranchWithLink = kind {
                        // LR = address of the instruction after the branch instruction
                        self.registers[14] = self.registers[15];
                    }

                    // target is already sign extended by the decoder
                    let offset = target.get() << 2;

                    // safe to cast to u32 because the value should always be positive
                    self.registers[15] = ((self.registers[15] as i32) + offset) as u32;
                }
            }

            InstructionKind::DataProcessing { condition, kind } => {
                if self.cpsr.condition_passed(condition) {
                    match kind {
                        DataProcessingKind::Calculation {
                            kind,
                            set_flags,
                            destination,
                            source,
                            shifter,
                        } => match kind {
                            CalculationKind::ADD => {
                                let register_operand = self.registers[source];
                                let (shifter_operand, _) = self.calculate_shifter(&shifter);
                                let (result, carry) =
                                    u32::carrying_add(register_operand, shifter_operand, false);

                                self.registers[destination] = result;

                                if let SetFlags::Set = set_flags {
                                    self.cpsr.n = (result as i32) < 0;
                                    self.cpsr.z = result == 0;
                                    self.cpsr.c = carry;
                                    self.cpsr.v =
                                        match u32::checked_add(register_operand, shifter_operand) {
                                            Some(_) => false,
                                            None => true,
                                        }
                                }
                            }

                            CalculationKind::SUB => {
                                let register_operand = self.registers[source];
                                let (shifter_operand, _) = self.calculate_shifter(&shifter);
                                let (result, borrow) =
                                    u32::borrowing_sub(register_operand, shifter_operand, false);

                                self.registers[destination] = result;

                                if let SetFlags::Set = set_flags {
                                    self.cpsr.n = (result as i32) < 0;
                                    self.cpsr.z = result == 0;
                                    self.cpsr.c = !borrow;
                                    self.cpsr.v =
                                        match u32::checked_sub(register_operand, shifter_operand) {
                                            Some(_) => false,
                                            None => true,
                                        }
                                }
                            }
                        },

                        DataProcessingKind::Comparison {
                            kind,
                            source,
                            shifter,
                        } => match kind {
                            ComparisonKind::CMP => {
                                let register_operand = self.registers[source];
                                let (shifter_operand, shifter_carry) =
                                    self.calculate_shifter(&shifter);
                                let (result, borrow) =
                                    u32::borrowing_sub(register_operand, shifter_operand, false);

                                self.cpsr.n = (result as i32) < 0;
                                self.cpsr.z = result == 0;
                                self.cpsr.c = !borrow;
                                self.cpsr.v =
                                    match u32::checked_sub(register_operand, shifter_operand) {
                                        Some(_) => false,
                                        None => true,
                                    }
                            }
                        },

                        DataProcessingKind::Move {
                            kind,
                            set_flags,
                            destination,
                            shifter,
                        } => {
                            let (shifter_operand, shifter_carry) = self.calculate_shifter(&shifter);

                            let result = match kind {
                                MoveKind::Move => shifter_operand,
                                MoveKind::MoveNot => !shifter_operand,
                            };

                            self.registers[destination] = result;

                            if let SetFlags::Set = set_flags {
                                self.cpsr.n = (result as i32) < 0;
                                self.cpsr.z = result == 0;
                                self.cpsr.c = shifter_carry;
                            }
                        }
                    }
                }
            }

            _ => todo!(),
        }
    }

    fn calculate_shifter(&mut self, shifter: &ShifterOperandCode) -> (u32, bool) {
        match shifter {
            ShifterOperandCode::Immediate(value) => {
                let shifter_operand = value.get();

                (
                    shifter_operand,
                    if shifter_operand == 0 {
                        self.cpsr.c
                    } else {
                        (shifter_operand as i32) < 0
                    },
                )
            }
            ShifterOperandCode::ImmediateShift(shift) => match shift.kind {
                Shift::LogicalShiftLeft => {
                    let base = self.registers[shift.base];
                    let amount = shift.amount.get();

                    (base << amount, Bits(base)[(32 - amount as usize) % 32] == 1)
                }

                Shift::LogicalShiftRight => {
                    let base = self.registers[shift.base];
                    let amount = shift.amount.get();

                    (base >> amount, Bits(base)[(amount as usize - 1) % 32] == 1)
                }

                Shift::ArithmeticShiftRight => {
                    let base = self.registers[shift.base];
                    let amount = shift.amount.get();

                    if amount == 0 {
                        if (base as i32) < 0 {
                            (0xFFFFFFFF, true)
                        } else {
                            (0, false)
                        }
                    } else {
                        (
                            ((base as i32) >> amount) as u32,
                            Bits(base)[amount as usize - 1] == 1,
                        )
                    }
                }

                Shift::RotateRight => {
                    let base = self.registers[shift.base];
                    let amount = shift.amount.get();

                    if amount == 0 {
                        // Rotate right with extend
                        (
                            // (C Flag Logical_Shift_Left 31) OR (Rm Logical_Shift_Right 1)
                            if self.cpsr.c { 1 } else { 0 } << 31 | base >> 1,
                            // Rm[0]
                            Bits(base)[0] == 1,
                        )
                    } else {
                        (
                            base.rotate_right(amount),
                            Bits(base)[amount as usize - 1] == 1,
                        )
                    }
                }
            },
            ShifterOperandCode::RegisterShift(shift) => match shift.kind {
                Shift::LogicalShiftLeft => {
                    let base = self.registers[shift.base];
                    let amount = Bits(self.registers[shift.amount]).range(0..=7);

                    match amount {
                        0 => (base, self.cpsr.c),
                        1..=31 => (base << amount, Bits(base)[32 - amount as usize] == 1),
                        32 => (base, Bits(base)[0] == 1),
                        _ => (0, false),
                    }
                }

                Shift::LogicalShiftRight => {
                    let base = self.registers[shift.base];
                    let amount = Bits(self.registers[shift.amount]).range(0..=7);

                    match amount {
                        0 => (base, self.cpsr.c),
                        1..=31 => (base >> amount, Bits(base)[amount as usize - 1] == 1),
                        32 => (base, Bits(base)[0] == 1),
                        _ => (0, false),
                    }
                }

                Shift::ArithmeticShiftRight => {
                    let base = self.registers[shift.base];
                    let amount = Bits(self.registers[shift.amount]).range(0..=7);

                    match amount {
                        0 => (base, self.cpsr.c),
                        1..=31 => (
                            ((base as i32) >> amount) as u32,
                            Bits(base)[amount as usize - 1] == 1,
                        ),
                        _ => {
                            if (base as i32) < 0 {
                                (0, false)
                            } else {
                                (0xFFFFFFFF, true)
                            }
                        }
                    }
                }

                Shift::RotateRight => {
                    let base = self.registers[shift.base];
                    let amount = Bits(self.registers[shift.amount]).range(0..=7);

                    if amount == 0 {
                        (base, self.cpsr.c)
                    } else {
                        let amount = Bits(self.registers[shift.amount]).range(0..=4);

                        if amount == 0 {
                            (base, (base as i32) < 0)
                        } else {
                            (
                                base.rotate_right(amount),
                                Bits(base)[amount as usize - 1] == 1,
                            )
                        }
                    }
                }
            },
        }
    }
}

impl CPSR {
    fn condition_passed(&self, condition: Condition) -> bool {
        match condition {
            Condition::EQ => self.z,
            Condition::NE => !self.z,
            Condition::CS => self.c,
            Condition::CC => !self.c,
            Condition::MI => self.n,
            Condition::PL => !self.n,
            Condition::VS => self.v,
            Condition::VC => !self.v,
            Condition::HI => self.c && !self.z,
            Condition::LS => !self.c || self.z,
            Condition::GE => self.n == self.v,
            Condition::LT => self.n != self.v,
            Condition::GT => !self.z && (self.n == self.v),
            Condition::LE => self.z || (self.n != self.v),
            Condition::AL => true,
        }
    }
}

impl CalculationKind {
    fn calculate(&self, a: u32, b: u32) -> u32 {
        match self {
            CalculationKind::ADD => a + b,
            CalculationKind::SUB => a - b,
        }
    }
}

impl Shift {
    fn apply(&self, base: u32, amount: u32) -> u32 {
        match self {
            Shift::LogicalShiftLeft => base << amount,
            Shift::LogicalShiftRight => base >> amount,
            Shift::ArithmeticShiftRight => ((base as i32) >> amount) as u32,
            Shift::RotateRight => base.rotate_right(amount),
        }
    }
}
