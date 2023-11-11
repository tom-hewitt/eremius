use crate::{
    decoder::Bits,
    ir::{Condition, Imm, Rd, Rm, Rn, ShiftedRegister, ShifterOperandCode, SignedImm},
    parser::{
        AddressingOffset, AddressingOffsetValue, BranchKind, CalculationKind, DataProcessingKind,
        InstructionKind, LoadStoreAddressCode, LoadStoreKind, LoadStoreQuantity, OffsetMode,
        SetFlags, Shift, Sign,
    },
};

#[test]
fn test_branch_decode() {
    let instruction = InstructionKind::Branch {
        condition: Condition::AL,
        kind: BranchKind::Branch,
        target: SignedImm::new(-10),
    };

    assert_eq!(
        instruction,
        InstructionKind::decode(&Bits(0b11101010111111111111111111110110)).unwrap()
    )
}

#[test]
fn test_data_processing_decode() {
    let instruction = InstructionKind::DataProcessing {
        condition: Condition::AL,
        kind: DataProcessingKind::Calculation {
            kind: CalculationKind::ADD,
            set_flags: SetFlags::DontSet,
            destination: Rd(0),
            source: Rn(1),
            shifter: ShifterOperandCode::ImmediateShift(ShiftedRegister {
                kind: Shift::LogicalShiftLeft,
                amount: Imm::new(0),
                base: Rm(2),
            }),
        },
    };

    assert_eq!(
        instruction,
        InstructionKind::decode(&Bits(0b11100000100000010000000000000010)).unwrap()
    )
}

#[test]
fn test_load_store_decode() {
    let instruction = InstructionKind::LoadStore {
        condition: Condition::AL,
        kind: LoadStoreKind::Load,
        quantity: LoadStoreQuantity::Word,
        destination: Rd(0),
        address: LoadStoreAddressCode {
            base: Rn(1),
            offset: AddressingOffset {
                sign: Sign::Negative,
                mode: OffsetMode::PreIndexed,
                value: AddressingOffsetValue::Immediate(Imm::new(10)),
            },
        },
    };

    assert_eq!(
        instruction,
        InstructionKind::decode(&Bits(0b11100101001100010000000000001010)).unwrap()
    )
}
