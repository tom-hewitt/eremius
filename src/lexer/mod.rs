use std::{iter::Peekable, ops::Range};

use cursor::Cursor;

mod cursor;

#[cfg(test)]
mod tests;

#[derive(Debug, PartialEq, Clone)]
pub struct Token<'a> {
    pub kind: TokenKind,
    pub span: &'a str,
    pub range: Range<usize>,
}

impl<'a> Token<'a> {
    pub fn separate_suffix(&self, n: usize) -> (&'a str, &'a str) {
        let boundary = self.span.len() - n;

        if boundary <= 0 {
            return ("", self.span);
        }

        (&self.span[..boundary], &self.span[boundary..])
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum TokenKind {
    /// A Symbol, Register, Boolean, Instruction (Condition included), Directive, or Psuedo-Instruction
    ///
    /// # Examples
    /// Symbol: `loop`
    ///
    /// Register: `R0`, `r0`, `SP`
    ///
    /// Boolean: `TRUE`
    ///
    /// Instruction: `STR`, `BNE`, `add`
    ///
    /// Directive: `DEFW`
    ///
    /// Psuedo-Instruction: `ADRL`
    Identifier,

    /// A Comment, starting with a semi-colon and occupying the remainder of the line it begins on
    ///
    /// # Examples
    /// `; explanation goes here!`
    Comment,

    /// A decimal number
    Decimal,

    /// A hexadecimal number prefixed with 0x
    Hexadecimal,

    /// An n-base number with an underscore between the base and the number
    NBaseNumber,

    /// A character in single quotes
    Character,

    /// A String used after a Literal Sign (#), DEFW, or SVC
    ///
    /// Should be converted to a char if it only contains one character
    String,

    /// A `#` symbol, used before a decimal, hexadecimal, n-base number, character, single-character string, or boolean in braces
    LiteralSign,

    /// Braces, used to contain a Boolean, or a sequence of registers for stack instructions
    ///
    /// # Examples
    /// Boolean: `{TRUE}`
    ///
    /// Registers: `{r0-r5}`
    OpenBrace,
    CloseBrace,

    /// Brackets, used to contain a Load/Store data offset
    OpenBracket,
    CloseBracket,

    /// A '!', used for pre-index addressing
    ExclamationMark,

    /// A `=` symbol, used in the `LDR Rd, =const` pseudo-instruction
    EqualSign,

    /// A `+` symbol, used in addressing offsets for load store instructions
    Plus,
    /// A `-` symbol, used as a minus sign in addressing offsets for load store instructions, or as a hyphen in register lists
    HyphenMinus,

    /// A Commma, used to delimit instruction arguments
    Comma,

    /// A new line
    NewLine,

    /// A sequence of unicode whitespace (excluding the newline character)
    Whitespace,

    Unknown,
}

pub struct Tokens<'a> {
    input: &'a str,
    cursor: Cursor<'a>,
}

impl<'a> Iterator for Tokens<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Token<'a>> {
        let char = match self.cursor.next() {
            Some(c) => c,
            None => return None,
        };

        let kind = match char {
            // Comment
            ';' => self.comment(),

            // New Line
            '\n' => TokenKind::NewLine,

            // Whitespace Sequence
            c if c.is_whitespace() => self.whitespace(),

            // decimal, hexadecimal, or n-base number
            digit @ '0'..='9' => self.number(digit),

            '&' => self.hexadecimal(),

            // character
            '\'' => self.character(),

            // String
            '"' => self.string(),

            // Identifier
            'A'..='Z' | 'a'..='z' | '_' => self.identifier(),

            // One-Symbol Tokens
            '#' => TokenKind::LiteralSign,
            '{' => TokenKind::OpenBrace,
            '}' => TokenKind::CloseBrace,
            '[' => TokenKind::OpenBracket,
            ']' => TokenKind::CloseBracket,
            '!' => TokenKind::ExclamationMark,
            '=' => TokenKind::EqualSign,
            '+' => TokenKind::Plus,
            '-' => TokenKind::HyphenMinus,
            ',' => TokenKind::Comma,

            _ => TokenKind::Unknown,
        };

        let range = self.cursor.finish_token();
        let span = &self.input[range.clone()];

        Some(Token { kind, span, range })
    }
}

pub struct Lexer<'a> {
    pub input: &'a str,
    tokens: Peekable<Tokens<'a>>,
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.tokens.next()
    }
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Lexer<'a> {
        Lexer {
            input,
            tokens: Tokens::new(input).peekable(),
        }
    }

    pub fn peek(&mut self) -> Option<&Token<'a>> {
        self.tokens.peek()
    }

    pub fn next_ignore_whitespace(&mut self) -> Option<Token<'a>> {
        loop {
            match self.next() {
                Some(Token {
                    kind: TokenKind::Whitespace,
                    ..
                }) => continue,
                next => return next,
            }
        }
    }

    pub fn peek_ignore_whitespace(&mut self) -> Option<&Token<'a>> {
        loop {
            match self.peek() {
                Some(Token {
                    kind: TokenKind::Whitespace,
                    ..
                }) => {
                    self.next();
                }
                _ => break,
            }
        }

        self.peek()
    }
}

impl<'a> Tokens<'a> {
    pub fn new(input: &'a str) -> Tokens<'a> {
        Tokens {
            input,
            cursor: Cursor::new(input),
        }
    }

    fn comment(&mut self) -> TokenKind {
        self.cursor.eat_while(|c| c != '\n');

        TokenKind::Comment
    }

    fn whitespace(&mut self) -> TokenKind {
        self.cursor.eat_while(char::is_whitespace);

        TokenKind::Whitespace
    }

    fn identifier(&mut self) -> TokenKind {
        self.cursor.eat_while(valid_identifier_char);

        TokenKind::Identifier
    }

    /// tokenizes a decimal, hexadecimal, or n-base number
    fn number(&mut self, first_digit: char) -> TokenKind {
        if first_digit == '0' {
            if let Some('x') = self.cursor.peek() {
                // eat the x
                self.cursor.next();

                return self.hexadecimal();
            }
        }

        self.cursor.eat_while(char::is_numeric);

        match self.cursor.peek() {
            Some('_') => {
                // eat the _
                self.cursor.next();

                // eat the number
                self.cursor.eat_while(char::is_numeric);

                TokenKind::NBaseNumber
            }

            _ => TokenKind::Decimal,
        }
    }

    fn hexadecimal(&mut self) -> TokenKind {
        // eat the number
        self.cursor.eat_while(|c| c.is_ascii_hexdigit());

        return TokenKind::Hexadecimal;
    }

    fn character(&mut self) -> TokenKind {
        loop {
            match self.cursor.next() {
                // skip next character
                Some('\\') => {
                    self.cursor.next();
                }

                Some('\'') => break,

                Some(_) => continue,

                None => panic!("unterminated character literal"),
            }
        }

        TokenKind::Character
    }

    fn string(&mut self) -> TokenKind {
        loop {
            match self.cursor.next() {
                // skip next character
                Some('\\') => {
                    self.cursor.next();
                }

                Some('"') => break,

                Some(_) => continue,

                None => panic!("unterminated string literal"),
            }
        }

        TokenKind::String
    }
}

/// From the ARMv5 Assembler User Guide:
///
/// "You can use uppercase letters, lowercase letters, numeric characters, or the underscore character in
/// symbol names. Symbol names are case-sensitive, and all characters in the symbol name are
/// significant.""
fn valid_identifier_char(c: char) -> bool {
    match c {
        'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => true,
        _ => false,
    }
}
