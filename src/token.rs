#![allow(dead_code)]

use bytes::Bytes;

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
    UnExpectedToken { ch: char, line: u32 },
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
            Token::UnExpectedToken { ch, line } => f.write_fmt(format_args!(
                "[line {line}] Error: Unexpected character: {ch}"
            )),
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
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if self.reached_eof {
            return None;
        }
        if self.remaining.len() == 0 {
            self.reached_eof = true;
            return Some(Token::EOF);
        }

        let Some(slice) = self.next_byte() else {
            self.reached_eof = true;
            return Some(Token::EOF);
        };
        // println!(">> found next_byte, slice: {:?}", slice);
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
            b"*" => {
                self.remaining = self.remaining.slice(1..);
                Some(Token::STAR)
            }
            b"." => {
                self.remaining = self.remaining.slice(1..);
                Some(Token::DOT)
            }
            b"," => {
                self.remaining = self.remaining.slice(1..);
                Some(Token::COMMA)
            }
            b"+" => {
                self.remaining = self.remaining.slice(1..);
                Some(Token::PLUS)
            }
            b"-" => {
                self.remaining = self.remaining.slice(1..);
                Some(Token::MINUS)
            }
            b";" => {
                self.remaining = self.remaining.slice(1..);
                Some(Token::SEMICOLON)
            }
            b"=" => {
                let peeked_token = self.peek_token();
                // if let None = peeked_token {}

                let bytes = match peeked_token {
                    None => {
                        self.remaining = self.remaining.slice(1..);
                        return Some(Token::EQUAL);
                    }
                    Some(bytes) => bytes,
                };
                if let b"=" = bytes.as_ref() {
                    self.remaining = self.remaining.slice(2..);
                    return Some(Token::EQUALEQUAL);
                }
                self.remaining = self.remaining.slice(1..);
                return Some(Token::EQUAL);
            }
            b"!" => {
                let peeked_token = self.peek_token();
                let bytes = match peeked_token {
                    None => {
                        self.remaining = self.remaining.slice(1..);
                        return Some(Token::BANG);
                    }
                    Some(bytes) => bytes,
                };
                if let b"=" = bytes.as_ref() {
                    self.remaining = self.remaining.slice(2..);
                    return Some(Token::BANGEQUAL);
                }
                self.remaining = self.remaining.slice(1..);
                return Some(Token::BANG);
            }
            b"<" => {
                let peeked_token = self.peek_token();
                let bytes = match peeked_token {
                    None => {
                        self.remaining = self.remaining.slice(1..);
                        return Some(Token::LESS);
                    }
                    Some(bytes) => bytes,
                };
                if let b"=" = bytes.as_ref() {
                    self.remaining = self.remaining.slice(2..);
                    return Some(Token::LESSEQUAL);
                }
                self.remaining = self.remaining.slice(1..);
                return Some(Token::LESS);
            }
            b">" => {
                let peeked_token = self.peek_token();
                let bytes = match peeked_token {
                    None => {
                        self.remaining = self.remaining.slice(1..);
                        return Some(Token::GREATER);
                    }
                    Some(bytes) => bytes,
                };
                if let b"=" = bytes.as_ref() {
                    self.remaining = self.remaining.slice(2..);
                    return Some(Token::GREATEREQUAL);
                }
                self.remaining = self.remaining.slice(1..);
                return Some(Token::GREATER);
            }
            b"/" => {
                let peeked_token = self.peek_token();
                let bytes = match peeked_token {
                    None => {
                        self.remaining = self.remaining.slice(1..);
                        return Some(Token::SLASH);
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
                                return Some(Token::EOF);
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
                    Some(Token::SLASH)
                }
            }
            unexpected => {
                self.remaining = self.remaining.slice(1..);
                let x = unexpected[0] as u32;
                let ch = unsafe { char::from_u32_unchecked(x) };
                let line = self.line;
                Some(Token::UnExpectedToken { ch, line })
            }
        };
        token_to_return
    }
}
