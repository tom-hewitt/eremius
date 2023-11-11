use crate::{
    ir::{
        Condition, Imm, MultipleAddressingMode, Rd, Rm, Rn, RotatedImm8, Rs, ShiftedRegister,
        ShifterOperandCode, SignedImm,
    },
    parser::{
        AddressingOffset, AddressingOffsetValue, BranchKind, CalculationKind, DataProcessingKind,
        InstructionKind, LoadStoreAddressCode, LoadStoreKind, LoadStoreQuantity, OffsetMode,
        RegisterList, SetFlags, Shift, Sign, WriteBack,
    },
};

use std::ops::{Deref, Index, RangeInclusive};

mod tests;

pub struct Bits(pub u32);

impl Index<usize> for Bits {
    type Output = u32;

    fn index(&self, index: usize) -> &Self::Output {
        match (self.0 >> index) & 1 {
            0b0 => &0b0,
            0b1 => &0b1,
            _ => unreachable!(),
        }
    }
}

impl Bits {
    pub fn range(&self, index: RangeInclusive<usize>) -> u32 {
        (self.0 >> index.start()) & ((1 << (index.end() - index.start() + 1)) - 1)
    }
}

impl Deref for Bits {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub struct InvalidInstructionError;

impl InstructionKind {
    pub fn decode(bits: &Bits) -> Result<Self, InvalidInstructionError> {
        match bits.range(25..=27) {
            // data processing
            0b000 | 0b001 => Ok(InstructionKind::DataProcessing {
                condition: Condition::decode(bits)?,
                kind: DataProcessingKind::decode(bits)?,
            }),

            // load store
            0b010 | 0b011 => Ok(InstructionKind::LoadStore {
                condition: Condition::decode(bits)?,
                kind: LoadStoreKind::decode(bits),
                quantity: LoadStoreQuantity::decode(bits),
                destination: Rd::decode(bits),
                address: LoadStoreAddressCode::decode(bits)?,
            }),

            // load store multiple
            0b100 => Ok(InstructionKind::LoadStoreMultiple {
                condition: Condition::decode(bits)?,
                kind: LoadStoreKind::decode(bits),
                mode: MultipleAddressingMode::decode(bits)?,
                base: Rn::decode(bits),
                write_back: WriteBack::decode(bits),
                register_list: RegisterList::decode(bits),
            }),

            // branch
            0b101 => Ok(InstructionKind::Branch {
                condition: Condition::decode(bits)?,
                kind: BranchKind::decode(bits),
                target: SignedImm::decode(bits),
            }),

            // supervisor call
            0b111 => Ok(InstructionKind::SuperVisorCall {
                condition: Condition::decode(bits)?,
                immediate: Imm::decode(bits),
            }),

            _ => todo!(),
        }
    }
}

impl Condition {
    fn decode(bits: &Bits) -> Result<Self, InvalidInstructionError> {
        match bits.range(28..=31) {
            0b0000 => Ok(Self::EQ),
            0b0001 => Ok(Self::NE),
            0b0010 => Ok(Self::CS),
            0b0011 => Ok(Self::CC),
            0b0100 => Ok(Self::MI),
            0b0110 => Ok(Self::VS),
            0b0111 => Ok(Self::VC),
            0b1000 => Ok(Self::HI),
            0b1001 => Ok(Self::LS),
            0b1010 => Ok(Self::GE),
            0b1011 => Ok(Self::LT),
            0b1100 => Ok(Self::GT),
            0b1101 => Ok(Self::LE),
            0b1110 => Ok(Self::AL),
            _ => Err(InvalidInstructionError),
        }
    }
}

impl DataProcessingKind {
    fn decode(bits: &Bits) -> Result<Self, InvalidInstructionError> {
        match bits.range(21..=24) {
            // Data Processing
            0b0100 | 0b0010 => Ok(DataProcessingKind::Calculation {
                kind: CalculationKind::decode(bits)?,
                set_flags: SetFlags::decode(bits),
                destination: Rd::decode(bits),
                source: Rn::decode(bits),
                shifter: ShifterOperandCode::decode(bits),
            }),

            //
            _ => Err(InvalidInstructionError),
        }
    }
}

impl CalculationKind {
    fn decode(bits: &Bits) -> Result<Self, InvalidInstructionError> {
        match bits.range(21..=24) {
            0b0100 => Ok(Self::ADD),
            0b0010 => Ok(Self::SUB),
            _ => Err(InvalidInstructionError),
        }
    }
}

impl SetFlags {
    fn decode(bits: &Bits) -> Self {
        match bits[20] {
            0b0 => Self::DontSet,
            0b1 => Self::Set,
            _ => unreachable!(),
        }
    }
}

impl Rn {
    fn decode(bits: &Bits) -> Self {
        Self(bits.range(16..=19) as u8)
    }
}

impl Rd {
    fn decode(bits: &Bits) -> Self {
        Self(bits.range(12..=15) as u8)
    }
}

impl Rs {
    fn decode(bits: &Bits) -> Self {
        Self(bits.range(8..=11) as u8)
    }
}

impl Rm {
    fn decode(bits: &Bits) -> Self {
        Self(bits.range(0..=3) as u8)
    }
}

impl ShifterOperandCode<RotatedImm8, Imm<5>> {
    fn decode(bits: &Bits) -> Self {
        match bits[25] {
            0b0 => match bits[4] {
                0b0 => Self::ImmediateShift(ShiftedRegister::<Imm<5>, Rm>::decode(bits)),
                0b1 => Self::RegisterShift(ShiftedRegister::<Rs, Rm>::decode(bits)),
                _ => unreachable!(),
            },
            0b1 => Self::Immediate(RotatedImm8::decode(bits)),
            _ => unreachable!(),
        }
    }
}

impl ShiftedRegister<Imm<5>, Rm> {
    fn decode(bits: &Bits) -> Self {
        Self {
            kind: Shift::decode(bits),
            amount: Imm::try_from(bits.range(7..=11)).unwrap(),
            base: Rm::decode(bits),
        }
    }
}

impl ShiftedRegister<Rs, Rm> {
    fn decode(bits: &Bits) -> Self {
        Self {
            kind: Shift::decode(bits),
            amount: Rs::decode(bits),
            base: Rm::decode(bits),
        }
    }
}

impl Shift {
    fn decode(bits: &Bits) -> Self {
        match bits.range(5..=6) {
            0b00 => Self::LogicalShiftLeft,
            0b01 => Self::LogicalShiftRight,
            0b10 => Self::ArithmeticShiftRight,
            0b11 => Self::RotateRight,
            _ => unreachable!(),
        }
    }
}

impl RotatedImm8 {
    fn decode(bits: &Bits) -> Self {
        Self::new(bits.range(0..=7) as u8, bits.range(8..=11) as u8)
    }
}

impl LoadStoreKind {
    fn decode(bits: &Bits) -> Self {
        match bits[20] {
            0b1 => Self::Load,
            0b0 => Self::Store,
            _ => unreachable!(),
        }
    }
}

impl LoadStoreQuantity {
    fn decode(bits: &Bits) -> Self {
        match bits[22] {
            0b1 => Self::Byte,
            0b0 => Self::Word,
            _ => unreachable!(),
        }
    }
}

impl LoadStoreAddressCode<Imm<12>, Imm<5>> {
    fn decode(bits: &Bits) -> Result<Self, InvalidInstructionError> {
        Ok(Self {
            base: Rn::decode(bits),
            offset: AddressingOffset::decode(bits)?,
        })
    }
}

impl AddressingOffset<Imm<12>, Imm<5>> {
    fn decode(bits: &Bits) -> Result<Self, InvalidInstructionError> {
        Ok(Self {
            sign: Sign::decode(bits),
            value: AddressingOffsetValue::decode(bits),
            mode: OffsetMode::decode(bits)?,
        })
    }
}

impl Sign {
    fn decode(bits: &Bits) -> Self {
        match bits[23] {
            0b1 => Self::Positive,
            0b0 => Self::Negative,
            _ => unreachable!(),
        }
    }
}

impl AddressingOffsetValue<Imm<12>, Imm<5>> {
    fn decode(bits: &Bits) -> Self {
        match bits[25] {
            0b0 => Self::Immediate(Imm::try_from(bits.range(0..=11)).unwrap()),
            0b1 => Self::Register(Rm::decode(bits)),
            _ => unreachable!(),
        }
    }
}

impl OffsetMode {
    fn decode(bits: &Bits) -> Result<Self, InvalidInstructionError> {
        match (bits[24], bits[21]) {
            (0b1, 0b0) => Ok(Self::Offset),
            (0b1, 0b1) => Ok(Self::PreIndexed),
            (0b0, 0b0) => Ok(Self::PostIndexed),
            _ => Err(InvalidInstructionError),
        }
    }
}

impl MultipleAddressingMode {
    fn decode(bits: &Bits) -> Result<Self, InvalidInstructionError> {
        let (p, u, l) = (bits[20], bits[23], bits[24]);

        match (l, p, u) {
            (1, 0, 0) => Ok(MultipleAddressingMode::DecrementAfter),
            (1, 0, 1) => Ok(MultipleAddressingMode::IncrementAfter),
            (1, 1, 0) => Ok(MultipleAddressingMode::DecrementBefore),
            (1, 1, 1) => Ok(MultipleAddressingMode::IncrementBefore),
            _ => Err(InvalidInstructionError),
        }
    }
}

impl WriteBack {
    fn decode(bits: &Bits) -> Self {
        match bits[21] {
            1 => WriteBack::WriteBack,
            _ => WriteBack::NoWriteBack,
        }
    }
}

impl RegisterList {
    fn decode(bits: &Bits) -> Self {
        let mut registers = [false; 16];

        for i in 0..16 {
            registers[i] = bits[i] == 1;
        }

        RegisterList { registers }
    }
}

impl BranchKind {
    fn decode(bits: &Bits) -> Self {
        match bits[24] {
            1 => BranchKind::BranchWithLink,
            _ => BranchKind::Branch,
        }
    }
}

impl SignedImm<24> {
    fn decode(bits: &Bits) -> Self {
        let extend_bits = if bits[23] == 1 {
            Bits(u32::MAX).range(24..=31) << 24
        } else {
            0
        };

        SignedImm::new((bits.range(0..=23) | extend_bits) as i32)
    }
}

impl Imm<24> {
    fn decode(bits: &Bits) -> Self {
        Imm::new(bits.range(0..=23))
    }
}
