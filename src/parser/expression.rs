pub(crate) enum Expression {
    NilLiteral,
    BooleanLiteral(bool),
}

impl std::fmt::Debug for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::NilLiteral => write!(f, "nil"),
            Expression::BooleanLiteral(v) => write!(f, "{}", v),
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
