use std::rc::Rc;

use bytes::Bytes;

use crate::token::Token;

pub(crate) enum Expression {
    NilLiteral,
    // NOTE: I had to add the "Print" expression because of majorly one reason.
    // 1. Lox's "and"/"or" operator works like javascript's. that means below is a valid expression
    // `true and print "this should be printed"
    // 2. Along with the above reason, also the "print" is not a function but is built directly
    //    into the language.
    Print(Box<Expression>),
    BooleanLiteral(bool),
    NumberLiteral(f64),
    StringLiteral(Bytes),
    Ident(Bytes),
    GroupedExpression(Box<Expression>),
    PrefixExpression {
        operator: Token,
        expr: Box<Expression>,
    },
    InfixExpression {
        operator: Token,
        left_expr: Box<Expression>,
        right_expr: Box<Expression>,
    },
    Function(Rc<FunctionExpression>),
    Call(CallExpression),
}
pub(crate) struct CallExpression {
    pub(crate) callee: Box<Expression>,
    pub(crate) arguments: Option<Vec<Expression>>,
}

pub(crate) struct FunctionExpression {
    pub(crate) name: Option<Token>,
    pub(crate) parameters: Option<Vec<Token>>,
    pub(crate) body: Vec<Statement>,
}

impl std::fmt::Debug for FunctionExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let FunctionExpression { name, .. } = self;
        write!(f, "<fn")?;
        match name {
            Some(name) => match name {
                Token::Identifier(name) => {
                    let name = unsafe { std::str::from_utf8_unchecked(name) };
                    write!(f, " {name}>")?;
                }
                _ => unreachable!(),
            },
            None => write!(f, ">")?,
        };
        Ok(())
    }
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
            Expression::InfixExpression {
                operator,
                left_expr,
                right_expr,
            } => write!(f, "({operator} {:?} {:?})", left_expr, right_expr),
            Expression::Ident(ident_bytes) => write!(f, "ident: {}", unsafe {
                std::str::from_utf8_unchecked(ident_bytes.as_ref())
            }),
            Expression::Print(e) => write!(f, "print {:?}", e.as_ref()),
            Expression::Function(fe) => write!(f, "{fe:?}", fe = fe.as_ref()),
            Expression::Call(CallExpression { callee, arguments }) => {
                write!(f, "{callee:?}(", callee = callee.as_ref())?;
                if let Some(args) = arguments {
                    let args_count = args.len();
                    for (index, arg) in args.iter().enumerate() {
                        write!(f, "{arg:?}")?;
                        if index != args_count - 1 {
                            write!(f, ",")?;
                        }
                    }
                }
                write!(f, ")")
            }
        }
    }
}

#[derive(Clone)]
pub(crate) enum Precedence {
    Lowest = 1,
    Assign = 2,
    Equals = 3,
    Or = 4,
    And = 5,
    LessGreater = 6,
    Sum = 7,
    Product = 8,
    Prefix = 9,
    Call = 10,
}

impl Precedence {
    pub(crate) fn value(&self) -> u8 {
        self.clone() as u8
    }
}

pub(crate) struct VarDeclaration {
    pub(crate) identifier: Bytes,
    pub(crate) expr: Option<Expression>,
}

// pub(crate) struct Assignment {
//     pub(crate) identifier: Bytes,
//     pub(crate) expr: Expression,
// }

pub(crate) struct IfStatement {
    pub(crate) expr: Expression,
    pub(crate) if_block: Statement,
    pub(crate) else_block: Option<Statement>,
}

pub(crate) struct WhileLoop {
    pub(crate) expr: Option<Expression>,
    pub(crate) block: Box<Statement>,
}

pub(crate) enum Statement {
    Expression(Expression),
    Print(Expression),
    VarDeclaration(VarDeclaration),
    // Assignment(Assignment),
    Block(Vec<Statement>),
    IfStatement(Box<IfStatement>),
    WhileLoop(WhileLoop),
    Return(Expression),
}

impl Statement {
    fn print_statements(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        prefix_whitespace: &str,
        statements: &Vec<Statement>,
    ) -> std::fmt::Result {
        for stmt in statements.iter() {
            match &stmt {
                Statement::Block(stms) => {
                    write!(f, "{prefix_whitespace}{{\n")?;
                    let _ =
                        self.print_statements(f, format!("{prefix_whitespace}  ").as_str(), &stms)?;
                    write!(f, "{prefix_whitespace}}}\n")?;
                }
                st => write!(f, "{prefix_whitespace}{:?}\n", st)?,
            }
        }
        Ok(())
    }
}

impl std::fmt::Debug for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Statement::Expression(e) => match e {
                Expression::Function(_) => write!(f, "{e:?}"),
                _ => write!(f, "{:?};", e),
            },

            Statement::Print(e) => write!(f, "print {:?};", e),
            Statement::VarDeclaration(VarDeclaration { identifier, expr }) => {
                let identifier = unsafe { std::str::from_utf8_unchecked(identifier.as_ref()) };
                match expr {
                    Some(expr) => write!(f, "var {} = {:?};", identifier, expr),
                    None => write!(f, "var {};", identifier),
                }
            }
            Statement::Block(statements) => {
                write!(f, "{{\n")?;
                self.print_statements(f, "  ", statements)?;
                write!(f, "}}")?;

                Ok(())
            }
            Statement::IfStatement(stmt) => {
                let IfStatement {
                    if_block,
                    expr,
                    else_block,
                } = stmt.as_ref();

                let _ = write!(f, "if {:?} ", expr)?;
                write!(f, "{:?}", if_block)?; // TODO: better pretty printing.
                let Some(else_block) = else_block else {
                    return Ok(());
                };
                write!(f, " else {:?}\n", else_block)
            }
            Statement::WhileLoop(WhileLoop { expr, block }) => {
                write!(f, "while ( {:?} ) {:?}", expr, block)
            }
            Statement::Return(e) => write!(f, "return {e:?}"),
        }
    }
}
