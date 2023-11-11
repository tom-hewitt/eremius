use crate::lexer::{Token, TokenKind::*, Tokens};

#[test]
fn test() {
    let input = "   B loop ; infinite loop!
    SVC 2
    LDR R12, [R13, #-4]";

    let tokens: Vec<Token> = Tokens::new(input).collect();

    let expected = vec![
        Token {
            kind: Whitespace,
            span: "   ",
            range: 0..3,
        },
        Token {
            kind: Identifier,
            span: "B",
            range: 3..4,
        },
        Token {
            kind: Whitespace,
            span: " ",
            range: 4..5,
        },
        Token {
            kind: Identifier,
            span: "loop",
            range: 5..9,
        },
        Token {
            kind: Whitespace,
            span: " ",
            range: 9..10,
        },
        Token {
            kind: Comment,
            span: "; infinite loop!",
            range: 10..26,
        },
        Token {
            kind: NewLine,
            span: "\n",
            range: 26..27,
        },
        Token {
            kind: Whitespace,
            span: "    ",
            range: 27..31,
        },
        Token {
            kind: Identifier,
            span: "SVC",
            range: 31..34,
        },
        Token {
            kind: Whitespace,
            span: " ",
            range: 34..35,
        },
        Token {
            kind: Decimal,
            span: "2",
            range: 35..36,
        },
        Token {
            kind: NewLine,
            span: "\n",
            range: 36..37,
        },
        Token {
            kind: Whitespace,
            span: "    ",
            range: 37..41,
        },
        Token {
            kind: Identifier,
            span: "LDR",
            range: 41..44,
        },
        Token {
            kind: Whitespace,
            span: " ",
            range: 44..45,
        },
        Token {
            kind: Identifier,
            span: "R12",
            range: 45..48,
        },
        Token {
            kind: Comma,
            span: ",",
            range: 48..49,
        },
        Token {
            kind: Whitespace,
            span: " ",
            range: 49..50,
        },
        Token {
            kind: OpenBracket,
            span: "[",
            range: 50..51,
        },
        Token {
            kind: Identifier,
            span: "R13",
            range: 51..54,
        },
        Token {
            kind: Comma,
            span: ",",
            range: 54..55,
        },
        Token {
            kind: Whitespace,
            span: " ",
            range: 55..56,
        },
        Token {
            kind: LiteralSign,
            span: "#",
            range: 56..57,
        },
        Token {
            kind: HyphenMinus,
            span: "-",
            range: 57..58,
        },
        Token {
            kind: Decimal,
            span: "4",
            range: 58..59,
        },
        Token {
            kind: CloseBracket,
            span: "]",
            range: 59..60,
        },
    ];

    assert_eq!(tokens, expected)
}
