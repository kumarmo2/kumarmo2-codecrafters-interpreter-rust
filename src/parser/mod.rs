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

pub(crate) enum ParseError {
    EmptySource,
    ImpossibleError,
    LexicalError(LexicalError),
    ExpectedTokenNotFound { expected: &'static str, got: Token },
    UnmatchedParentheses,
}

impl std::fmt::Debug for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::EmptySource => write!(f, "EmptySource"),
            ParseError::ImpossibleError => write!(f, "ImpossibleError"),
            ParseError::LexicalError(e) => write!(f, "{:?}", e),
            ParseError::ExpectedTokenNotFound { .. } => write!(f, "ExpectedTokenNotFound"),
            ParseError::UnmatchedParentheses => write!(f, "Error: Unmatched parentheses."),
        }
    }
}

type ParseResult<T> = Result<T, ParseError>;

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

    fn advance_token(&mut self) {
        std::mem::swap(&mut self.curr_token, &mut self.peek_token);
        // TODO: remove unwraps
        self.peek_token = self._token_iterator.next().unwrap().unwrap();
    }

    fn parse_prefix_grouped_expression(&mut self) -> ParseResult<Expression> {
        self.advance_token();
        if let Token::RParen = self.curr_token {
            return Err(ParseError::ExpectedTokenNotFound {
                expected: "expression",
                got: Token::RParen,
            });
        }

        let expr = self.parse_expression(Precedence::Lowest)?;
        let Token::RParen = self.peek_token else {
            return Err(ParseError::UnmatchedParentheses);
        };
        Ok(Expression::GroupedExpression(Box::new(expr)))
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
            Token::LParen => self.parse_prefix_grouped_expression()?,
            Token::Nil => Expression::NilLiteral,
            _ => unimplemented!(),
        };
        Ok(expr)
    }
}
