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
    ExpectedTokenNotFound {
        expected: &'static str,
        got: Token,
        line: u32,
    },
    UnmatchedParentheses,
}

impl std::fmt::Debug for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::EmptySource => write!(f, "EmptySource"),
            ParseError::ImpossibleError => write!(f, "ImpossibleError"),
            ParseError::LexicalError(e) => write!(f, "{:?}", e),
            ParseError::ExpectedTokenNotFound {
                line,
                got,
                expected,
            } => write!(f, "[line {line}] Error at '{got}': expect {expected}"),
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

    pub(crate) fn get_curr_line(&self) -> u32 {
        self._token_iterator.get_curr_line()
    }

    fn advance_token(&mut self) {
        let should_forward_peek_token = if let Token::EOF = self.peek_token {
            false
        } else {
            true
        };
        std::mem::swap(&mut self.curr_token, &mut self.peek_token);
        eprintln!(">>advance_token called: curr_token: {:?}", self.curr_token);
        if should_forward_peek_token {
            // TODO: remove unwraps
            self.peek_token = self._token_iterator.next().unwrap().unwrap();
        }
    }

    fn parse_prefix_grouped_expression(&mut self) -> ParseResult<Expression> {
        if let Token::RParen = self.peek_token {
            return Err(ParseError::ExpectedTokenNotFound {
                expected: "expression",
                got: Token::RParen,
                line: self._token_iterator.get_curr_line(),
            });
        }
        self.advance_token();

        let expr = self.parse_expression(Precedence::Lowest)?;
        let Token::RParen = self.peek_token else {
            return Err(ParseError::UnmatchedParentheses);
        };
        self.advance_token();
        Ok(Expression::GroupedExpression(Box::new(expr)))
    }

    fn parse_prefix_operator_expression(&mut self) -> ParseResult<Expression> {
        let operator = self.curr_token.clone();
        self.advance_token();
        let expression = self.parse_expression(Precedence::Prefix)?;
        Ok(Expression::PrefixExpression {
            operator: operator,
            expr: Box::new(expression),
        })
    }

    fn parse_infix_operator_expression(
        &mut self,
        left_expr: Expression,
    ) -> ParseResult<Expression> {
        let operator = self.curr_token.clone();
        eprintln!(
            "inside parse_infix_operator_expression, left_expr: {:?}, operator: {:?}",
            left_expr, operator
        );
        self.advance_token();
        eprintln!("...token advanced in parse_infix_operator_expression");
        let right_expr = self.parse_expression(operator.get_precedence())?;
        // self.advance_token();
        Ok(Expression::InfixExpression {
            operator: operator,
            left_expr: Box::new(left_expr),
            right_expr: Box::new(right_expr),
        })
    }

    #[allow(unused_variables)]
    pub(crate) fn parse_expression(
        &mut self,
        precendence: Precedence,
    ) -> Result<Expression, ParseError> {
        let mut left_expr = match self.curr_token.clone() {
            Token::True => Expression::BooleanLiteral(true),
            Token::False => Expression::BooleanLiteral(false),
            Token::NumberLiteral(val, _) => Expression::NumberLiteral(val),
            Token::StringLiteral(bytes) => Expression::StringLiteral(bytes.clone()),
            Token::LParen => self.parse_prefix_grouped_expression()?,
            Token::MINUS | Token::BANG => self.parse_prefix_operator_expression()?,
            Token::Nil => {
                self.advance_token();
                Expression::NilLiteral
            }
            t => {
                return Err(ParseError::ExpectedTokenNotFound {
                    expected: "expression",
                    got: t,
                    line: self._token_iterator.get_curr_line(),
                })
            }
        };

        loop {
            eprintln!(
                "curr_token: {curr_token}, precendence: {precendence}",
                curr_token = self.curr_token,
                precendence = self.curr_token.get_precedence().value()
            );
            if let Token::EOF = self.peek_token {
                eprintln!("&&&&&&&&&&&&& found EOF");
                break;
            }
            if precendence.value() >= self.peek_token.get_precedence().value() {
                break;
            }
            left_expr = match self.peek_token.clone() {
                Token::PLUS
                | Token::MINUS
                | Token::SLASH
                | Token::STAR
                | Token::LESS
                | Token::LESSEQUAL
                | Token::GREATER
                | Token::GREATEREQUAL
                | Token::EQUALEQUAL
                | Token::BANGEQUAL => {
                    self.advance_token();
                    let expr = self.parse_infix_operator_expression(left_expr)?;
                    expr
                }
                t => unimplemented!("unimplemented for token: {}", t),
            }
        }

        Ok(left_expr)
    }
}
