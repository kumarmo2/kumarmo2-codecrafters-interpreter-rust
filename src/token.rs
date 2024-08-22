#![allow(dead_code)]

use core::str;
use std::{collections::HashMap, str::FromStr};

use bytes::Bytes;
use lazy_static::lazy_static;

use crate::parser::expression::Precedence;

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

lazy_static! {
    pub(crate) static ref KEYWORDS: HashMap<&'static str, Token> = {
        let mut m = HashMap::new();
        m.insert("and", Token::And);
        m.insert("class", Token::Class);
        m.insert("else", Token::Else);
        m.insert("false", Token::False);
        m.insert("for", Token::For);
        m.insert("fun", Token::Fun);
        m.insert("if", Token::If);
        m.insert("nil", Token::Nil);
        m.insert("or", Token::Or);
        m.insert("print", Token::Print);
        m.insert("return", Token::Return);
        m.insert("super", Token::Super);
        m.insert("this", Token::This);
        m.insert("true", Token::True);
        m.insert("var", Token::Var);
        m.insert("while", Token::While);
        m
    };
}

#[derive(Clone)]
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
    StringLiteral(Bytes),
    NumberLiteral(f64, Bytes),
    Identifier(Bytes),
    And,
    Class,
    Else,
    False,
    For,
    Fun,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,
    EOF,
}

impl Token {
    pub(crate) fn get_precedence(&self) -> Precedence {
        match self {
            Token::PLUS | Token::MINUS => Precedence::Sum,
            Token::SLASH | Token::STAR => Precedence::Product,
            Token::LESS | Token::GREATER | Token::LESSEQUAL | Token::GREATEREQUAL => {
                Precedence::LessGreater
            }
            Token::BANGEQUAL | Token::EQUALEQUAL => Precedence::Equals,
            _ => Precedence::Lowest,
        }
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::LParen => f.write_str("("),
            Token::RParen => f.write_str(")"),
            Token::LBrace => f.write_str("{"),
            Token::RBrace => f.write_str("}"),
            Token::STAR => f.write_str("*"),
            Token::DOT => f.write_str("."),
            Token::COMMA => f.write_str(","),
            Token::PLUS => f.write_str("+"),
            Token::MINUS => f.write_str("-"),
            Token::SEMICOLON => f.write_str(";"),
            Token::EQUAL => f.write_str("="),
            Token::EQUALEQUAL => f.write_str("=="),
            Token::BANG => f.write_str("!"),
            Token::BANGEQUAL => f.write_str("!="),
            Token::LESS => f.write_str("<"),
            Token::LESSEQUAL => f.write_str("<="),
            Token::GREATER => f.write_str(">"),
            Token::GREATEREQUAL => f.write_str(">="),
            Token::SLASH => f.write_str("/"),
            Token::COMMENT(_) => unimplemented!("Will not display comment"),
            Token::StringLiteral(s) => {
                // TODO: remove unsafe
                let string =
                    unsafe { String::from_str(std::str::from_utf8_unchecked(&s)).unwrap() };
                f.write_fmt(format_args!("{string}"))
            }
            Token::NumberLiteral(number, _) => f.write_fmt(format_args!("{}", number)),
            Token::Identifier(identifier_bytes) => f.write_fmt(format_args!(
                // TODO: remove unwraps.
                "{}",
                String::from_str(std::str::from_utf8(identifier_bytes.as_ref()).unwrap()).unwrap()
            )),
            Token::And => f.write_str("and"),
            Token::Class => f.write_str("class"),
            Token::Else => f.write_str("else"),
            Token::False => f.write_str("false"),
            Token::For => f.write_str("for"),
            Token::Fun => f.write_str("fun"),
            Token::If => f.write_str("if"),
            Token::Nil => f.write_str("nil"),
            Token::Or => f.write_str("or"),
            Token::Print => f.write_str("print"),
            Token::Return => f.write_str("return"),
            Token::Super => f.write_str("super"),
            Token::This => f.write_str("this"),
            Token::True => f.write_str("true"),
            Token::Var => f.write_str("var"),
            Token::While => f.write_str("while"),
            Token::EOF => f.write_str(""),
        }
    }
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
            Token::StringLiteral(s) => {
                // TODO: remove unsafe
                let string =
                    unsafe { String::from_str(std::str::from_utf8_unchecked(&s)).unwrap() };
                f.write_fmt(format_args!("STRING \"{string}\" {string}"))
            }
            Token::NumberLiteral(number, bytes) => f.write_fmt(format_args!(
                "NUMBER {} {:?}",
                String::from_str(std::str::from_utf8(bytes.as_ref()).unwrap()).unwrap(),
                number
            )),
            Token::Identifier(identifier_bytes) => f.write_fmt(format_args!(
                // TODO: remove unwraps.
                "IDENTIFIER {} null",
                String::from_str(std::str::from_utf8(identifier_bytes.as_ref()).unwrap()).unwrap()
            )),
            Token::And => f.write_str("AND and null"),
            Token::Class => f.write_str("CLASS class null"),
            Token::Else => f.write_str("ELSE else null"),
            Token::False => f.write_str("FALSE false null"),
            Token::For => f.write_str("FOR for null"),
            Token::Fun => f.write_str("FUN fun null"),
            Token::If => f.write_str("IF if null"),
            Token::Nil => f.write_str("NIL nil null"),
            Token::Or => f.write_str("OR or null"),
            Token::Print => f.write_str("PRINT print null"),
            Token::Return => f.write_str("RETURN return null"),
            Token::Super => f.write_str("SUPER super null"),
            Token::This => f.write_str("THIS this null"),
            Token::True => f.write_str("TRUE true null"),
            Token::Var => f.write_str("VAR var null"),
            Token::While => f.write_str("WHILE while null"),
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
    pub(crate) fn get_curr_line(&self) -> u32 {
        self.line
    }
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
        let ch = slice[0] as char;
        let token_to_return = match ch {
            '(' => {
                self.remaining = self.remaining.slice(1..);
                Some(Ok(Token::LParen))
            }
            ')' => {
                self.remaining = self.remaining.slice(1..);
                Some(Ok(Token::RParen))
            }
            '{' => {
                self.remaining = self.remaining.slice(1..);
                Some(Ok(Token::LBrace))
            }
            '}' => {
                self.remaining = self.remaining.slice(1..);
                Some(Ok(Token::RBrace))
            }
            '*' => {
                self.remaining = self.remaining.slice(1..);
                Some(Ok(Token::STAR))
            }
            '.' => {
                self.remaining = self.remaining.slice(1..);
                Some(Ok(Token::DOT))
            }
            ',' => {
                self.remaining = self.remaining.slice(1..);
                Some(Ok(Token::COMMA))
            }
            '+' => {
                self.remaining = self.remaining.slice(1..);
                Some(Ok(Token::PLUS))
            }
            '-' => {
                self.remaining = self.remaining.slice(1..);
                Some(Ok(Token::MINUS))
            }
            ';' => {
                self.remaining = self.remaining.slice(1..);
                Some(Ok(Token::SEMICOLON))
            }
            '=' => {
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
            '!' => {
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
            '<' => {
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
            '>' => {
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
            '/' => {
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
            '\"' => {
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
                        // TODO: remove unwrap and unsafe
                        self.remaining = self.remaining.slice(size_of_str + 1..);
                        return Some(Ok(Token::StringLiteral(bytes.clone())));
                    } else {
                        size_of_str += 1;
                    }
                }
            }
            '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
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
                let bytes = self.remaining.slice(0..digit_count);
                let number = f64::from_str(std::str::from_utf8(bytes.as_ref()).unwrap()).unwrap();
                self.remaining = self.remaining.slice(digit_count..);
                Some(Ok(Token::NumberLiteral(number, bytes)))
            }
            ch if ch.is_alphabetic() || ch == '_' => {
                let mut identifier_len = 1;
                loop {
                    if self.remaining.slice(identifier_len..).len() == 0 {
                        break;
                    }
                    let ch = self.remaining[identifier_len] as char;
                    if ch.is_alphanumeric() || ch == '_' {
                        identifier_len += 1;
                    } else {
                        break;
                    }
                }
                let bytes = self.remaining.slice(0..identifier_len);
                let utf_bytes = unsafe { std::str::from_utf8_unchecked(bytes.as_ref()) };
                let token = if KEYWORDS.contains_key(utf_bytes) {
                    let x = KEYWORDS.get(utf_bytes).unwrap().clone();
                    Some(Ok(x))
                } else {
                    Some(Ok(Token::Identifier(bytes)))
                };
                self.remaining = self.remaining.slice(identifier_len..);
                token
            }
            unexpected => {
                self.remaining = self.remaining.slice(1..);
                let ch = unexpected;
                let line = self.line;
                Some(Err(LexicalError::UnExpectedToken { ch, line }))
            }
        };
        token_to_return
    }
}
