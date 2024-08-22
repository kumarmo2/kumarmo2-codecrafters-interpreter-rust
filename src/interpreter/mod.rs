#![allow(dead_code, unused_variables)]
use bytes::Bytes;

use crate::{
    parser::{
        expression::{Expression, Precedence},
        ParseError, Parser,
    },
    token::Token,
};

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
    ExpectedSomethingButGotOther { expected: &'static str, got: Object },
}

impl std::fmt::Debug for EvaluationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvaluationError::ParseError(e) => write!(f, "{:?}", e),
            EvaluationError::ExpectedSomethingButGotOther { expected, got } => {
                write!(f, "expected: {expected}, but got: {got}")
            }
        }
    }
}

impl Interpreter {
    pub(crate) fn from_source(source: String) -> Result<Self, ParseError> {
        let parser = Parser::from_source(source)?;
        Ok(Self { parser })
    }

    fn evaluate_infix_expression(
        &mut self,
        operator: Token,
        left_expr: &Expression,
        right_expr: &Expression,
    ) -> Result<Object, EvaluationError> {
        let left_value = self.evaluate_expression(left_expr)?;
        let left_value = match left_value {
            Object::Number(v) => v,
            object => {
                return Err(EvaluationError::ExpectedSomethingButGotOther {
                    expected: "number",
                    got: object,
                })
            }
        };
        let right_value = self.evaluate_expression(right_expr)?;
        let right_value = match right_value {
            Object::Number(v) => v,
            object => {
                return Err(EvaluationError::ExpectedSomethingButGotOther {
                    expected: "number",
                    got: object,
                })
            }
        };
        // TODO: add check for divide by 0.
        let value = match operator {
            Token::STAR => Object::Number(left_value * right_value),
            Token::SLASH => Object::Number(left_value / right_value),
            Token::PLUS => Object::Number(left_value + right_value),
            Token::MINUS => Object::Number(left_value - right_value),
            token => unimplemented!("{token}"),
        };
        Ok(value)
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
                    return Err(EvaluationError::ExpectedSomethingButGotOther {
                        expected: "number",
                        got: object,
                    })
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
