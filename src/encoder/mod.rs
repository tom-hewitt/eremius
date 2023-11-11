use crate::{
    ir::{
        AddressingOffset, BranchKind, Condition, DataProcessingKind, Imm, InstructionKind,
        LoadStoreAddressCode, LoadStoreKind, LoadStoreQuantity, MoveKind, MultipleAddressingMode,
        OffsetMode, Rd, RegisterList, Rm, Rn, RotatedImm8, Rs, SetFlags, Shift, ShiftedRegister,
        ShifterOperandCode, Sign, SignedImm, WriteBack,
    },
    parser::AddressingOffsetValue,
};

mod bits;
mod tests;

pub trait Encode {
    fn encode(&self) -> u32;
}

impl Encode for InstructionKind {
    fn encode(&self) -> u32 {
        match self {
            InstructionKind::Branch {
                condition,
                kind,
                target,
            } => {
                // ENCODING:
                //
                //  3 3 2 2 2 2 2 2 2 2 2 2 1 1 1 1 1 1 1 1 1 1
                //  1 0 9 8 7 6 5 4 3 2 1 0 9 8 7 6 5 4 3 2 1 0 9 8 7 6 5 4 3 2 1 0
                // |cond   |1 0 1|l|target                                         |
                condition.encode() | 0b101 << 25 | kind.encode() | target.encode()
            }

            InstructionKind::DataProcessing { condition, kind } => {
                // ENCODING:
                //
                //  3 3 2 2 2 2 2 2 2 2 2 2 1 1 1 1 1 1 1 1 1 1
                //  1 0 9 8 7 6 5 4 3 2 1 0 9 8 7 6 5 4 3 2 1 0 9 8 7 6 5 4 3 2 1 0
                // |cond   |0 0|I|op     |S|Rn     |Rd     |shifter                |

                match kind {
                    DataProcessingKind::Move {
                        kind,
                        set_flags,
                        destination,
                        shifter,
                    } => {
                        // ENCODING:
                        //
                        //  3 3 2 2 2 2 2 2 2 2 2 2 1 1 1 1 1 1 1 1 1 1
                        //  1 0 9 8 7 6 5 4 3 2 1 0 9 8 7 6 5 4 3 2 1 0 9 8 7 6 5 4 3 2 1 0
                        // |cond   |0 0|I|1 1|N|1|S|SBZ    |Rd     |shifter                |

                        condition.encode()
                            | 0b11 << 23
                            | kind.encode()
                            | 1 << 21
                            | destination.encode()
                            | shifter.encode()
                    }

                    DataProcessingKind::Comparison {
                        kind,
                        source,
                        shifter,
                    } => {
                        // ENCODING:
                        //
                        //  3 3 2 2 2 2 2 2 2 2 2 2 1 1 1 1 1 1 1 1 1 1
                        //  1 0 9 8 7 6 5 4 3 2 1 0 9 8 7 6 5 4 3 2 1 0 9 8 7 6 5 4 3 2 1 0
                        // |cond   |0 0|I|1 0 1 0|1|R n    |SBZ    |shifter                |
                        condition.encode()
                            | 0b1010 << 21
                            | 1 << 20
                            | source.encode()
                            | shifter.encode()
                    }

                    DataProcessingKind::Calculation {
                        kind,
                        set_flags,
                        destination,
                        source,
                        shifter,
                    } => {
                        // ENCODING:
                        //
                        //  3 3 2 2 2 2 2 2 2 2 2 2 1 1 1 1 1 1 1 1 1 1
                        //  1 0 9 8 7 6 5 4 3 2 1 0 9 8 7 6 5 4 3 2 1 0 9 8 7 6 5 4 3 2 1 0
                        // |cond   |0 0|I|0 1 0 0|S|Rn     |Rd     |shifter                |
                        condition.encode()
                            | 0b0100 << 21
                            | set_flags.encode()
                            | source.encode()
                            | destination.encode()
                            | shifter.encode()
                    }
                }
            }

            InstructionKind::LoadStore {
                condition,
                kind,
                quantity,
                destination,
                address,
            } => {
                // ENCODING:
                //
                //  3 3 2 2 2 2 2 2 2 2 2 2 1 1 1 1 1 1 1 1 1 1
                //  1 0 9 8 7 6 5 4 3 2 1 0 9 8 7 6 5 4 3 2 1 0 9 8 7 6 5 4 3 2 1 0
                // |cond   |0 1|I|P|U|B|W|L|Rn     |Rd     |offset                 |
                condition.encode()
                    | 1 << 26
                    | kind.encode()
                    | quantity.encode()
                    | destination.encode()
                    | address.encode()
            }

            InstructionKind::LoadStoreMultiple {
                condition,
                kind,
                mode,
                base,
                write_back,
                register_list,
            } => {
                // ENCODING:
                //
                //  3 3 2 2 2 2 2 2 2 2 2 2 1 1 1 1 1 1 1 1 1 1
                //  1 0 9 8 7 6 5 4 3 2 1 0 9 8 7 6 5 4 3 2 1 0 9 8 7 6 5 4 3 2 1 0
                // |cond   |1 0 0|P|U|S|W|L|Rn     |register list                  |

                // S is unpredictable in user mode, so we're currently ignoring it

                condition.encode()
                    | 0b100 << 25
                    | mode.encode()
                    | write_back.encode()
                    | base.encode()
                    | register_list.encode()
            }

            InstructionKind::SuperVisorCall {
                condition,
                immediate,
            } => condition.encode() | immediate.encode(),
        }
    }
}

impl Encode for Condition {
    /// sets bits `28` to `31`
    fn encode(&self) -> u32 {
        (*self as u32) << 28
    }
}

impl<const N: u32> Encode for Imm<N> {
    /// sets bits `0` to `N`
    fn encode(&self) -> u32 {
        self.get() as u32
    }
}

impl<const N: u32> Encode for SignedImm<N> {
    /// sets bits `0` to `N`
    fn encode(&self) -> u32 {
        (self.get() as u32) & bits::bottom_n(N)
    }
}

impl Encode for BranchKind {
    /// sets bit `24`
    fn encode(&self) -> u32 {
        match self {
            BranchKind::BranchWithLink => 1 << 24,
            BranchKind::Branch => 0,
        }
    }
}

impl Encode for MoveKind {
    /// sets bit `22`
    fn encode(&self) -> u32 {
        match self {
            MoveKind::Move => 0,
            MoveKind::MoveNot => 1 << 22,
        }
    }
}

impl Encode for SetFlags {
    /// sets bit `20`
    fn encode(&self) -> u32 {
        match self {
            SetFlags::Set => 1 << 20,
            SetFlags::DontSet => 0,
        }
    }
}

impl Encode for LoadStoreKind {
    /// sets bit `20`
    fn encode(&self) -> u32 {
        match self {
            LoadStoreKind::Load => 1 << 20,
            LoadStoreKind::Store => 0,
        }
    }
}

impl Encode for LoadStoreQuantity {
    /// sets bit `22`
    fn encode(&self) -> u32 {
        match self {
            LoadStoreQuantity::Byte => 1 << 22,
            LoadStoreQuantity::Word => 0,
        }
    }
}

impl Encode for Rn {
    /// sets bits `16` to `19`
    fn encode(&self) -> u32 {
        (self.0 as u32) << 16
    }
}

impl Encode for Rd {
    /// sets bits `12` to `15`
    fn encode(&self) -> u32 {
        (self.0 as u32) << 12
    }
}

impl Encode for Rm {
    /// sets bits `0` to `3`
    fn encode(&self) -> u32 {
        self.0 as u32
    }
}

impl Encode for Rs {
    /// sets bits `8` to `11`
    fn encode(&self) -> u32 {
        (self.0 as u32) << 8
    }
}

impl Encode for ShifterOperandCode {
    /// sets bits `0` to `11`
    fn encode(&self) -> u32 {
        match self {
            ShifterOperandCode::Immediate(imm) => imm.encode(),
            ShifterOperandCode::ImmediateShift(shift) => shift.encode(),
            ShifterOperandCode::RegisterShift(shift) => shift.encode(),
        }
    }
}

impl Encode for ShiftedRegister<Imm<5>> {
    /// sets bits `0` to `11`
    fn encode(&self) -> u32 {
        self.amount.encode() | self.kind.encode() | self.base.encode()
    }
}

impl Encode for RotatedImm8 {
    /// sets bits `0` to `12`
    fn encode(&self) -> u32 {
        ((self.rotate() as u32) << 8) | (self.value() as u32)
    }
}

impl Encode for Shift {
    /// sets bits `5` to `6`
    fn encode(&self) -> u32 {
        (*self as u32) << 5
    }
}

impl Encode for ShiftedRegister<Rs> {
    /// sets bits `0` to `11`
    fn encode(&self) -> u32 {
        self.amount.encode() | self.kind.encode() | self.base.encode()
    }
}

impl Encode for LoadStoreAddressCode<Imm<12>, Imm<5>> {
    /// sets bits `0` to `15`
    fn encode(&self) -> u32 {
        self.base.encode() | self.offset.encode()
    }
}

impl Encode for AddressingOffset<Imm<12>, Imm<5>> {
    /// sets bits `0` to `11`
    fn encode(&self) -> u32 {
        self.mode.encode() | self.sign.encode() | self.value.encode()
    }
}

impl Encode for OffsetMode {
    /// sets bits `24` and `21`
    fn encode(&self) -> u32 {
        let (p, w) = match self {
            OffsetMode::Offset => (1, 0),
            OffsetMode::PreIndexed => (1, 1),
            OffsetMode::PostIndexed => (0, 0),
        };

        p << 24 | w << 21
    }
}

impl Encode for Sign {
    /// sets bit `23`
    fn encode(&self) -> u32 {
        match self {
            Sign::Positive => 1 << 23,
            Sign::Negative => 0,
        }
    }
}

impl Encode for AddressingOffsetValue<Imm<12>, Imm<5>> {
    /// sets bits `0` to `11`
    fn encode(&self) -> u32 {
        match self {
            AddressingOffsetValue::Immediate(imm) => imm.encode(),
            AddressingOffsetValue::Register(register) => register.encode(),
            AddressingOffsetValue::ScaledRegister(shifted_register) => shifted_register.encode(),
        }
    }
}

impl Encode for MultipleAddressingMode {
    /// sets bits `20`, `23` and `24`
    fn encode(&self) -> u32 {
        let (l, p, u) = match self {
            MultipleAddressingMode::DecrementAfter => (1, 0, 0),
            MultipleAddressingMode::IncrementAfter => (1, 0, 1),
            MultipleAddressingMode::DecrementBefore => (1, 1, 0),
            MultipleAddressingMode::IncrementBefore => (1, 1, 1),
        };

        p << 24 | u << 23 | l << 20
    }
}

impl Encode for WriteBack {
    /// sets bit `21`
    fn encode(&self) -> u32 {
        match self {
            WriteBack::WriteBack => 1 << 21,
            WriteBack::NoWriteBack => 0,
        }
    }
}

impl Encode for RegisterList {
    /// sets bits `0` to `15`
    fn encode(&self) -> u32 {
        self.registers
            .iter()
            .enumerate()
            .fold(0, |mask, (i, value)| {
                mask | if *value { 1 } else { 0 } << i as u32
            })
    }
}
