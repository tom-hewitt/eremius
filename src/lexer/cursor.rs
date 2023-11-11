use std::{ops::Range, str::Chars};

pub struct Cursor<'a> {
    chars: Chars<'a>,
    current_token: Range<usize>,
}

impl<'a> Cursor<'a> {
    pub fn new(input: &'a str) -> Cursor<'a> {
        Cursor {
            chars: input.chars(),
            current_token: 0..0,
        }
    }

    pub fn finish_token(&mut self) -> Range<usize> {
        let range = self.current_token.clone();

        self.current_token = range.end..range.end;

        range
    }

    pub fn peek(&mut self) -> Option<char> {
        self.chars.clone().next()
    }

    pub fn eat_while(&mut self, mut predicate: impl FnMut(char) -> bool) {
        loop {
            match self.peek() {
                Some(char) if predicate(char) => {
                    self.next();
                }
                Some(_) | None => break,
            }
        }
    }
}

impl<'a> Iterator for Cursor<'a> {
    type Item = char;

    fn next(&mut self) -> Option<char> {
        self.current_token.end += 1;

        self.chars.next()
    }
}
