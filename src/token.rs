#![allow(dead_code)]

use std::fmt::Write;

use bytes::Bytes;

pub(crate) enum Token {
    LParen, // `(`
    RParen, // `)`
    LBrace, // `{`
    RBrace, // `}`
    EOF,
}

impl std::fmt::Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str_repr = match self {
            Token::LParen => "LEFT_PAREN ( null",
            Token::RParen => "RIGHT_PAREN ) null",
            Token::LBrace => "LEFT_BRACE { null",
            Token::RBrace => "RIGHT_BRACE } null",
            Token::EOF => "EOF  null",
        };
        f.write_str(&str_repr)
    }
}

// impl Token {
//     pub(crate) fn to_string(&self) -> String {
//         match self {
//             Token::LParen => "LEFT_PAREN ( null".to_owned(),
//             Token::RParen => "RIGHT_PAREN ) null".to_owned(),
//             Token::EOF => "EOF  null".to_owned(),
//         }
//     }
// }

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
        }
    }
}

pub(crate) struct TokenIterator {
    remaining: Bytes,
    reached_eof: bool,
}

impl Iterator for TokenIterator {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if self.reached_eof {
            return None;
        }
        if self.remaining.len() == 0 {
            self.reached_eof = true;
            return Some(Token::EOF);
        }

        loop {
            if self.remaining.len() == 0 {
                self.reached_eof = true;
                return Some(Token::EOF);
            }
            let ch = self.remaining.slice(0..1);
            if *ch == *b" " || *ch == *b"\t" {
                self.remaining = self.remaining.slice(1..);
            } else {
                break;
            }
        }
        let slice = self.remaining.slice(0..1);
        let ch = slice.as_ref();
        let token_to_return = match ch {
            b"(" => {
                self.remaining = self.remaining.slice(1..);
                Some(Token::LParen)
            }
            b")" => {
                self.remaining = self.remaining.slice(1..);
                Some(Token::RParen)
            }
            b"{" => {
                self.remaining = self.remaining.slice(1..);
                Some(Token::LBrace)
            }
            b"}" => {
                self.remaining = self.remaining.slice(1..);
                Some(Token::RBrace)
            }
            _ => unimplemented!(),
        };
        token_to_return
    }
}
