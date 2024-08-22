use bytes::Bytes;

use crate::token::Token;

pub(crate) enum Expression {
    NilLiteral,
    BooleanLiteral(bool),
    NumberLiteral(f64),
    StringLiteral(Bytes),
    GroupedExpression(Box<Expression>),
    PrefixExpression {
        operator: Token,
        expr: Box<Expression>,
    },
}

impl std::fmt::Debug for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::NilLiteral => write!(f, "nil"),
            Expression::BooleanLiteral(v) => write!(f, "{}", v),
            Expression::NumberLiteral(v) => write!(f, "{:?}", v),
            Expression::StringLiteral(bytes) => {
                let str = unsafe { std::str::from_utf8_unchecked(bytes.as_ref()) };
                write!(f, "{}", str)
            }
            Expression::GroupedExpression(e) => write!(f, "(group {:?})", e),
            Expression::PrefixExpression { operator, expr } => {
                write!(f, "({} {:?})", operator, expr)
            }
        }
    }
}

#[derive(Clone)]
pub(crate) enum Precedence {
    Lowest = 1,
}

impl Precedence {
    pub(crate) fn value(&self) -> u32 {
        self.clone() as u32
    }
}
