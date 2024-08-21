#![allow(dead_code)]

use expression::{Expression, Precedence};

use crate::token::{LexicalError, Scanner, Token, TokenIterator};
pub(crate) mod expression;

pub(crate) struct Parser {
    _scanner: Scanner,
    _token_iterator: TokenIterator,
    curr_token: Token,
    peek_token: Token,
}

#[derive(Debug)]
pub(crate) enum ParseError {
    EmptySource,
    ImpossibleError,
    LexicalError(LexicalError),
}

impl Parser {
    pub(crate) fn from_source(source: String) -> Result<Self, ParseError> {
        let scanner = Scanner::new(source);
        let mut token_iterator = scanner.iter();

        let curr_token = token_iterator
            .next()
            .ok_or(ParseError::EmptySource)?
            .map_err(|e| ParseError::LexicalError(e))?;

        if let Token::EOF = curr_token {
            return Err(ParseError::EmptySource);
        }

        let peek_token = token_iterator
            .next()
            .ok_or_else(|| unreachable!())?
            .map_err(|e| ParseError::LexicalError(e))?;

        Ok(Self {
            _scanner: scanner,
            _token_iterator: token_iterator,
            curr_token,
            peek_token,
        })
    }

    #[allow(unused_variables)]
    pub(crate) fn parse_expression(
        &mut self,
        precendence: Precedence,
    ) -> Result<Expression, ParseError> {
        let expr = match &self.curr_token {
            Token::True => Expression::BooleanLiteral(true),
            Token::False => Expression::BooleanLiteral(false),
            Token::NumberLiteral(val, _) => Expression::NumberLiteral(*val),
            Token::StringLiteral(bytes) => Expression::StringLiteral(bytes.clone()),
            Token::Nil => Expression::NilLiteral,
            _ => unimplemented!(),
        };
        Ok(expr)
    }
}
