#![allow(dead_code)]

use core::str;
use std::{i64, str::FromStr};

use bytes::Bytes;

pub enum LexicalError {
    UnExpectedToken { ch: char, line: u32 }, // Error token.
    UnterminatedString { line: u32 },        // Error Token.
}

impl std::fmt::Debug for LexicalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LexicalError::UnExpectedToken { ch, line } => f.write_fmt(format_args!(
                "[line {line}] Error: Unexpected character: {ch}"
            )),
            LexicalError::UnterminatedString { line } => {
                f.write_fmt(format_args!("[line {line}] Error: Unterminated string."))
            }
        }
    }
}

// enum Number {
//     Whole(i64),
//     Float(f64),
// }

// impl std::fmt::Debug for Number {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             Number::Whole(n) => f.write_fmt(format_args!("{:?}", n)),
//             Number::Float(n) => f.write_fmt(format_args!("{:?}", n)),
//         }
//     }
// }

// impl std::fmt::Display for Number {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             Number::Whole(n) => f.write_fmt(format_args!("{}", n)),
//             Number::Float(n) => f.write_fmt(format_args!("{}", n)),
//         }
//     }
// }

pub(crate) enum Token {
    LParen, // `(`
    RParen, // `)`
    LBrace, // `{`
    RBrace, // `}`
    STAR,   //  `*`
    DOT,    // `.`
    COMMA,  // `,`
    PLUS,   // `+`
    MINUS,  // `-`
    SLASH,  // `/`
    COMMENT(Bytes),
    SEMICOLON,    // `;`
    EQUAL,        // =
    EQUALEQUAL,   // ==
    BANG,         // !
    BANGEQUAL,    // !=
    LESS,         // <
    LESSEQUAL,    // <=
    GREATER,      // >
    GREATEREQUAL, // >=
    StringLiteral(String),
    NumberLiteral(f64),
    EOF,
}

impl std::fmt::Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::LParen => f.write_str("LEFT_PAREN ( null"),
            Token::RParen => f.write_str("RIGHT_PAREN ) null"),
            Token::LBrace => f.write_str("LEFT_BRACE { null"),
            Token::RBrace => f.write_str("RIGHT_BRACE } null"),
            Token::STAR => f.write_str("STAR * null"),
            Token::DOT => f.write_str("DOT . null"),
            Token::COMMA => f.write_str("COMMA , null"),
            Token::PLUS => f.write_str("PLUS + null"),
            Token::MINUS => f.write_str("MINUS - null"),
            Token::SEMICOLON => f.write_str("SEMICOLON ; null"),
            Token::EQUAL => f.write_str("EQUAL = null"),
            Token::EQUALEQUAL => f.write_str("EQUAL_EQUAL == null"),
            Token::BANG => f.write_str("BANG ! null"),
            Token::BANGEQUAL => f.write_str("BANG_EQUAL != null"),
            Token::LESS => f.write_str("LESS < null"),
            Token::LESSEQUAL => f.write_str("LESS_EQUAL <= null"),
            Token::GREATER => f.write_str("GREATER > null"),
            Token::GREATEREQUAL => f.write_str("GREATER_EQUAL >= null"),
            Token::SLASH => f.write_str("SLASH / null"),
            Token::COMMENT(_) => f.write_str("COMMENT  null"),
            Token::StringLiteral(s) => f.write_fmt(format_args!("STRING \"{s}\" {s}")),
            Token::NumberLiteral(number) => {
                f.write_fmt(format_args!("NUMBER {} {:?}", number, number))
            }
            Token::EOF => f.write_str("EOF  null"),
        }
    }
}

pub(crate) struct Scanner {
    _source: Bytes,
}

impl Scanner {
    pub(crate) fn new(source: String) -> Self {
        Self {
            _source: Bytes::from(source),
        }
    }

    pub(crate) fn iter(&self) -> TokenIterator {
        TokenIterator {
            remaining: self._source.clone(),
            reached_eof: false,
            line: 1,
        }
    }
}

pub(crate) struct TokenIterator {
    remaining: Bytes,
    reached_eof: bool,
    line: u32,
}

impl TokenIterator {
    fn skip_whitespaces(&mut self) {
        loop {
            if self.remaining.len() == 0 {
                return;
            }
            let ch = self.remaining.slice(0..1);
            if *ch == *b"\n" {
                self.line += 1;
                self.remaining = self.remaining.slice(1..);
                continue;
            }
            if *ch == *b" " || *ch == *b"\t" {
                self.remaining = self.remaining.slice(1..);
            } else {
                break;
            }
        }
    }
    fn next_byte(&mut self) -> Option<Bytes> {
        self.skip_whitespaces();
        if self.remaining.len() == 0 {
            return None;
        }
        Some(self.remaining.slice(0..1))
    }

    fn peek_token(&self) -> Option<Bytes> {
        if self.remaining.len() == 1 {
            return None;
        }
        Some(self.remaining.slice(1..2))
    }
}

impl Iterator for TokenIterator {
    type Item = Result<Token, LexicalError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.reached_eof {
            return None;
        }
        if self.remaining.len() == 0 {
            self.reached_eof = true;
            return Some(Ok(Token::EOF));
        }

        let Some(slice) = self.next_byte() else {
            self.reached_eof = true;
            return Some(Ok(Token::EOF));
        };
        // println!(">> found next_byte, slice: {:?}", slice);
        let ch = slice.as_ref();
        let token_to_return = match ch {
            b"(" => {
                self.remaining = self.remaining.slice(1..);
                Some(Ok(Token::LParen))
            }
            b")" => {
                self.remaining = self.remaining.slice(1..);
                Some(Ok(Token::RParen))
            }
            b"{" => {
                self.remaining = self.remaining.slice(1..);
                Some(Ok(Token::LBrace))
            }
            b"}" => {
                self.remaining = self.remaining.slice(1..);
                Some(Ok(Token::RBrace))
            }
            b"*" => {
                self.remaining = self.remaining.slice(1..);
                Some(Ok(Token::STAR))
            }
            b"." => {
                self.remaining = self.remaining.slice(1..);
                Some(Ok(Token::DOT))
            }
            b"," => {
                self.remaining = self.remaining.slice(1..);
                Some(Ok(Token::COMMA))
            }
            b"+" => {
                self.remaining = self.remaining.slice(1..);
                Some(Ok(Token::PLUS))
            }
            b"-" => {
                self.remaining = self.remaining.slice(1..);
                Some(Ok(Token::MINUS))
            }
            b";" => {
                self.remaining = self.remaining.slice(1..);
                Some(Ok(Token::SEMICOLON))
            }
            b"=" => {
                let peeked_token = self.peek_token();
                // if let None = peeked_token {}

                let bytes = match peeked_token {
                    None => {
                        self.remaining = self.remaining.slice(1..);
                        return Some(Ok(Token::EQUAL));
                    }
                    Some(bytes) => bytes,
                };
                if let b"=" = bytes.as_ref() {
                    self.remaining = self.remaining.slice(2..);
                    return Some(Ok(Token::EQUALEQUAL));
                }
                self.remaining = self.remaining.slice(1..);
                return Some(Ok(Token::EQUAL));
            }
            b"!" => {
                let peeked_token = self.peek_token();
                let bytes = match peeked_token {
                    None => {
                        self.remaining = self.remaining.slice(1..);
                        return Some(Ok(Token::BANG));
                    }
                    Some(bytes) => bytes,
                };
                if let b"=" = bytes.as_ref() {
                    self.remaining = self.remaining.slice(2..);
                    return Some(Ok(Token::BANGEQUAL));
                }
                self.remaining = self.remaining.slice(1..);
                return Some(Ok(Token::BANG));
            }
            b"<" => {
                let peeked_token = self.peek_token();
                let bytes = match peeked_token {
                    None => {
                        self.remaining = self.remaining.slice(1..);
                        return Some(Ok(Token::LESS));
                    }
                    Some(bytes) => bytes,
                };
                if let b"=" = bytes.as_ref() {
                    self.remaining = self.remaining.slice(2..);
                    return Some(Ok(Token::LESSEQUAL));
                }
                self.remaining = self.remaining.slice(1..);
                return Some(Ok(Token::LESS));
            }
            b">" => {
                let peeked_token = self.peek_token();
                let bytes = match peeked_token {
                    None => {
                        self.remaining = self.remaining.slice(1..);
                        return Some(Ok(Token::GREATER));
                    }
                    Some(bytes) => bytes,
                };
                if let b"=" = bytes.as_ref() {
                    self.remaining = self.remaining.slice(2..);
                    return Some(Ok(Token::GREATEREQUAL));
                }
                self.remaining = self.remaining.slice(1..);
                return Some(Ok(Token::GREATER));
            }
            b"/" => {
                let peeked_token = self.peek_token();
                let bytes = match peeked_token {
                    None => {
                        self.remaining = self.remaining.slice(1..);
                        return Some(Ok(Token::SLASH));
                    }
                    Some(bytes) => bytes,
                };
                if let b"/" = bytes.as_ref() {
                    self.remaining = self.remaining.slice(2..);
                    loop {
                        let peeked_token = self.peek_token();
                        let bytes = match peeked_token {
                            None => {
                                self.reached_eof = true;
                                return Some(Ok(Token::EOF));
                            }
                            Some(bytes) => bytes,
                        };

                        if let b"\n" = bytes.as_ref() {
                            self.remaining = self.remaining.slice(1..);
                            return self.next();
                        } else {
                            self.remaining = self.remaining.slice(1..);
                        }
                    }
                } else {
                    self.remaining = self.remaining.slice(1..);
                    Some(Ok(Token::SLASH))
                }
            }
            b"\"" => {
                self.remaining = self.remaining.slice(1..);
                let mut size_of_str: usize = 0;
                let remaining_size = self.remaining.len();
                loop {
                    if size_of_str == remaining_size {
                        self.remaining = self.remaining.slice(remaining_size..);
                        return Some(Err(LexicalError::UnterminatedString { line: self.line }));
                    }
                    let x = self.remaining[size_of_str];
                    if *b"\"" == [x] {
                        let bytes = self.remaining.slice(0..size_of_str);
                        // TODO: remove unwrap.
                        let string = String::from_str(str::from_utf8(&bytes).unwrap()).unwrap();
                        self.remaining = self.remaining.slice(size_of_str + 1..);
                        return Some(Ok(Token::StringLiteral(string)));
                    } else {
                        size_of_str += 1;
                    }
                }
            }
            b"0" | b"1" | b"2" | b"3" | b"4" | b"5" | b"6" | b"7" | b"8" | b"9" => {
                let mut digit_count = 1;
                let mut is_float = false;
                loop {
                    if self.remaining.slice(digit_count..).len() == 0 {
                        break;
                    }
                    let ch = self.remaining[digit_count] as char;
                    if ch.is_digit(10) {
                        digit_count += 1;
                    } else if ch == '.' && is_float == false {
                        // TODO: we should probably check that after the ".", there is also some
                        // digit
                        if self.remaining.slice(digit_count + 1..).len() == 0 {
                            break;
                        }
                        let ch = self.remaining[digit_count + 1] as char;
                        if !ch.is_digit(10) {
                            break;
                        } else {
                            digit_count += 1;
                            is_float = true;
                        }
                    } else {
                        break;
                    }
                }
                let number = f64::from_str(
                    std::str::from_utf8(self.remaining.slice(0..digit_count).as_ref()).unwrap(),
                )
                .unwrap();
                self.remaining = self.remaining.slice(digit_count..);
                Some(Ok(Token::NumberLiteral(number)))
                // let token = if is_float {
                // } else {
                //     let number = i64::from_str(
                //         std::str::from_utf8(self.remaining.slice(0..digit_count).as_ref()).unwrap(),
                //     )
                //     .unwrap();
                //     Some(Ok(Token::NumberLiteral(Number::Whole(number))))
                // };
            }
            unexpected => {
                self.remaining = self.remaining.slice(1..);
                let x = unexpected[0] as u32;
                let ch = unsafe { char::from_u32_unchecked(x) };
                let line = self.line;
                Some(Err(LexicalError::UnExpectedToken { ch, line }))
            }
        };
        token_to_return
    }
}
