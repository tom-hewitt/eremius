use std::{fmt::Display, ops::Range};

use crate::{ir::ShiftedRegister, lexer::Tokens};
pub use crate::{
    ir::{
        AddressingOffset, AddressingOffsetValue, BranchKind, CalculationKind, ComparisonKind,
        DataProcessingKind, InstructionKind, LoadStoreAddressCode, LoadStoreKind,
        LoadStoreQuantity, MoveKind, OffsetMode, RegisterList, SetFlags, Shift, Sign, WriteBack,
    },
    lexer::{Lexer, Token, TokenKind},
    parser::{
        keywords::{Mnemonic, ShiftName, MNEMONICS, REGISTERS, SHIFT_KINDS, SHIFT_NAMES},
        statements::*,
    },
};
use unicase::UniCase;

mod keywords;
mod statements;

#[cfg(test)]
mod tests;

#[derive(Debug, Default, PartialEq)]
pub struct Line {
    pub label: Option<String>,
    pub statement: Option<Statement>,
}

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    line_count: usize,
}

#[derive(Debug)]
pub struct LineError<'a> {
    token: Option<Token<'a>>,
    message: &'static str,
}

#[derive(Debug)]
pub struct ParseError {
    line_number: usize,
    line: String,
    /// The range of the token (or lack of token) causing the error.
    /// Note: not nexessarily contained within the line - there could be a missing character at the end of a line.
    bad_token_range: Range<usize>,
    message: &'static str,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let arrow_line: String = self
            .line
            .chars()
            .enumerate()
            .map(|(i, c)| match c {
                '\t' => '\t',
                _ if self.bad_token_range.contains(&i) => '^',
                _ => ' ',
            })
            .collect();

        write!(
            f,
            "\nError at line {}, token \"{}\": {}\n{}\n{}\n",
            self.line_number,
            &self.line[self.bad_token_range.clone()],
            self.message,
            self.line,
            arrow_line
        )
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = Result<Line, ParseError>;

    fn next(&mut self) -> Option<Result<Line, ParseError>> {
        let line_start_token = self.lexer.peek()?;

        let line_start = line_start_token.range.start;

        let token = self.lexer.next_ignore_whitespace()?;

        self.line_count += 1;

        match self.line(&token) {
            Ok(line) => Some(Ok(line)),

            // display error helper
            Err(LineError { token, message }) => {
                let line = match self.lexer.input[line_start..].split_once('\n') {
                    Some((line, _)) => line,
                    None => &self.lexer.input[line_start..],
                };

                let token_start = match &token {
                    Some(token) => token.range.start - line_start,
                    None => line.len(),
                };

                let token_length = match token {
                    Some(token) => token.range.len(),
                    None => 1,
                };

                let bad_token_range = token_start..(token_start + token_length);

                Some(Err(ParseError {
                    line_number: self.line_count,
                    line: line.to_owned(),
                    bad_token_range,
                    message,
                }))
            }
        }
    }
}

impl<'a> Lexer<'a> {
    pub fn parse(self) -> Parser<'a> {
        Parser {
            lexer: self,
            line_count: 0,
        }
    }
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Parser<'a> {
        Parser {
            lexer: Lexer::new(input),
            line_count: 0,
        }
    }

    fn line(&mut self, token: &Token<'a>) -> Result<Line, LineError<'a>> {
        match token.kind {
            TokenKind::Identifier => {
                let line =
                    match self.mnemonic(&token) {
                        // if the line starts with a valid mnemonic
                        Ok(mnemonic) => {
                            let statement = self.statement(mnemonic)?;

                            self.line_end()?;

                            Line {
                                label: None,
                                statement: Some(statement),
                            }
                        }

                        // otherwise, line must start with a label
                        Err(_) => Line {
                            label: Some(token.span.to_owned()),
                            statement: match self.lexer.next_ignore_whitespace() {
                                // line end - there is no statement
                                None
                                | Some(Token {
                                    kind: TokenKind::NewLine,
                                    ..
                                })
                                | Some(Token {
                                    kind: TokenKind::Comment,
                                    ..
                                }) => None,

                                // must be a valid mnemonic
                                Some(
                                    next @ Token {
                                        kind: TokenKind::Identifier,
                                        ..
                                    },
                                ) => {
                                    let mnemonic = self.mnemonic(&next)?;

                                    let statement = self.statement(mnemonic)?;
                                    self.line_end()?;

                                    Some(statement)
                                }

                                // no other tokens can are allowed to follow a label
                                token => return Err(LineError {
                                    token,
                                    message:
                                        "Expected a statement, comment, or newline after a label",
                                }),
                            },
                        },
                    };

                Ok(line)
            }

            // if the line only contains a comment then just return an empty line
            TokenKind::Comment => {
                self.line_end()?;
                Ok(Line::default())
            }

            // empty line
            TokenKind::NewLine => Ok(Line::default()),

            // no other tokens are allowed to start a line
            _ => {
                return Err(LineError {
                    token: Some(token.clone()),
                    message: "Expected the line to start with a label, statement, or comma",
                })
            }
        }
    }

    fn mnemonic(&mut self, identifier: &Token<'a>) -> Result<Mnemonic, LineError<'a>> {
        match MNEMONICS.get(&UniCase::new(identifier.span)).cloned() {
            None => Err(LineError {
                token: Some(identifier.clone()),
                message: "Invalid Mnemonic",
            }),
            Some(result) => Ok(result),
        }
    }

    fn statement(&mut self, mnemonic: Mnemonic) -> Result<Statement, LineError<'a>> {
        Ok(match mnemonic {
            // Branch
            Mnemonic::B { condition, l } => {
                let target = self.label()?;

                Statement::Instruction {
                    kind: InstructionKind::Branch {
                        condition,
                        kind: if l {
                            BranchKind::BranchWithLink
                        } else {
                            BranchKind::Branch
                        },
                        target,
                    },
                }
            }

            // Data Processing - Move
            Mnemonic::MOV {
                condition,
                s: set_flags,
            } => {
                let destination = self.register()?.into();
                self.comma()?;
                let shifter = self.shifter()?;

                Statement::Instruction {
                    kind: InstructionKind::DataProcessing {
                        condition,
                        kind: DataProcessingKind::Move {
                            kind: MoveKind::Move,
                            set_flags: if set_flags {
                                SetFlags::Set
                            } else {
                                SetFlags::DontSet
                            },
                            destination,
                            shifter,
                        },
                    },
                }
            }

            // Data Processing - Comparison
            Mnemonic::CMP { condition } => {
                let source = self.register()?.into();
                self.comma()?;
                let shifter = self.shifter()?;

                Statement::Instruction {
                    kind: InstructionKind::DataProcessing {
                        condition,
                        kind: DataProcessingKind::Comparison {
                            kind: ComparisonKind::CMP,
                            source,
                            shifter,
                        },
                    },
                }
            }

            // Data Processing - Calculation
            Mnemonic::ADD {
                condition,
                s: set_flags,
            }
            | Mnemonic::SUB {
                condition,
                s: set_flags,
            } => {
                let kind = match mnemonic {
                    Mnemonic::ADD { .. } => CalculationKind::ADD,
                    Mnemonic::SUB { .. } => CalculationKind::SUB,
                    _ => unreachable!(),
                };

                let destination = self.register()?.into();
                self.comma()?;
                let source = self.register()?.into();
                self.comma()?;
                let shifter = self.shifter()?;

                Statement::Instruction {
                    kind: InstructionKind::DataProcessing {
                        condition,
                        kind: DataProcessingKind::Calculation {
                            kind,
                            set_flags: if set_flags {
                                SetFlags::Set
                            } else {
                                SetFlags::DontSet
                            },
                            destination,
                            source,
                            shifter,
                        },
                    },
                }
            }

            // Load/Store
            // LDR can be either an instruction or pseudo-instruction so handle it separately
            Mnemonic::LDR { condition } => {
                let destination = self.register()?.into();
                self.comma()?;

                match self.lexer.peek_ignore_whitespace() {
                    Some(Token {
                        kind: TokenKind::EqualSign,
                        ..
                    }) => {
                        // eat the =
                        self.lexer.next();

                        let value = self.expression()?;

                        Statement::PseudoInstruction {
                            kind: PseudoInstructionKind::LoadRegisterConstant {
                                condition,
                                destination,
                                value,
                            },
                        }
                    }
                    _ => {
                        let address = self.load_store_address()?;

                        Statement::Instruction {
                            kind: InstructionKind::LoadStore {
                                condition,
                                kind: LoadStoreKind::Load,
                                quantity: LoadStoreQuantity::Word,
                                destination,
                                address,
                            },
                        }
                    }
                }
            }

            // all these instructions have the same structure
            Mnemonic::LDRB { condition }
            | Mnemonic::STR { condition }
            | Mnemonic::STRB { condition } => {
                let (kind, quantity) = match mnemonic {
                    Mnemonic::LDRB { .. } => (LoadStoreKind::Load, LoadStoreQuantity::Byte),
                    Mnemonic::STR { .. } => (LoadStoreKind::Store, LoadStoreQuantity::Word),
                    Mnemonic::STRB { .. } => (LoadStoreKind::Store, LoadStoreQuantity::Byte),
                    _ => unreachable!(),
                };

                let destination = self.register()?.into();
                self.comma()?;
                let address = self.load_store_address()?;

                Statement::Instruction {
                    kind: InstructionKind::LoadStore {
                        condition,
                        kind,
                        quantity,
                        destination,
                        address,
                    },
                }
            }

            // Load/Store Multiple
            Mnemonic::LDM { condition, mode } | Mnemonic::STM { condition, mode } => {
                let kind = match mnemonic {
                    Mnemonic::LDM { .. } => LoadStoreKind::Load,
                    Mnemonic::STM { .. } => LoadStoreKind::Store,
                    _ => unreachable!(),
                };

                let base = self.register()?.into();

                let write_back = match self.lexer.next_ignore_whitespace() {
                    Some(Token {
                        kind: TokenKind::ExclamationMark,
                        ..
                    }) => {
                        self.comma()?;
                        WriteBack::WriteBack
                    }
                    Some(Token {
                        kind: TokenKind::Comma,
                        ..
                    }) => WriteBack::NoWriteBack,
                    token => {
                        return Err(LineError {
                            token,
                            message: "Expected an Exclamation Mark or Comma",
                        })
                    }
                };

                let register_list = self.register_list()?;

                Statement::Instruction {
                    kind: InstructionKind::LoadStoreMultiple {
                        condition,
                        kind,
                        mode,
                        base,
                        write_back,
                        register_list,
                    },
                }
            }

            // SuperVisor Call
            Mnemonic::SVC { condition } => {
                let immediate = self.expression()?;

                Statement::Instruction {
                    kind: InstructionKind::SuperVisorCall {
                        condition,
                        immediate,
                    },
                }
            }

            // Pseudo-Instruction - Address Register
            Mnemonic::ADR { condition, l: long } => {
                let destination = self.register()?;
                self.comma()?;
                let label = self.label()?;

                Statement::PseudoInstruction {
                    kind: PseudoInstructionKind::AddressRegister {
                        condition,
                        long,
                        destination,
                        label,
                    },
                }
            }

            // Directive - Define Space
            Mnemonic::DEFS => {
                let size = self.number()? as usize;

                let fill = match self.lexer.peek_ignore_whitespace() {
                    Some(Token {
                        kind: TokenKind::Comma,
                        ..
                    }) => {
                        self.lexer.next();

                        Some(self.number()? as u8) // TODO: check it fits in u8, and allow for characters
                    }
                    _ => None,
                };

                Statement::Directive {
                    kind: DirectiveKind::Definition {
                        kind: DefinitionKind::Space { size, fill },
                    },
                }
            }

            Mnemonic::DEFB => {
                let mut bytes = Vec::new();

                loop {
                    match self.lexer.next_ignore_whitespace() {
                        Some(Token {
                            kind: TokenKind::Decimal,
                            span,
                            ..
                        }) => {
                            bytes
                                .push(BytesDefinition::Byte(u8::from_str_radix(span, 10).unwrap()));
                        }
                        Some(Token {
                            kind: TokenKind::String,
                            span,
                            ..
                        }) => {
                            bytes.push(BytesDefinition::String(span[1..span.len() - 1].to_owned()))
                        }
                        _ => break,
                    }

                    match self.lexer.peek_ignore_whitespace() {
                        Some(Token {
                            kind: TokenKind::Comma,
                            ..
                        }) => {
                            self.lexer.next();
                            continue;
                        }
                        _ => break,
                    }
                }

                Statement::Directive {
                    kind: DirectiveKind::Definition {
                        kind: DefinitionKind::Bytes { bytes },
                    },
                }
            }

            // Directive - Define Words
            Mnemonic::DEFW => {
                let mut words = Vec::new();

                loop {
                    match self.lexer.next_ignore_whitespace() {
                        Some(Token {
                            kind: TokenKind::Decimal,
                            span,
                            ..
                        }) => {
                            words.push(u32::from_str_radix(span, 10).unwrap());
                        }
                        _ => break,
                    }

                    match self.lexer.peek_ignore_whitespace() {
                        Some(Token {
                            kind: TokenKind::Comma,
                            ..
                        }) => {
                            self.lexer.next();
                            continue;
                        }
                        _ => break,
                    }
                }

                Statement::Directive {
                    kind: DirectiveKind::Definition {
                        kind: DefinitionKind::Words { words },
                    },
                }
            }

            // Directive - Align
            Mnemonic::ALIGN => Statement::Directive {
                kind: DirectiveKind::Align,
            },

            // Directive - Origin
            Mnemonic::ORIGIN => {
                let address = self.expression()?;

                Statement::Directive {
                    kind: DirectiveKind::Origin { address },
                }
            }

            // Directive - Entry Point
            Mnemonic::ENTRY => Statement::Directive {
                kind: DirectiveKind::EntryPoint,
            },

            // Directive - Constant
            Mnemonic::EQU => {
                let value = self.expression()?;

                Statement::Directive {
                    kind: DirectiveKind::Constant { value },
                }
            }
        })
    }

    fn line_end(&mut self) -> Result<(), LineError<'a>> {
        match self.lexer.next_ignore_whitespace() {
            Some(Token {
                kind: TokenKind::Comment,
                ..
            }) => self.new_line_or_eof(),
            Some(Token {
                kind: TokenKind::NewLine,
                ..
            }) => Ok(()),
            None => Ok(()),

            token => Err(LineError {
                token,
                message: "Expected a comment, or the end of the line",
            }),
        }
    }

    fn new_line_or_eof(&mut self) -> Result<(), LineError<'a>> {
        match self.lexer.next_ignore_whitespace() {
            None
            | Some(Token {
                kind: TokenKind::NewLine,
                ..
            }) => Ok(()),

            token => Err(LineError {
                token,
                message: "Expected the end of the line",
            }),
        }
    }

    fn number(&mut self) -> Result<u32, LineError<'a>> {
        match self.lexer.next_ignore_whitespace() {
            Some(Token {
                kind: TokenKind::Decimal,
                span,
                ..
            }) => Ok(u32::from_str_radix(span, 10).unwrap()),

            token => Err(LineError {
                token,
                message: "Expected a number",
            }),
        }
    }

    fn register_list(&mut self) -> Result<RegisterList, LineError<'a>> {
        match self.lexer.next_ignore_whitespace() {
            Some(Token {
                kind: TokenKind::OpenBrace,
                ..
            }) => (),
            token => {
                return Err(LineError {
                    token,
                    message: "Expected a Register List",
                })
            }
        }

        let mut registers = [false; 16];

        loop {
            let register = self.register()?;

            match self.lexer.next_ignore_whitespace() {
                // Range
                Some(Token {
                    kind: TokenKind::HyphenMinus,
                    ..
                }) => {
                    let to_register = self.register()?;

                    let range = register.0..=to_register.0;

                    for i in range {
                        registers[i as usize] = true;
                    }

                    match self.lexer.next_ignore_whitespace() {
                        Some(Token {
                            kind: TokenKind::Comma,
                            ..
                        }) => continue,
                        Some(Token {
                            kind: TokenKind::CloseBrace,
                            ..
                        }) => break,
                        token => {
                            return Err(LineError {
                                token,
                                message: "Invalid token in Register List",
                            })
                        }
                    }
                }

                // Comma
                Some(Token {
                    kind: TokenKind::Comma,
                    ..
                }) => {
                    let n = register.0;
                    registers[n as usize] = true;

                    continue;
                }

                // End
                Some(Token {
                    kind: TokenKind::CloseBrace,
                    ..
                }) => {
                    let n = register.0;
                    registers[n as usize] = true;

                    break;
                }

                token => {
                    return Err(LineError {
                        token,
                        message: "Invalid token in Register List",
                    })
                }
            }
        }

        Ok(RegisterList { registers })
    }

    fn close_brace(&mut self) -> Result<(), LineError<'a>> {
        match self.lexer.next_ignore_whitespace() {
            Some(Token {
                kind: TokenKind::CloseBrace,
                ..
            }) => Ok(()),
            token => Err(LineError {
                token,
                message: "Expected a Close Brace",
            }),
        }
    }

    fn shifter(&mut self) -> Result<ShifterOperandExpression, LineError<'a>> {
        match self.lexer.peek_ignore_whitespace() {
            Some(Token {
                kind: TokenKind::LiteralSign,
                ..
            }) => {
                // eat the #
                self.lexer.next();

                Ok(ShifterOperandExpression::Immediate(self.expression()?))
            }

            Some(Token {
                kind: TokenKind::Identifier,
                ..
            }) => {
                let register = self.register()?.into();

                match self.lexer.peek_ignore_whitespace() {
                    Some(Token {
                        kind: TokenKind::Comma,
                        ..
                    }) => {
                        // eat the comma
                        self.lexer.next();

                        let shift_name = self.shift_name()?;

                        match shift_name {
                            ShiftName::RotateRightExtended => {
                                Ok(ShifterOperandExpression::RotateRightWithExtend(register))
                            }
                            _ => {
                                let kind = shift_name.try_into().unwrap();

                                let amount = self.shifter_shift_amount()?;

                                Ok(ShifterOperandExpression::ShiftedRegister(ShiftedRegister {
                                    kind,
                                    amount,
                                    base: register,
                                }))
                            }
                        }
                    }
                    _ => Ok(ShifterOperandExpression::Register(register)),
                }
            }

            _ => {
                return Err(LineError {
                    token: self.lexer.next(),
                    message: "Invalid Operand. Expected a Literal or a Register",
                })
            }
        }
    }

    fn shifter_shift_amount(&mut self) -> Result<ShifterOperandShiftAmount, LineError<'a>> {
        match self.lexer.peek_ignore_whitespace() {
            Some(Token {
                kind: TokenKind::LiteralSign,
                ..
            }) => {
                // eat the #
                self.lexer.next();

                Ok(ShifterOperandShiftAmount::Immediate(self.expression()?))
            }

            Some(Token {
                kind: TokenKind::Identifier,
                ..
            }) => Ok(ShifterOperandShiftAmount::Register(self.register()?.into())),

            _ => {
                return Err(LineError {
                    token: self.lexer.next(),
                    message: "Invalid Shift Value",
                })
            }
        }
    }

    fn load_store_address(&mut self) -> Result<LoadStoreAddress, LineError<'a>> {
        match self.lexer.peek_ignore_whitespace() {
            // Addressing Mode
            Some(Token {
                kind: TokenKind::OpenBracket,
                ..
            }) => {
                // eat the {
                self.lexer.next();

                let base = self.register()?.into();

                let offset = self.addressing_offset()?;

                Ok(LoadStoreAddress::AddressingMode(LoadStoreAddressCode {
                    base,
                    offset,
                }))
            }

            // Address
            Some(Token {
                kind: TokenKind::LiteralSign,
                ..
            }) => {
                // eat the #
                self.lexer.next();

                let expression = self.expression()?;

                Ok(LoadStoreAddress::Expression(expression))
            }

            // treat as an expression even without the #
            // this isn't in the ARM spec because it's up to the assembler, but this is how aasm seems to work
            Some(_) => {
                let expression = self.expression()?;

                Ok(LoadStoreAddress::Expression(expression))
            }

            None => Err(LineError {
                token: None,
                message: "Expected a Load Store Address",
            }),
        }
    }

    fn addressing_offset(
        &mut self,
    ) -> Result<AddressingOffset<Expression, Expression>, LineError<'a>> {
        match self.lexer.next_ignore_whitespace() {
            // No offset or Post-Index
            Some(Token {
                kind: TokenKind::CloseBracket,
                ..
            }) => {
                match self.lexer.peek_ignore_whitespace() {
                    // Post-Index
                    Some(Token {
                        kind: TokenKind::Comma,
                        ..
                    }) => {
                        // eat the comma
                        self.lexer.next();

                        let (value, sign) = self.offset_value()?;

                        Ok(AddressingOffset {
                            sign,
                            value,
                            mode: OffsetMode::PostIndexed,
                        })
                    }

                    // No Offset
                    _ => Ok(AddressingOffset {
                        sign: Sign::Positive,
                        value: AddressingOffsetValue::Immediate(Expression::Number {
                            base: 10,
                            n: 0,
                        }),
                        mode: OffsetMode::Offset,
                    }),
                }
            }

            // Offset or Pre-Indexed
            Some(Token {
                kind: TokenKind::Comma,
                ..
            }) => {
                // eat the comma
                self.lexer.next();

                let (value, sign) = self.offset_value()?;

                let mode = match self.lexer.next_ignore_whitespace() {
                    Some(Token {
                        kind: TokenKind::CloseBracket,
                        ..
                    }) => {
                        match self.lexer.peek_ignore_whitespace() {
                            // Pre-Indexed
                            Some(Token {
                                kind: TokenKind::ExclamationMark,
                                ..
                            }) => {
                                // eat the !
                                self.lexer.next();

                                OffsetMode::PreIndexed
                            }

                            // Offset
                            _ => OffsetMode::Offset,
                        }
                    }
                    token => {
                        return Err(LineError {
                            token,
                            message: "Expected a Close Bracket",
                        })
                    }
                };

                Ok(AddressingOffset { sign, value, mode })
            }

            token => Err(LineError {
                token,
                message: "Invalid Addressing Offset",
            }),
        }
    }

    fn offset_value(
        &mut self,
    ) -> Result<(AddressingOffsetValue<Expression, Expression>, Sign), LineError<'a>> {
        match self.lexer.peek_ignore_whitespace() {
            // Literal
            Some(Token {
                kind: TokenKind::LiteralSign,
                ..
            }) => {
                // eat the #
                self.lexer.next();

                let sign = self.sign();

                let offset = self.expression()?;

                Ok((AddressingOffsetValue::Immediate(offset), sign))
            }

            // Register or Scaled Register
            Some(Token {
                kind: TokenKind::Identifier | TokenKind::Plus | TokenKind::HyphenMinus,
                ..
            }) => {
                let sign = self.sign();

                let register = self.register()?.into();

                let format = match self.lexer.peek_ignore_whitespace() {
                    Some(Token {
                        kind: TokenKind::Comma,
                        ..
                    }) => {
                        // eat the comma
                        self.lexer.next();

                        let kind = self.shift_kind()?;

                        match self.lexer.next_ignore_whitespace() {
                            Some(Token {
                                kind: TokenKind::LiteralSign,
                                ..
                            }) => (),
                            token => {
                                return Err(LineError {
                                    token,
                                    message: "Expected a Literal Value for the Shift",
                                })
                            }
                        }

                        let amount = self.expression()?;

                        AddressingOffsetValue::ScaledRegister(ShiftedRegister {
                            kind,
                            amount,
                            base: register,
                        })
                    }
                    _ => AddressingOffsetValue::Register(register),
                };

                Ok((format, sign))
            }

            _ => Err(LineError {
                token: self.lexer.next(),
                message: "Invalid Offset Format",
            }),
        }
    }

    fn sign(&mut self) -> Sign {
        match self.lexer.peek_ignore_whitespace() {
            Some(Token {
                kind: TokenKind::Plus,
                ..
            }) => {
                // eat the sign
                self.lexer.next();

                Sign::Positive
            }

            Some(Token {
                kind: TokenKind::HyphenMinus,
                ..
            }) => {
                // eat the sign
                self.lexer.next();

                Sign::Negative
            }

            _ => Sign::Positive,
        }
    }

    fn shift_name(&mut self) -> Result<ShiftName, LineError<'a>> {
        match self.lexer.next_ignore_whitespace() {
            Some(
                token @ Token {
                    kind: TokenKind::Identifier,
                    span,
                    ..
                },
            ) => match SHIFT_NAMES.get(&UniCase::new(span)).cloned() {
                Some(shift) => Ok(shift),
                _ => Err(LineError {
                    token: Some(token),
                    message: "Invalid Shift Operation",
                }),
            },
            token => Err(LineError {
                token,
                message: "Expected a Shift Operation",
            }),
        }
    }

    fn shift_kind(&mut self) -> Result<Shift, LineError<'a>> {
        match self.lexer.next_ignore_whitespace() {
            Some(
                token @ Token {
                    kind: TokenKind::Identifier,
                    span,
                    ..
                },
            ) => match SHIFT_KINDS.get(&UniCase::new(span)).cloned() {
                Some(shift) => Ok(shift),
                _ => Err(LineError {
                    token: Some(token),
                    message: "Invalid Shift Operation",
                }),
            },
            token => Err(LineError {
                token,
                message: "Expected a Shift Operation",
            }),
        }
    }

    fn comma(&mut self) -> Result<(), LineError<'a>> {
        match self.lexer.next_ignore_whitespace() {
            Some(Token {
                kind: TokenKind::Comma,
                ..
            }) => Ok(()),
            token => Err(LineError {
                token,
                message: "Expected a comma",
            }),
        }
    }

    fn register(&mut self) -> Result<Register, LineError<'a>> {
        match self.lexer.next_ignore_whitespace() {
            Some(
                token @ Token {
                    kind: TokenKind::Identifier,
                    span,
                    ..
                },
            ) => match REGISTERS.get(&UniCase::new(span)).cloned() {
                Some(register) => Ok(register),
                _ => Err(LineError {
                    token: Some(token),
                    message: "Invalid Register",
                }),
            },
            token => Err(LineError {
                token,
                message: "Expected a Register",
            }),
        }
    }

    fn label(&mut self) -> Result<Symbol, LineError<'a>> {
        match self.lexer.next_ignore_whitespace() {
            Some(Token {
                kind: TokenKind::Identifier,
                span,
                ..
            }) => Ok(Symbol(span.to_owned())),
            token => Err(LineError {
                token,
                message: "Expected a label",
            }),
        }
    }

    fn expression(&mut self) -> Result<Expression, LineError<'a>> {
        let expression = match self.lexer.next_ignore_whitespace() {
            Some(Token {
                kind: TokenKind::Decimal,
                span,
                ..
            }) => Ok(Expression::Number {
                base: 10,
                n: u32::from_str_radix(span, 10).unwrap(),
            }),

            Some(Token {
                kind: TokenKind::Hexadecimal,
                span,
                ..
            }) => Ok(Expression::Number {
                base: 16,
                n: u32::from_str_radix(&span[2..], 16).unwrap(),
            }),

            Some(Token {
                kind: TokenKind::NBaseNumber,
                span,
                ..
            }) => {
                let (base, number) = span.split_once('_').unwrap();
                let base = base.parse().unwrap();

                Ok(Expression::Number {
                    base,
                    n: u32::from_str_radix(number, base).unwrap(),
                })
            }

            Some(
                token @ Token {
                    kind: TokenKind::Character,
                    span,
                    ..
                },
            ) => match span[1..span.len() - 1].chars().next() {
                Some(char) if span.len() == 3 => Ok(Expression::Character(char)),
                _ => Err(LineError {
                    token: Some(token),
                    message: "Expected a single character within the single quotes",
                }),
            },

            Some(Token {
                kind: TokenKind::String,
                span,
                ..
            }) => Ok(Expression::String(span[1..span.len() - 1].to_string())),

            // boolean
            Some(Token {
                kind: TokenKind::OpenBrace,
                ..
            }) => {
                let boolean = match self.lexer.next() {
                    Some(Token {
                        kind: TokenKind::Identifier,
                        span: "TRUE",
                        ..
                    }) => true,
                    Some(Token {
                        kind: TokenKind::Identifier,
                        span: "FALSE",
                        ..
                    }) => false,
                    token => {
                        return Err(LineError {
                            token,
                            message: "Expected TRUE or FALSE after an Open Brace",
                        })
                    }
                };

                self.close_brace()?;

                Ok(Expression::Boolean(boolean))
            }

            Some(Token {
                kind: TokenKind::Identifier,
                span,
                ..
            }) => Ok(Expression::Symbol(Symbol(span.to_owned()))),

            token => Err(LineError {
                token,
                message: "Expected Number, Character, String, or Boolean",
            }),
        };

        match self.lexer.peek_ignore_whitespace() {
            Some(Token {
                kind: kind @ TokenKind::Plus | kind @ TokenKind::HyphenMinus,
                ..
            }) => {
                let operator = match kind {
                    TokenKind::Plus => DiadicOperator::Plus,
                    TokenKind::HyphenMinus => DiadicOperator::Minus,
                    _ => unreachable!(),
                };

                // eat the operator
                self.lexer.next();

                Ok(Expression::Diadic(
                    Box::new(expression?),
                    operator,
                    Box::new(self.expression()?),
                ))
            }

            _ => expression,
        }
    }
}
