#![allow(dead_code, unused_variables)]
use std::{borrow::Borrow, io::Read, string};

use bytes::{Buf, BufMut, Bytes, BytesMut};

use crate::{
    parser::{
        expression::{Expression, Precedence},
        ParseError, Parser,
    },
    token::{self, Token},
};

#[derive(Clone)]
pub(crate) enum Object {
    Number(f64),
    Boolean(bool),
    String(Bytes),
    Nil,
}

impl Object {
    pub(crate) fn get_truthy_value(&self) -> bool {
        match self {
            Object::Number(_) => true,
            Object::Boolean(v) => *v,
            Object::String(_) => true,
            Object::Nil => false,
        }
    }
}

impl std::fmt::Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::Number(v) => write!(f, "{}", v),
            Object::Boolean(v) => write!(f, "{}", v),
            Object::Nil => write!(f, "nil"),
            Object::String(bytes) => {
                let str = unsafe { std::str::from_utf8_unchecked(bytes.as_ref()) };
                write!(f, "{}", str)
            }
        }
    }
}

pub(crate) struct Interpreter {
    parser: Parser,
}

pub(crate) enum EvaluationError {
    ParseError(ParseError),
    ExpectedSomethingButGotOther {
        expected: &'static str,
        got: Object,
    },
    Adhoc(String),
    InvalidOperation {
        left: Object,
        operator: Token,
        right: Object,
    },
}

impl std::fmt::Debug for EvaluationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvaluationError::ParseError(e) => write!(f, "{:?}", e),
            EvaluationError::ExpectedSomethingButGotOther { expected, got } => {
                write!(f, "expected: {expected}, but got: {got}")
            }
            EvaluationError::InvalidOperation {
                left,
                operator,
                right,
            } => write!(
                f,
                "InvalidOperation: {operator}, left: {left}, right: {right}"
            ),
            EvaluationError::Adhoc(str) => write!(f, "{str}"),
        }
    }
}

impl Interpreter {
    pub(crate) fn from_source(source: String) -> Result<Self, ParseError> {
        let parser = Parser::from_source(source)?;
        Ok(Self { parser })
    }

    fn evaluate_string_infix_operation(
        operator: Token,
        left: &Bytes,
        right: &Bytes,
    ) -> Result<Object, EvaluationError> {
        match operator.clone() {
            Token::PLUS => {
                let mut buf = BytesMut::with_capacity(left.len() + right.len());
                buf.put(left.as_ref());
                buf.put(right.as_ref());
                let bytes = buf.freeze();
                Ok(Object::String(bytes))
            }
            Token::EQUALEQUAL => {
                let left = unsafe { std::str::from_utf8_unchecked(left.as_ref()) };
                let right = unsafe { std::str::from_utf8_unchecked(right.as_ref()) };
                Ok(Object::Boolean(left == right))
            }
            Token::BANGEQUAL => {
                let left = unsafe { std::str::from_utf8_unchecked(left.as_ref()) };
                let right = unsafe { std::str::from_utf8_unchecked(right.as_ref()) };
                Ok(Object::Boolean(left != right))
            }
            token => Err(EvaluationError::InvalidOperation {
                left: Object::String(left.clone()),
                operator,
                right: Object::String(right.clone()),
            }),
        }
    }

    fn evaluate_numeric_infix_operation(
        operator: Token,
        left_value: f64,
        right_value: f64,
    ) -> Object {
        match operator {
            Token::STAR => Object::Number(left_value * right_value),
            Token::SLASH => Object::Number(left_value / right_value),
            Token::PLUS => Object::Number(left_value + right_value),
            Token::MINUS => Object::Number(left_value - right_value),
            Token::EQUALEQUAL => Object::Boolean(left_value == right_value),
            Token::BANGEQUAL => Object::Boolean(left_value != right_value),
            Token::LESS => Object::Boolean(left_value < right_value),
            Token::LESSEQUAL => Object::Boolean(left_value <= right_value),
            Token::GREATER => Object::Boolean(left_value > right_value),
            Token::GREATEREQUAL => Object::Boolean(left_value >= right_value),
            token => unimplemented!("{token}"),
        }
    }

    fn evaluate_infix_expression_for_different_types_of_operands(
        operator: Token,
        left: &Object,
        right: &Object,
    ) -> Result<Object, EvaluationError> {
        match operator {
            Token::EQUALEQUAL => match (left, right) {
                (Object::Nil, Object::Nil) => Ok(Object::Boolean(true)),
                _ => Ok(Object::Boolean(false)),
            },
            Token::BANGEQUAL => Ok(Object::Boolean(true)),
            Token::PLUS
            | Token::MINUS
            | Token::SLASH
            | Token::STAR
            | Token::LESS
            | Token::LESSEQUAL
            | Token::GREATER
            | Token::GREATEREQUAL => {
                Err(EvaluationError::Adhoc(format!("Operands must be numbers.")))
            }
            _ => Err(EvaluationError::InvalidOperation {
                left: left.clone(),
                operator: operator,
                right: right.clone(),
            }),
        }
    }

    fn evaluate_infix_expression(
        &mut self,
        operator: Token,
        left_expr: &Expression,
        right_expr: &Expression,
    ) -> Result<Object, EvaluationError> {
        let left_value = self.evaluate_expression(left_expr)?;
        let right_value = self.evaluate_expression(right_expr)?;
        match (&left_value, &right_value) {
            (Object::Number(left), Object::Number(right)) => Ok(
                Interpreter::evaluate_numeric_infix_operation(operator, *left, *right),
            ),
            (Object::String(left), Object::String(right)) => {
                Interpreter::evaluate_string_infix_operation(operator, left, right)
            }
            (Object::Boolean(left), Object::Boolean(right)) => match operator.clone() {
                Token::EQUALEQUAL => Ok(Object::Boolean(*left == *right)),
                Token::BANGEQUAL => Ok(Object::Boolean(*left != *right)),
                token => Err(EvaluationError::InvalidOperation {
                    left: left_value,
                    operator: operator.clone(),
                    right: right_value,
                }),
            },
            _ => Interpreter::evaluate_infix_expression_for_different_types_of_operands(
                operator,
                &left_value,
                &right_value,
            ),
        }
    }

    fn evaluate_prefix_expression(
        &mut self,
        operator: Token,
        expression: &Expression,
    ) -> Result<Object, EvaluationError> {
        let value = self.evaluate_expression(expression)?;
        let object = match operator {
            Token::BANG => Object::Boolean(!value.get_truthy_value()),
            Token::MINUS => match value {
                Object::Number(v) => Object::Number(-v),
                object => {
                    return Err(EvaluationError::Adhoc(format!(
                        "Operand must be a number.\n[line {}]",
                        self.parser.get_curr_line()
                    )))
                }
            },
            t => unreachable!("token: {}", t),
        };
        Ok(object)
    }

    fn evaluate_expression(&mut self, expression: &Expression) -> Result<Object, EvaluationError> {
        let val = match expression {
            crate::parser::expression::Expression::NilLiteral => Object::Nil,
            crate::parser::expression::Expression::BooleanLiteral(v) => Object::Boolean(*v),
            crate::parser::expression::Expression::NumberLiteral(v) => Object::Number(*v),
            crate::parser::expression::Expression::StringLiteral(bytes) => {
                Object::String(bytes.clone())
            }
            crate::parser::expression::Expression::GroupedExpression(expr) => {
                self.evaluate_expression(expr.as_ref())?
            }
            crate::parser::expression::Expression::PrefixExpression { operator, expr } => {
                self.evaluate_prefix_expression(operator.clone(), expr.as_ref())?
            }
            crate::parser::expression::Expression::InfixExpression {
                operator,
                left_expr,
                right_expr,
            } => self.evaluate_infix_expression(
                operator.clone(),
                left_expr.as_ref(),
                right_expr.as_ref(),
            )?,
        };
        Ok(val)
    }

    pub(crate) fn evaluate(&mut self) -> Result<Object, EvaluationError> {
        let expression = self
            .parser
            .parse_expression(Precedence::Lowest)
            .or_else(|e| Err(EvaluationError::ParseError(e)))?;

        self.evaluate_expression(&expression)
    }
}
