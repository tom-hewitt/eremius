use std::collections::HashMap;

use crate::ir::{
    Condition, InstructionKind, LoadStoreAddressCode, Rd, Rm, Rn, Rs, ShiftedRegister,
};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Symbol(pub String);

pub type StatementInstructionKind =
    InstructionKind<Symbol, LoadStoreAddress, ShifterOperandExpression, Expression>;

#[derive(Debug, PartialEq)]
pub enum Statement {
    Instruction { kind: StatementInstructionKind },
    PseudoInstruction { kind: PseudoInstructionKind },
    Directive { kind: DirectiveKind },
}

#[derive(Debug, PartialEq)]
pub enum PseudoInstructionKind {
    LoadRegisterConstant {
        condition: Condition,
        destination: Rd,
        value: Expression,
    },

    AddressRegister {
        condition: Condition,
        long: bool,
        destination: Register,
        label: Symbol,
    },
}

#[derive(Debug, PartialEq)]
pub enum DirectiveKind {
    Definition { kind: DefinitionKind },
    Align,
    Origin { address: Expression },
    EntryPoint,
    Constant { value: Expression },
}

#[derive(Debug, PartialEq)]
pub enum LoadStoreAddress {
    Expression(Expression),
    AddressingMode(LoadStoreAddressCode<Expression, Expression>),
}

#[derive(Debug, PartialEq)]
pub enum DefinitionKind {
    Space { size: usize, fill: Option<u8> },
    Bytes { bytes: Vec<BytesDefinition> },
    Words { words: Vec<u32> },
}

#[derive(Debug, PartialEq)]
pub enum BytesDefinition {
    Byte(u8),
    String(String),
}

impl IntoIterator for BytesDefinition {
    type Item = u8;
    type IntoIter = BytesDefinitionIter;

    fn into_iter(self) -> Self::IntoIter {
        BytesDefinitionIter {
            bytes: self,
            index: 0,
        }
    }
}

// zero alloc iterator for byte definitions
pub struct BytesDefinitionIter {
    bytes: BytesDefinition,
    index: usize,
}

impl Iterator for BytesDefinitionIter {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        match &self.bytes {
            BytesDefinition::Byte(byte) => {
                if self.index == 0 {
                    Some(*byte)
                } else {
                    None
                }
            }
            BytesDefinition::String(string) => {
                if self.index < string.as_bytes().len() {
                    Some(string.as_bytes()[self.index])
                } else {
                    None
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Number { base: u32, n: u32 },
    Character(char),
    String(String),
    Boolean(bool),
    Symbol(Symbol),
    Diadic(Box<Expression>, DiadicOperator, Box<Expression>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum DiadicOperator {
    Plus,
    Minus,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Register(pub u8);

impl Into<Rd> for Register {
    fn into(self) -> Rd {
        Rd(self.0)
    }
}

impl Into<Rn> for Register {
    fn into(self) -> Rn {
        Rn(self.0)
    }
}

impl Into<Rm> for Register {
    fn into(self) -> Rm {
        Rm(self.0)
    }
}

impl Into<Rs> for Register {
    fn into(self) -> Rs {
        Rs(self.0)
    }
}

#[derive(Debug, PartialEq)]
pub enum ShifterOperandExpression {
    Immediate(Expression),
    Register(Register),
    ShiftedRegister(ShiftedRegister<ShifterOperandShiftAmount, Register>),
    RotateRightWithExtend(Register),
}

#[derive(Debug, PartialEq)]
pub enum ShifterOperandShiftAmount {
    Immediate(Expression),
    Register(Register),
}
