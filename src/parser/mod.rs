#![allow(dead_code)]

use std::rc::Rc;

use expression::{
    CallExpression, Expression, FunctionExpression, IfStatement, Precedence, Statement,
    VarDeclaration, WhileLoop,
};

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
    TooManyArguments {
        at: Token,
    },
    UnmatchedParentheses,
    InvalidAssignmentTarget,
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
            ParseError::InvalidAssignmentTarget => {
                write!(f, "Error at '=': Invalid assignment target.")
            }
            ParseError::TooManyArguments { at } => {
                write!(f, "Error at '{at}': Can't have more than 255 arguments.")
            }
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
        if should_forward_peek_token {
            // TODO: remove unwraps
            self.peek_token = self._token_iterator.next().unwrap().unwrap();
        } else {
            self.peek_token = Token::EOF;
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
        self.advance_token();
        let right_expr = self.parse_expression(operator.get_precedence())?;
        Ok(Expression::InfixExpression {
            operator: operator,
            left_expr: Box::new(left_expr),
            right_expr: Box::new(right_expr),
        })
    }

    #[allow(unused_variables)]
    fn parse_call_expression(&mut self, left_expr: Expression) -> ParseResult<Expression> {
        self.advance_token();
        let mut args: Vec<Expression> = vec![];
        loop {
            if let Token::RParen = &self.curr_token {
                // self.advance_token();
                break;
            }
            if args.len() == 255 {
                return Err(ParseError::TooManyArguments {
                    at: self.curr_token.clone(),
                });
            }
            let arg = self.parse_expression(Precedence::Lowest)?;
            self.advance_token();

            match &self.curr_token {
                Token::RParen => (),
                Token::COMMA => {
                    self.advance_token();
                    // TODO: Here I should make sure the arguments list is not ending with `comma`.
                }
                token => {
                    return Err(ParseError::ExpectedTokenNotFound {
                        expected: "expression",
                        got: token.clone(),
                        line: self.get_curr_line(),
                    })
                }
            }
            args.push(arg);
        }
        let args = match args.len() {
            0 => None,
            _ => Some(args),
        };
        Ok(Expression::Call(CallExpression {
            arguments: args,
            callee: Box::new(left_expr),
        }))
    }

    fn parse_function_expression(&mut self) -> ParseResult<Expression> {
        self.advance_token();
        let name: Option<Token>;
        if let Token::Identifier(_) = &self.curr_token {
            name = Some(self.curr_token.clone());
            self.advance_token();
        } else {
            name = None;
        }
        match &self.curr_token {
            Token::LParen => (),
            token => {
                return Err(ParseError::ExpectedTokenNotFound {
                    expected: "(",
                    got: token.clone(),
                    line: self.get_curr_line(),
                })
            }
        };
        self.advance_token();

        let mut params: Vec<Token> = Vec::new();

        loop {
            if let Token::RParen = &self.curr_token {
                self.advance_token();
                break;
            }

            let name_token = match &self.curr_token {
                Token::Identifier(_) => self.curr_token.clone(),
                t => {
                    return Err(ParseError::ExpectedTokenNotFound {
                        expected: "identifier",
                        got: t.clone(),
                        line: self.get_curr_line(),
                    })
                }
            };
            self.advance_token();

            match &self.curr_token {
                Token::RParen => (),
                Token::COMMA => match &self.peek_token {
                    Token::Identifier(_) => self.advance_token(),
                    t => {
                        return Err(ParseError::ExpectedTokenNotFound {
                            expected: "identifier",
                            got: t.clone(),
                            line: self.get_curr_line(),
                        })
                    }
                },
                t => {
                    return Err(ParseError::ExpectedTokenNotFound {
                        expected: "identifier",
                        got: t.clone(),
                        line: self.get_curr_line(),
                    })
                }
            }
            params.push(name_token);
        }
        let body = match &self.curr_token {
            Token::LBrace => self.parse_block_statement(true)?,
            token => {
                return Err(ParseError::ExpectedTokenNotFound {
                    expected: "{",
                    got: token.clone(),
                    line: self.get_curr_line(),
                })
            }
        };
        let params = match params.len() {
            0 => None,
            _ => Some(params),
        };
        let stmts = if let Statement::Block(stmts) = body {
            stmts
        } else {
            unreachable!();
        };
        Ok(Expression::Function(Rc::new(FunctionExpression {
            body: stmts,
            parameters: params,
            name,
        })))
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
            Token::Identifier(ident_bytes) => Expression::Ident(ident_bytes.clone()),
            Token::Print => {
                self.advance_token();
                let expr = self.parse_expression(precendence.clone())?;
                Expression::Print(Box::new(expr))
            }
            Token::Fun => {
                let fn_expr = self.parse_function_expression()?;
                fn_expr
            }
            Token::Nil => Expression::NilLiteral,
            t => {
                return Err(ParseError::ExpectedTokenNotFound {
                    expected: "expression",
                    got: t,
                    line: self._token_iterator.get_curr_line(),
                })
            }
        };

        loop {
            if let Token::EOF = self.peek_token {
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
                | Token::And
                | Token::Or
                | Token::EQUALEQUAL
                | Token::BANGEQUAL => {
                    self.advance_token();
                    let expr = self.parse_infix_operator_expression(left_expr)?;
                    expr
                }
                Token::LParen => {
                    self.advance_token();
                    self.parse_call_expression(left_expr)?
                }
                Token::EQUAL => {
                    // NOTE:  assignment is different from other infix operators as this is right associative.
                    self.advance_token();
                    self.parse_assignment_infix_expression(left_expr)?
                }
                t => unimplemented!("unimplemented for token: {}", t),
            }
        }

        Ok(left_expr)
    }

    fn parse_assignment_infix_expression(
        &mut self,
        left_expr: Expression,
    ) -> ParseResult<Expression> {
        self.advance_token();
        match &left_expr {
            Expression::Ident(_) => (),
            _ => return Err(ParseError::InvalidAssignmentTarget),
        };
        let right_expr = self.parse_expression(Precedence::Lowest)?;

        Ok(Expression::InfixExpression {
            operator: Token::EQUAL,
            left_expr: Box::new(left_expr),
            right_expr: Box::new(right_expr),
        })
    }

    fn ensure_semicolon_at_statement_end(&mut self) -> Result<(), ParseError> {
        match &self.peek_token {
            Token::SEMICOLON => {
                self.advance_token();
                Ok(())
            }
            _ => {
                return Err(ParseError::ExpectedTokenNotFound {
                    expected: ";",
                    got: self.peek_token.clone(),
                    line: self.get_curr_line(),
                });
            }
        }
    }

    fn parse_var_declaration(&mut self) -> Result<Statement, ParseError> {
        self.advance_token();
        let ident_bytes = match self.curr_token.clone() {
            Token::Identifier(iden_bytes) => iden_bytes,
            token => {
                return Err(ParseError::ExpectedTokenNotFound {
                    expected: "Identifier",
                    got: token,
                    line: self.get_curr_line(),
                })
            }
        };
        match self.peek_token.clone() {
            Token::SEMICOLON => Ok(Statement::VarDeclaration(VarDeclaration {
                identifier: ident_bytes,
                expr: None,
            })),
            Token::EQUAL => {
                self.advance_token();
                self.advance_token();
                let expr = self.parse_expression(Precedence::Lowest)?;
                Ok(Statement::VarDeclaration(VarDeclaration {
                    identifier: ident_bytes,
                    expr: Some(expr),
                }))
            }
            token => Err(ParseError::ExpectedTokenNotFound {
                expected: "expression",
                got: token,
                line: self.get_curr_line(),
            }),
        }
    }

    fn parse_if_statement(&mut self) -> Result<Statement, ParseError> {
        self.advance_token();
        let expr = self.parse_expression(Precedence::Lowest)?;
        self.advance_token();
        let if_block = self.parse_statement()?;
        let mut else_block = None;
        if let Token::Else = self.curr_token {
            self.advance_token();
            else_block = Some(self.parse_statement()?);
        }
        Ok(Statement::IfStatement(Box::new(IfStatement {
            else_block,
            expr,
            if_block,
        })))
    }

    fn parse_return_statement(&mut self) -> Result<Statement, ParseError> {
        self.advance_token();
        let expr = self.parse_expression(Precedence::Lowest)?;
        return Ok(Statement::Return(expr));
    }
    fn parse_while_statement(&mut self) -> Result<Statement, ParseError> {
        self.advance_token();
        let expr: Option<Expression>;
        if let Token::LBrace = &self.curr_token {
            expr = None
        } else {
            expr = Some(self.parse_expression(Precedence::Lowest)?);
            self.advance_token();
        }

        let stmt = self.parse_statement()?;
        Ok(Statement::WhileLoop(WhileLoop {
            expr,
            block: Box::new(stmt),
        }))
    }

    fn parse_for_statement_and_desugar_it(&mut self) -> Result<Statement, ParseError> {
        self.advance_token();
        let var_declaration: Option<Statement>;
        let conditional_expr: Option<Expression>;
        let incr_stmt: Option<Statement>;
        let block_body: Statement;
        if let Token::SEMICOLON = self.curr_token {
            self.advance_token();
            var_declaration = None;
        } else {
            let stmt = self.parse_statement()?;
            var_declaration = Some(stmt);
        }

        if let Token::SEMICOLON = self.curr_token {
            conditional_expr = None;
        } else {
            conditional_expr = Some(self.parse_expression(Precedence::Lowest)?);
            self.advance_token();
        }
        self.advance_token();
        if let Token::LBrace = self.curr_token {
            block_body = self.parse_statement()?;
            incr_stmt = None;
        } else {
            incr_stmt = Some(self.parse_single_statement_without_semicolon()?);
            self.advance_token();
            block_body = self.parse_statement()?;
        }

        let mut final_block_stmts = Vec::<Statement>::new();
        if let Some(v) = var_declaration {
            final_block_stmts.push(v);
        }

        let mut while_block_body: Vec<Statement> = vec![block_body];

        if let Some(v) = incr_stmt {
            while_block_body.push(v);
        }
        let while_loop = Statement::WhileLoop(WhileLoop {
            expr: conditional_expr,
            block: Box::new(Statement::Block(while_block_body)),
        });
        final_block_stmts.push(while_loop);
        Ok(Statement::Block(final_block_stmts))
    }

    fn parse_single_statement_without_semicolon(&mut self) -> ParseResult<Statement> {
        let stmt = match &self.curr_token {
            Token::Print => {
                self.advance_token();
                let expr = self.parse_expression(Precedence::Lowest)?;
                Statement::Print(expr)
            }
            Token::Var => self.parse_var_declaration()?,
            Token::If => return self.parse_if_statement(),
            Token::While => return self.parse_while_statement(),
            Token::For => self.parse_for_statement_and_desugar_it()?,
            Token::Return => self.parse_return_statement()?,
            _ => Statement::Expression(self.parse_expression(Precedence::Lowest)?),
        };
        Ok(stmt)
    }

    fn parse_single_statement(&mut self) -> Result<Statement, ParseError> {
        let stmt = self.parse_single_statement_without_semicolon()?;
        // println!("{stmt:?}");
        match &stmt {
            Statement::IfStatement(_) | Statement::WhileLoop(_) | Statement::Block(_) => {
                return Ok(stmt)
            }
            Statement::Expression(Expression::Function(_)) => {
                self.advance_token();
                return Ok(stmt);
            }
            _ => {
                self.ensure_semicolon_at_statement_end()?;
                self.advance_token();
                Ok(stmt)
            }
        }
    }

    fn parse_block_statement(
        &mut self,
        is_block_part_of_expression: bool,
    ) -> Result<Statement, ParseError> {
        self.advance_token();
        let mut stms: Vec<Statement> = vec![];
        loop {
            if let Token::RBrace = self.curr_token {
                if !is_block_part_of_expression {
                    self.advance_token();
                }
                break;
            }
            let stmt = self.parse_statement()?;
            stms.push(stmt);
        }
        Ok(Statement::Block(stms))
    }

    fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        let stmt = match &self.curr_token {
            Token::LBrace => self.parse_block_statement(false)?,
            _ => self.parse_single_statement()?,
        };
        Ok(stmt)
    }

    pub(crate) fn parse_program(&mut self) -> Result<Vec<Statement>, ParseError> {
        let mut statements = vec![];
        loop {
            if let Token::EOF = self.curr_token {
                break;
            }
            let stmt = self.parse_statement()?;
            statements.push(stmt);
        }
        Ok(statements)
    }
}
