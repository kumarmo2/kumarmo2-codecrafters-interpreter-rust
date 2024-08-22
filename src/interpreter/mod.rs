#![allow(dead_code, unused_variables)]
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
    Nil,
}

impl std::fmt::Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::Number(v) => write!(f, "{}", v),
            Object::Boolean(v) => write!(f, "{}", v),
            Object::Nil => write!(f, "nil"),
        }
    }
}

pub(crate) struct Interpreter {
    parser: Parser,
}

pub(crate) enum EvaluationError {
    ParseError(ParseError),
}

impl std::fmt::Debug for EvaluationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvaluationError::ParseError(e) => write!(f, "{:?}", e),
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
        todo!()
    }

    fn evaluate_expression(&mut self, expression: &Expression) -> Result<Object, EvaluationError> {
        let val = match expression {
            crate::parser::expression::Expression::NilLiteral => Object::Nil,
            crate::parser::expression::Expression::BooleanLiteral(v) => Object::Boolean(*v),
            crate::parser::expression::Expression::NumberLiteral(v) => Object::Number(*v),
            crate::parser::expression::Expression::StringLiteral(_) => todo!(),
            crate::parser::expression::Expression::GroupedExpression(_) => todo!(),
            crate::parser::expression::Expression::PrefixExpression { .. } => todo!(),
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
