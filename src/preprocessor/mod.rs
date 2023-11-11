use std::collections::HashMap;
use std::iter;

use smallvec::SmallVec;

use crate::assembler::AssemblyError;
use crate::parser::{
    DefinitionKind, DirectiveKind, Expression, Line, ParseError, Parser, PseudoInstructionKind,
    Statement, StatementInstructionKind, Symbol,
};
use crate::resolver::SymbolTable;

mod tests;

#[derive(Debug)]
pub enum PreProcessedStatement {
    Instruction { kind: StatementInstructionKind },
    PseudoInstruction { kind: PseudoInstructionKind },
    // most data definitions will be a single word, so we can increase the performance by using a small vector, which doesn't allocate until its length is greater than 4
    Data(SmallVec<[u8; 4]>),
}

#[derive(Debug)]
pub enum PreProcessError {
    ParseError(ParseError),
    OriginAddressError,
}

impl<'a> Parser<'a> {
    pub fn preprocess(self) -> Result<PreProcessResult, AssemblyError> {
        Ok(PreProcessor::new().run(self)?)
    }
}

struct PreProcessor {
    /// a pair of an address and a statement
    statements: Vec<(usize, PreProcessedStatement)>,
    symbol_table: SymbolTable<Expression>,
    entry_point: usize,
    /// maps the address of a statement to the source line that generated it
    pub source_map: HashMap<usize, usize>,
    address: usize,
    /// a queue of labels to be inserted into the symbol table at the next address
    label_queue: Vec<String>,
}

#[derive(Debug)]
pub struct PreProcessResult {
    pub statements: Vec<(usize, PreProcessedStatement)>,
    pub symbol_table: SymbolTable<Expression>,
    pub entry_point: usize,
    /// maps the address of a statement to the source line that generated it
    pub source_map: HashMap<usize, usize>,
}

impl PreProcessor {
    pub fn new() -> Self {
        Self {
            statements: Vec::new(),
            symbol_table: SymbolTable::new(),
            entry_point: 0,
            source_map: HashMap::new(),
            address: 0,
            label_queue: Vec::new(),
        }
    }

    pub fn run<'a>(
        mut self,
        lines: impl Iterator<Item = Result<Line, ParseError>> + 'a,
    ) -> Result<PreProcessResult, PreProcessError> {
        for (source_line, line) in lines.enumerate() {
            match line {
                Err(e) => return Err(PreProcessError::ParseError(e)),

                Ok(line) => {
                    // insert the label into the symbol table
                    match line.label {
                        Some(label) => self.label_queue.push(label),
                        None => (),
                    }

                    match line.statement {
                        None => (),

                        Some(statement) => match statement {
                            Statement::Instruction { kind } => self.insert_addressed_statement(
                                PreProcessedStatement::Instruction { kind },
                                4,
                                source_line,
                            ),

                            Statement::PseudoInstruction { kind } => match kind {
                                PseudoInstructionKind::AddressRegister { long, .. } => self
                                    .insert_addressed_statement(
                                        PreProcessedStatement::PseudoInstruction { kind },
                                        if long { 8 } else { 4 },
                                        source_line,
                                    ),

                                PseudoInstructionKind::LoadRegisterConstant { .. } => self
                                    .insert_addressed_statement(
                                        PreProcessedStatement::PseudoInstruction { kind },
                                        4,
                                        source_line,
                                    ),
                            },

                            // we need to apply assembler directives
                            Statement::Directive { kind } => match kind {
                                DirectiveKind::Definition { kind } => {
                                    let bytes: SmallVec<[u8; 4]> = match kind {
                                        DefinitionKind::Space { size, fill } => {
                                            iter::repeat(fill.unwrap_or(0)).take(size).collect()
                                        }
                                        DefinitionKind::Bytes { bytes } => bytes
                                            .into_iter()
                                            // use flat map with an inner iterator to avoid unnecessary allocations
                                            .flat_map(|bytes| bytes.into_iter())
                                            .collect(),
                                        DefinitionKind::Words { words } => {
                                            words.into_iter().flat_map(u32::to_be_bytes).collect()
                                        }
                                    };

                                    let size = bytes.len();

                                    self.insert_addressed_statement(
                                        PreProcessedStatement::Data(bytes),
                                        size,
                                        source_line,
                                    );
                                }

                                DirectiveKind::Align => {
                                    self.address += 4 - (self.address % 4);
                                }

                                DirectiveKind::Constant { value } => {
                                    for label in self.label_queue.drain(..) {
                                        self.symbol_table.insert(Symbol(label), value.clone());
                                    }
                                }

                                DirectiveKind::Origin { address } => {
                                    self.address =
                                        match address.backwards_resolve(&self.symbol_table) {
                                            Ok(n) => n as usize,
                                            _ => return Err(PreProcessError::OriginAddressError),
                                        };
                                }

                                DirectiveKind::EntryPoint => {
                                    self.entry_point = self.address;
                                }
                            },
                        },
                    }
                }
            }
        }

        Ok(PreProcessResult {
            statements: self.statements,
            symbol_table: self.symbol_table,
            entry_point: self.entry_point,
            source_map: self.source_map,
        })
    }

    fn insert_addressed_statement(
        &mut self,
        statement: PreProcessedStatement,
        size: usize,
        source_line: usize,
    ) {
        for label in self.label_queue.drain(..) {
            self.symbol_table.insert(
                Symbol(label),
                Expression::Number {
                    base: 10,
                    n: self.address as u32,
                },
            );
        }

        self.source_map.insert(self.address, source_line);

        self.statements.push((self.address, statement));

        self.address += size;
    }
}
