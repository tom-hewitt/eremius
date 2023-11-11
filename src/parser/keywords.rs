// note: we use phf instead of match statements because it allows for case-insensitivity and constant lookup time
use phf::phf_map;
use unicase::UniCase;

use crate::ir::{Condition, MultipleAddressingMode};

use super::{statements::Register, Shift};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mnemonic {
    // Branch Instructions
    B {
        l: bool,
        condition: Condition,
    },

    // Data Processing Instructions
    ADD {
        condition: Condition,
        s: bool,
    },
    SUB {
        condition: Condition,
        s: bool,
    },
    CMP {
        condition: Condition,
    },
    MOV {
        condition: Condition,
        s: bool,
    },

    // Data Transfer Instructions
    LDR {
        condition: Condition,
    },
    STR {
        condition: Condition,
    },
    LDRB {
        condition: Condition,
    },
    STRB {
        condition: Condition,
    },
    LDM {
        condition: Condition,
        mode: MultipleAddressingMode,
    },
    STM {
        condition: Condition,
        mode: MultipleAddressingMode,
    },

    // System Calls
    SVC {
        condition: Condition,
    },

    // Pseudo Instructions
    ADR {
        condition: Condition,
        l: bool,
    },

    // Assembler Directives
    DEFW,
    DEFB,
    DEFS,
    ALIGN,
    ORIGIN,
    ENTRY,
    EQU,
}

// mnemonic and condition perfect hash map
// generated in build.rs
include!(concat!(env!("OUT_DIR"), "/codegen.rs"));

// register lookup table
pub static REGISTERS: phf::Map<UniCase<&'static str>, Register> = phf_map! {
    UniCase::ascii("R0") => Register(0),
    UniCase::ascii("R1") => Register(1),
    UniCase::ascii("R2") => Register(2),
    UniCase::ascii("R3") => Register(3),
    UniCase::ascii("R4") => Register(4),
    UniCase::ascii("R5") => Register(5),
    UniCase::ascii("R6") => Register(6),
    UniCase::ascii("R7") => Register(7),
    UniCase::ascii("R8") => Register(8),
    UniCase::ascii("R9") => Register(9),
    UniCase::ascii("R10") => Register(10),
    UniCase::ascii("R11") => Register(11),
    UniCase::ascii("R12") => Register(12),
    // stack pointer
    UniCase::ascii("SP") => Register(13),
    UniCase::ascii("R13") => Register(13),
    // link register
    UniCase::ascii("LR") => Register(14),
    UniCase::ascii("R14") => Register(14),
    // program counter
    UniCase::ascii("PC") => Register(15),
    UniCase::ascii("R15") => Register(15),
};

/// included RRX
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShiftName {
    LogicalShiftLeft,
    LogicalShiftRight,
    ArithmeticShiftRight,
    RotateRight,
    RotateRightExtended,
}

impl TryInto<Shift> for ShiftName {
    type Error = ();

    fn try_into(self) -> Result<Shift, Self::Error> {
        match self {
            ShiftName::LogicalShiftLeft => Ok(Shift::LogicalShiftLeft),
            ShiftName::LogicalShiftRight => Ok(Shift::LogicalShiftRight),
            ShiftName::ArithmeticShiftRight => Ok(Shift::ArithmeticShiftRight),
            ShiftName::RotateRight => Ok(Shift::RotateRight),
            ShiftName::RotateRightExtended => Err(()),
        }
    }
}

// shift lookup table
pub static SHIFT_NAMES: phf::Map<UniCase<&'static str>, ShiftName> = phf_map! {
    UniCase::ascii("LSL") => ShiftName::LogicalShiftLeft,
    UniCase::ascii("LSR") => ShiftName::LogicalShiftRight,
    UniCase::ascii("ASR") => ShiftName::ArithmeticShiftRight,
    UniCase::ascii("ROR") => ShiftName::RotateRight,
    UniCase::ascii("RRX") => ShiftName::RotateRightExtended,
};

pub static SHIFT_KINDS: phf::Map<UniCase<&'static str>, Shift> = phf_map! {
    UniCase::ascii("LSL") => Shift::LogicalShiftLeft,
    UniCase::ascii("LSR") => Shift::LogicalShiftRight,
    UniCase::ascii("ASR") => Shift::ArithmeticShiftRight,
    UniCase::ascii("ROR") => Shift::RotateRight,
};
