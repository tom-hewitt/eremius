// NOTE: for each statement, lets just return an iterator over a Vec, because there could be an indefinite number of bytes in a data def
// use a smallvec for optimisations!!

use std::collections::HashMap;

use smallvec::{smallvec, SmallVec};

use crate::{
    assembler::AssemblyError,
    ir::{
        AddressingOffset, Imm, Rm, Rn, RotatedImm8, ShiftedRegister, ShifterOperandCode, SignedImm,
        UnencodableValueError,
    },
    parser::{
        AddressingOffsetValue, CalculationKind, DataProcessingKind, DiadicOperator, Expression,
        InstructionKind, LoadStoreAddress, LoadStoreAddressCode, MoveKind, OffsetMode,
        PseudoInstructionKind, SetFlags, Shift, ShifterOperandExpression,
        ShifterOperandShiftAmount, Sign, StatementInstructionKind, Symbol,
    },
    preprocessor::{PreProcessResult, PreProcessedStatement},
};

#[derive(Debug)]
pub enum ResolveError {
    SymbolNotFound(SymbolNotFoundError),
    UnencodableSignedValue(UnencodableValueError<i32>),
    UnencodableValue(UnencodableValueError<u32>),
}

impl From<SymbolNotFoundError> for ResolveError {
    fn from(value: SymbolNotFoundError) -> Self {
        Self::SymbolNotFound(value)
    }
}

impl From<UnencodableValueError<i32>> for ResolveError {
    fn from(value: UnencodableValueError<i32>) -> Self {
        Self::UnencodableSignedValue(value)
    }
}

impl From<UnencodableValueError<u32>> for ResolveError {
    fn from(value: UnencodableValueError<u32>) -> Self {
        Self::UnencodableValue(value)
    }
}

pub enum ResolvedStatement {
    // currently 2 is the upper limit for the number of instructions in a statement, so this should never allocate
    Instructions(SmallVec<[InstructionKind; 2]>),
    // most data statements are only a word, but this allows for more (at the cost of an allocation)
    // won't allocate in the average case
    Data(SmallVec<[u8; 4]>),
}

#[derive(Debug)]
pub struct SymbolNotFoundError;

#[derive(Debug)]
pub struct SymbolTable<E> {
    table: HashMap<Symbol, E>,
}

impl<E> SymbolTable<E> {
    pub fn new() -> Self {
        Self {
            table: HashMap::new(),
        }
    }

    pub fn insert(&mut self, symbol: Symbol, value: E) {
        self.table.insert(symbol, value);
    }

    pub fn get(&self, symbol: &Symbol) -> Option<&E> {
        self.table.get(symbol)
    }
}

impl SymbolTable<Expression> {
    pub fn resolve(&self) -> Result<SymbolTable<u32>, ResolveError> {
        let mut resolved_table = SymbolTable::new();

        for (symbol, expression) in self.table.iter() {
            let value = expression.resolve_and_insert(&self, &mut resolved_table)?;
            resolved_table.insert(symbol.clone(), value);
        }

        Ok(resolved_table)
    }
}

impl PreProcessedStatement {
    pub fn resolve(
        self,
        symbol_table: &SymbolTable<u32>,
        address: usize,
    ) -> Result<ResolvedStatement, ResolveError> {
        match self {
            PreProcessedStatement::Instruction { kind } => {
                Ok(ResolvedStatement::Instructions(smallvec![
                    kind.resolve(symbol_table, address)?
                ]))
            }
            PreProcessedStatement::PseudoInstruction { kind } => Ok(
                ResolvedStatement::Instructions(kind.resolve(&symbol_table, address)?),
            ),
            PreProcessedStatement::Data(data) => Ok(ResolvedStatement::Data(data)),
        }
    }
}

impl StatementInstructionKind {
    pub fn resolve(
        self,
        symbol_table: &SymbolTable<u32>,
        current_address: usize,
    ) -> Result<InstructionKind, ResolveError> {
        match self {
            InstructionKind::Branch {
                condition,
                kind,
                target,
            } => {
                let target_address = target.resolve(symbol_table)?;

                // the target address is word-aligned, so it is shifted 2 bits to the right for a larger range of values wihout losing precision
                let target =
                    SignedImm::try_from((target_address as i32 - current_address as i32) >> 2)?;

                Ok(InstructionKind::Branch {
                    condition,
                    kind,
                    target,
                })
            }

            InstructionKind::DataProcessing { condition, kind } => {
                Ok(InstructionKind::DataProcessing {
                    condition,
                    kind: kind.resolve(symbol_table, current_address)?,
                })
            }

            InstructionKind::LoadStore {
                condition,
                kind,
                quantity,
                destination,
                address,
            } => Ok(InstructionKind::LoadStore {
                condition,
                kind,
                quantity,
                destination,
                address: address.resolve(symbol_table, current_address)?,
            }),

            InstructionKind::LoadStoreMultiple {
                condition,
                kind,
                mode,
                base,
                write_back,
                register_list,
            } => Ok(InstructionKind::LoadStoreMultiple {
                condition,
                kind,
                mode,
                base,
                write_back,
                register_list,
            }),

            InstructionKind::SuperVisorCall {
                condition,
                immediate,
            } => Ok(InstructionKind::SuperVisorCall {
                condition,
                immediate: Imm::try_from(immediate.resolve(symbol_table)?)?,
            }),
        }
    }
}

impl DataProcessingKind<ShifterOperandExpression> {
    fn resolve(
        self,
        symbol_table: &SymbolTable<u32>,
        address: usize,
    ) -> Result<DataProcessingKind<ShifterOperandCode<RotatedImm8, Imm<5>>>, ResolveError> {
        Ok(match self {
            DataProcessingKind::Move {
                kind,
                set_flags,
                destination,
                shifter,
            } => DataProcessingKind::Move {
                kind,
                set_flags,
                destination,
                shifter: shifter.resolve(symbol_table)?,
            },

            DataProcessingKind::Comparison {
                kind,
                source,
                shifter,
            } => DataProcessingKind::Comparison {
                kind,
                source,
                shifter: shifter.resolve(symbol_table)?,
            },

            DataProcessingKind::Calculation {
                kind,
                set_flags,
                destination,
                source,
                shifter,
            } => DataProcessingKind::Calculation {
                kind,
                set_flags,
                destination,
                source,
                shifter: shifter.resolve(symbol_table)?,
            },
        })
    }
}

impl ShifterOperandExpression {
    fn resolve(
        self,
        symbol_table: &SymbolTable<u32>,
    ) -> Result<ShifterOperandCode<RotatedImm8, Imm<5>>, ResolveError> {
        match self {
            ShifterOperandExpression::Immediate(immediate) => Ok(ShifterOperandCode::Immediate(
                RotatedImm8::try_from(immediate.resolve(symbol_table)?)?,
            )),

            ShifterOperandExpression::Register(register) => {
                Ok(ShifterOperandCode::ImmediateShift(ShiftedRegister {
                    kind: Shift::LogicalShiftLeft,
                    amount: Imm::try_from(0).unwrap(),
                    base: register.into(),
                }))
            }

            ShifterOperandExpression::ShiftedRegister(ShiftedRegister { kind, amount, base }) => {
                match amount {
                    ShifterOperandShiftAmount::Immediate(immediate) => {
                        Ok(ShifterOperandCode::ImmediateShift(ShiftedRegister {
                            kind,
                            amount: Imm::try_from(immediate.resolve(symbol_table)?)?,
                            base: base.into(),
                        }))
                    }

                    ShifterOperandShiftAmount::Register(register) => {
                        Ok(ShifterOperandCode::RegisterShift(ShiftedRegister {
                            kind,
                            amount: register.into(),
                            base: base.into(),
                        }))
                    }
                }
            }

            ShifterOperandExpression::RotateRightWithExtend(register) => {
                Ok(ShifterOperandCode::ImmediateShift(ShiftedRegister {
                    kind: Shift::RotateRight,
                    amount: Imm::try_from(0).unwrap(),
                    base: register.into(),
                }))
            }
        }
    }
}

impl LoadStoreAddress {
    fn resolve(
        self,
        symbol_table: &SymbolTable<u32>,
        current_address: usize,
    ) -> Result<LoadStoreAddressCode<Imm<12>, Imm<5>>, ResolveError> {
        match self {
            LoadStoreAddress::Expression(expression) => {
                let target_address = expression.resolve(symbol_table)?;

                let pc_offset = target_address as i32 - current_address as i32;
                let sign = if pc_offset >= 0 {
                    Sign::Positive
                } else {
                    Sign::Negative
                };
                let value = pc_offset.abs() as u32;

                Ok(LoadStoreAddressCode {
                    base: Rn(15),
                    offset: AddressingOffset {
                        sign,
                        value: AddressingOffsetValue::Immediate(Imm::try_from(value)?),
                        mode: OffsetMode::Offset,
                    },
                })
            }

            LoadStoreAddress::AddressingMode(LoadStoreAddressCode { base, offset }) => {
                Ok(LoadStoreAddressCode {
                    base,
                    offset: offset.resolve(symbol_table)?,
                })
            }
        }
    }
}

impl AddressingOffset<Expression, Expression> {
    fn resolve(
        self,
        symbol_table: &SymbolTable<u32>,
    ) -> Result<AddressingOffset<Imm<12>, Imm<5>>, ResolveError> {
        Ok(AddressingOffset {
            sign: self.sign,
            value: self.value.resolve(symbol_table)?,
            mode: self.mode,
        })
    }
}

impl AddressingOffsetValue<Expression, Expression> {
    fn resolve(
        self,
        symbol_table: &SymbolTable<u32>,
    ) -> Result<AddressingOffsetValue<Imm<12>, Imm<5>>, ResolveError> {
        match self {
            AddressingOffsetValue::Immediate(immediate) => Ok(AddressingOffsetValue::Immediate(
                Imm::try_from(immediate.resolve(symbol_table)?)?,
            )),

            AddressingOffsetValue::Register(register) => {
                Ok(AddressingOffsetValue::Register(register))
            }

            AddressingOffsetValue::ScaledRegister(ShiftedRegister { kind, amount, base }) => {
                Ok(AddressingOffsetValue::ScaledRegister(ShiftedRegister {
                    kind,
                    amount: Imm::try_from(amount.resolve(symbol_table)?)?,
                    base,
                }))
            }
        }
    }
}

impl PseudoInstructionKind {
    fn resolve(
        self,
        symbol_table: &SymbolTable<u32>,
        current_address: usize,
    ) -> Result<SmallVec<[InstructionKind; 2]>, ResolveError> {
        match self {
            PseudoInstructionKind::LoadRegisterConstant {
                condition,
                destination,
                value,
            } => Ok(smallvec![InstructionKind::DataProcessing {
                condition,
                kind: DataProcessingKind::Move {
                    kind: MoveKind::Move,
                    set_flags: SetFlags::DontSet,
                    destination,
                    shifter: ShifterOperandCode::Immediate(RotatedImm8::try_from(
                        value.resolve(symbol_table)?,
                    )?),
                },
            }]),

            PseudoInstructionKind::AddressRegister {
                condition,
                long,
                destination,
                label,
            } => {
                let address = label.resolve(symbol_table)?;

                let offset = address as i32 - current_address as i32;

                let value = offset.abs() as u32;

                if long {
                    let (immediate, remainder) = RotatedImm8::nearest_with_remainder(value);

                    // if the remainder can't be encoded, then the value in unencodable
                    let remainder_immediate = RotatedImm8::try_from(remainder.abs() as u32)
                        .map_err(|_| UnencodableValueError { value })?;

                    Ok(smallvec![
                        // the first instruction gets us as near as possible
                        InstructionKind::DataProcessing {
                            condition,
                            kind: DataProcessingKind::Calculation {
                                kind: if offset >= 0 {
                                    CalculationKind::ADD
                                } else {
                                    CalculationKind::SUB
                                },
                                set_flags: SetFlags::DontSet,
                                destination: destination.into(),
                                source: Rn(15),
                                shifter: ShifterOperandCode::Immediate(immediate),
                            },
                        },
                        // the second instruction
                        if remainder >= 0 {
                            InstructionKind::DataProcessing {
                                condition,
                                kind: DataProcessingKind::Calculation {
                                    // the same as the first instruction
                                    kind: if offset >= 0 {
                                        CalculationKind::ADD
                                    } else {
                                        CalculationKind::SUB
                                    },
                                    set_flags: SetFlags::DontSet,
                                    destination: destination.into(),
                                    source: Rn(15),
                                    shifter: ShifterOperandCode::Immediate(remainder_immediate),
                                },
                            }
                        } else {
                            InstructionKind::DataProcessing {
                                condition,
                                kind: DataProcessingKind::Calculation {
                                    // the opposite to the first instruction
                                    kind: if offset >= 0 {
                                        CalculationKind::SUB
                                    } else {
                                        CalculationKind::ADD
                                    },
                                    set_flags: SetFlags::DontSet,
                                    destination: destination.into(),
                                    source: Rn(15),
                                    shifter: ShifterOperandCode::Immediate(remainder_immediate),
                                },
                            }
                        },
                    ])
                } else {
                    let immediate = RotatedImm8::try_from(value)?;

                    Ok(smallvec![InstructionKind::DataProcessing {
                        condition,
                        kind: DataProcessingKind::Calculation {
                            kind: if offset >= 0 {
                                CalculationKind::ADD
                            } else {
                                CalculationKind::SUB
                            },
                            set_flags: SetFlags::DontSet,
                            destination: destination.into(),
                            source: Rn(15),
                            shifter: ShifterOperandCode::Immediate(immediate),
                        },
                    }])
                }
            }
        }
    }
}

impl Expression {
    pub fn resolve(self, symbol_table: &SymbolTable<u32>) -> Result<u32, ResolveError> {
        match self {
            Expression::Number { n, base: _ } => Ok(n),
            Expression::Character(c) => Ok(c as u32),
            Expression::String(s) => Ok(s.as_bytes()[0] as u32),
            Expression::Boolean(b) => Ok(b as u32),
            Expression::Symbol(symbol) => Ok(symbol.resolve(symbol_table)?),
            Expression::Diadic(lhs, operator, rhs) => {
                let lhs = lhs.resolve(symbol_table)?;
                let rhs = rhs.resolve(symbol_table)?;

                match operator {
                    DiadicOperator::Plus => Ok(lhs + rhs),
                    DiadicOperator::Minus => Ok(lhs - rhs),
                }
            }
        }
    }

    pub fn resolve_and_insert(
        &self,
        unresolved_table: &SymbolTable<Expression>,
        resolved_table: &mut SymbolTable<u32>,
    ) -> Result<u32, ResolveError> {
        match self {
            Expression::Number { n, base: _ } => Ok(*n),
            Expression::Character(c) => Ok(*c as u32),
            Expression::String(s) => Ok(s.as_bytes()[0] as u32),
            Expression::Boolean(b) => Ok(*b as u32),
            Expression::Symbol(symbol) => {
                Ok(symbol.resolve_and_insert(unresolved_table, resolved_table)?)
            }
            Expression::Diadic(lhs, operator, rhs) => {
                let lhs = lhs.resolve_and_insert(unresolved_table, resolved_table)?;
                let rhs = rhs.resolve_and_insert(unresolved_table, resolved_table)?;

                match operator {
                    DiadicOperator::Plus => Ok(lhs + rhs),
                    DiadicOperator::Minus => Ok(lhs - rhs),
                }
            }
        }
    }

    /// only resolves labels before the expression
    pub fn backwards_resolve(
        &self,
        symbol_table: &SymbolTable<Expression>,
    ) -> Result<u32, ResolveError> {
        match self {
            Expression::Number { n, base: _ } => Ok(*n),
            Expression::Character(c) => Ok(*c as u32),
            Expression::String(s) => Ok(s.as_bytes()[0] as u32),
            Expression::Boolean(b) => Ok(*b as u32),
            Expression::Symbol(symbol) => Ok(symbol.backwards_resolve(symbol_table)?),
            Expression::Diadic(lhs, operator, rhs) => {
                let lhs = lhs.backwards_resolve(symbol_table)?;
                let rhs = rhs.backwards_resolve(symbol_table)?;

                match operator {
                    DiadicOperator::Plus => Ok(lhs + rhs),
                    DiadicOperator::Minus => Ok(lhs - rhs),
                }
            }
        }
    }
}

impl Symbol {
    pub fn resolve(self, symbol_table: &SymbolTable<u32>) -> Result<u32, ResolveError> {
        match symbol_table.get(&self) {
            Some(value) => Ok(*value),
            None => Err(SymbolNotFoundError)?,
        }
    }

    fn resolve_and_insert(
        &self,
        unresolved_table: &SymbolTable<Expression>,
        resolved_table: &mut SymbolTable<u32>,
    ) -> Result<u32, ResolveError> {
        match unresolved_table.get(&self) {
            Some(expression) => {
                let value = expression.resolve_and_insert(unresolved_table, resolved_table)?;

                resolved_table.insert(self.clone(), value);

                Ok(value)
            }
            None => Err(SymbolNotFoundError)?,
        }
    }

    /// only resolves labels before the symbol
    pub fn backwards_resolve(
        &self,
        symbol_table: &SymbolTable<Expression>,
    ) -> Result<u32, ResolveError> {
        match symbol_table.get(&self) {
            Some(expression) => Ok(expression.backwards_resolve(symbol_table)?),
            None => Err(SymbolNotFoundError)?,
        }
    }
}
