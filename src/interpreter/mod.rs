#![allow(dead_code, unused_variables)]

use std::{cell::RefCell, collections::HashMap, io::Write, rc::Rc};

use bytes::{BufMut, Bytes, BytesMut};
pub(crate) mod native;

use crate::{
    parser::{
        expression::{
            CallExpression, Expression, FunctionExpression, IfStatement, Precedence, Statement,
            VarDeclaration, WhileLoop,
        },
        ParseError, Parser,
    },
    token::Token,
    Void,
};
use crate::{Either, Either::Right};

#[derive(Clone)]
pub(crate) enum Object {
    Number(f64),
    Boolean(bool),
    String(Bytes),
    Function(Function),
    NativeFunction(Rc<dyn Fn(Option<Box<dyn Iterator<Item = Object>>>) -> Object>),
    Nil,
}

#[derive(Clone)]
pub(crate) struct Function {
    fe: Rc<FunctionExpression>,
    env: Env,
}

impl std::fmt::Debug for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::Number(v) => write!(f, "{}", v),
            Object::Boolean(v) => write!(f, "{}", v),
            Object::Nil => write!(f, "nil"),
            Object::String(bytes) => {
                let str = unsafe { std::str::from_utf8_unchecked(bytes.as_ref()) };
                write!(f, "{}", str)
            }
            Object::Function(fe) => write!(f, "{fe:?}", fe = fe.fe.as_ref()),
            Object::NativeFunction(_) => write!(f, "<native fn>"),
        }
    }
}

impl Object {
    pub(crate) fn get_truthy_value(&self) -> bool {
        match self {
            Object::Number(_) => true,
            Object::Boolean(v) => *v,
            Object::String(_) => true,
            Object::Nil => false,
            Object::Function(_) => true,
            Object::NativeFunction(_) => true,
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
            Object::Function(fe) => write!(f, "{fe:?}", fe = fe.fe.as_ref()),
            Object::NativeFunction(_) => write!(f, "<native fn>"),
        }
    }
}

type Env = Rc<RefCell<Environment>>;

#[derive(Default, Debug)]
pub(crate) struct Environment {
    values: HashMap<Bytes, Object>,
    parent_env: Option<Env>,
}

impl Environment {
    pub(crate) fn with_parent(parent: Env) -> Self {
        Self {
            values: HashMap::default(),
            parent_env: Some(parent),
        }
    }
    pub(crate) fn add(&mut self, key: Bytes, val: Object) -> Option<Object> {
        self.values.insert(key, val)
    }

    pub(crate) fn assign(&mut self, key: Bytes, val: Object) -> Option<Object> {
        if self.values.contains_key(key.as_ref()) {
            return self.values.insert(key, val);
        }
        if let Some(parent_env) = self.parent_env.as_ref() {
            return parent_env.clone().as_ref().borrow_mut().assign(key, val);
        }
        unreachable!()
    }

    pub(crate) fn get<K: AsRef<[u8]>>(&self, key: K) -> Object {
        if self.values.contains_key(key.as_ref()) {
            return self
                .values
                .get(key.as_ref())
                .map_or(Object::Nil, |v| v.clone());
        }
        if let Some(parent_env) = &self.parent_env {
            return parent_env.as_ref().borrow().get(key.as_ref());
        }
        Object::Nil
    }

    pub(crate) fn is_declared<K: AsRef<[u8]>>(&self, key: K) -> bool {
        if self.values.contains_key(key.as_ref()) {
            return true;
        }
        if let Some(parent_env) = &self.parent_env {
            return parent_env.as_ref().borrow().is_declared(key);
        }
        false
    }
}

pub(crate) struct Interpreter<W>
where
    W: Write,
{
    writer: W,
    parser: Parser,
}

pub(crate) enum EvaluationError {
    ParseError(ParseError),
    ExpectedSomethingButGotOther {
        expected: &'static str,
        got: Object,
    },
    Runtime(String),
    InvalidOperation {
        left: Object,
        operator: Token,
        right: Object,
    },
    UndefinedVariable {
        identifier: Bytes,
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
            EvaluationError::Runtime(str) => write!(f, "runtime error: {str}"),
            EvaluationError::UndefinedVariable { identifier } => {
                let ident = unsafe { std::str::from_utf8_unchecked(identifier) };
                write!(f, "undefined variable '{ident}'")
            }
        }
    }
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
fn evaluate_numeric_infix_operation(operator: Token, left_value: f64, right_value: f64) -> Object {
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
        Token::PLUS => {
            return Err(EvaluationError::Runtime(format!(
                "Operands must be two numbers or two strings."
            )))
        }
        Token::MINUS
        | Token::SLASH
        | Token::STAR
        | Token::LESS
        | Token::LESSEQUAL
        | Token::GREATER
        | Token::GREATEREQUAL => Err(EvaluationError::Runtime(format!(
            "Error: Operands must be numbers." // TODO: need to print the line number as well.
        ))),
        _ => Err(EvaluationError::InvalidOperation {
            left: left.clone(),
            operator: operator,
            right: right.clone(),
        }),
    }
}
impl<W> Interpreter<W>
where
    W: Write,
{
    pub(crate) fn from_source(source: String, writer: W) -> Result<Self, ParseError> {
        let parser = Parser::from_source(source)?;

        Ok(Self {
            writer,
            parser,
            // global_env: Environment::default(),
        })
    }

    fn evaluate_and_expression(
        &mut self,
        left_expr: &Expression,
        right_expr: &Expression,
        env: Env,
    ) -> Result<Object, EvaluationError> {
        let left_value = self.evaluate_expression(left_expr, env.clone())?;
        if !left_value.get_truthy_value() {
            Ok(left_value)
        } else {
            self.evaluate_expression(right_expr, env.clone())
        }
    }
    fn evaluate_or_expression(
        &mut self,
        left_expr: &Expression,
        right_expr: &Expression,
        env: Env,
    ) -> Result<Object, EvaluationError> {
        let left_value = self.evaluate_expression(left_expr, env.clone())?;
        if left_value.get_truthy_value() {
            Ok(left_value)
        } else {
            self.evaluate_expression(right_expr, env.clone())
        }
    }

    fn evaluate_assignment_infix_expression(
        &mut self,
        left_expr: &Expression,
        right_expr: &Expression,
        env: Env,
    ) -> Result<Object, EvaluationError> {
        let ident_bytes = match left_expr {
            Expression::Ident(ident_bytes) => ident_bytes,
            expr => {
                return Err(EvaluationError::Runtime(format!(
                    "expected expression but got {expr:?}"
                )))
            }
        };
        let value = self.evaluate_expression(right_expr, env.clone())?;
        if !env.as_ref().borrow().is_declared(ident_bytes.as_ref()) {
            return Err(EvaluationError::UndefinedVariable {
                identifier: ident_bytes.clone(),
            });
        }
        env.as_ref()
            .borrow_mut()
            .assign(ident_bytes.clone(), value.clone());
        Ok(value)
    }

    fn evaluate_infix_expression(
        &mut self,
        operator: Token,
        left_expr: &Expression,
        right_expr: &Expression,
        env: Rc<RefCell<Environment>>,
    ) -> Result<Object, EvaluationError> {
        if let Token::EQUAL = operator {
            return self.evaluate_assignment_infix_expression(left_expr, right_expr, env);
        }
        if let Token::And = operator {
            return self.evaluate_and_expression(left_expr, right_expr, env);
        }
        if let Token::Or = operator {
            return self.evaluate_or_expression(left_expr, right_expr, env);
        }
        let left_value = self.evaluate_expression(left_expr, env.clone())?;
        let right_value = self.evaluate_expression(right_expr, env.clone())?;
        match (&left_value, &right_value) {
            (Object::Number(left), Object::Number(right)) => {
                Ok(evaluate_numeric_infix_operation(operator, *left, *right))
            }
            (Object::String(left), Object::String(right)) => {
                evaluate_string_infix_operation(operator, left, right)
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
            _ => evaluate_infix_expression_for_different_types_of_operands(
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
        env: Rc<RefCell<Environment>>,
    ) -> Result<Object, EvaluationError> {
        let value = self.evaluate_expression(expression, env)?;
        let object = match operator {
            Token::BANG => Object::Boolean(!value.get_truthy_value()),
            Token::MINUS => match value {
                Object::Number(v) => Object::Number(-v),
                object => {
                    return Err(EvaluationError::Runtime(format!(
                        "Error: Operand must be a number.\n[line {}]",
                        self.parser.get_curr_line()
                    )))
                }
            },
            t => unreachable!("token: {}", t),
        };
        Ok(object)
    }

    fn evaluate_expression(
        &mut self,
        expression: &Expression,
        env: Env,
    ) -> Result<Object, EvaluationError> {
        let val = match expression {
            Expression::NilLiteral => Object::Nil,
            Expression::Ident(ident_bytes) => {
                if !env.as_ref().borrow().is_declared(ident_bytes) {
                    return Err(EvaluationError::UndefinedVariable {
                        identifier: ident_bytes.clone(),
                    });
                }
                env.as_ref().borrow().get(ident_bytes)
            }
            Expression::BooleanLiteral(v) => Object::Boolean(*v),
            Expression::NumberLiteral(v) => Object::Number(*v),
            Expression::StringLiteral(bytes) => Object::String(bytes.clone()),
            Expression::GroupedExpression(expr) => self.evaluate_expression(expr.as_ref(), env)?,
            Expression::PrefixExpression { operator, expr } => {
                self.evaluate_prefix_expression(operator.clone(), expr.as_ref(), env)?
            }
            Expression::InfixExpression {
                operator,
                left_expr,
                right_expr,
            } => self.evaluate_infix_expression(
                operator.clone(),
                left_expr.as_ref(),
                right_expr.as_ref(),
                env,
            )?,
            Expression::Print(e) => {
                let val = self.evaluate_expression(e.as_ref(), env.clone())?;
                let _ = writeln!(self.writer, "{}", val);
                Object::Nil
            }
            Expression::Function(fe) => {
                self.evaluate_funtion_expression(fe.clone(), env.clone())?
            }
            Expression::Call(ce) => self.evaluate_function_call(ce, env.clone())?,
        };
        Ok(val)
    }

    fn evaluate_native_function_call(
        &self,
        func: Rc<dyn Fn(Option<Box<dyn Iterator<Item = Object>>>) -> Object>,
    ) -> Result<Object, EvaluationError> {
        Ok((func.as_ref())(None))
    }

    fn evaluate_function_call(
        &mut self,
        call_expr: &CallExpression,
        env: Env,
    ) -> Result<Object, EvaluationError> {
        let Function {
            fe: func_expr,
            env: captured_env,
        } = match self.evaluate_expression(call_expr.callee.as_ref(), env.clone())? {
            Object::Function(fe) => fe,
            Object::NativeFunction(nfe) => return self.evaluate_native_function_call(nfe),
            expr => {
                return Err(EvaluationError::Runtime(format!(
                    "Callee must be a function"
                )))
            }
        };
        let arguments_count = call_expr
            .arguments
            .as_ref()
            .and_then(|args| Some(args.len()))
            .unwrap_or_else(|| 0);

        let mut parameter_count = func_expr
            .as_ref()
            .parameters
            .as_ref()
            .and_then(|args| Some(args.len()))
            .unwrap_or_else(|| 0);

        if arguments_count != parameter_count {
            return Err(EvaluationError::Runtime(format!(
                "Expected {parameter_count} arguments but got {arguments_count}."
            )));
        }
        let child_env = Rc::new(RefCell::new(Environment::with_parent(captured_env.clone())));
        if arguments_count != 0 {
            let mut parameters = func_expr.parameters.as_ref().unwrap().iter();
            let mut arguments = call_expr.arguments.as_ref().unwrap().iter();
            while parameter_count > 0 {
                let parameter = parameters.next().unwrap();
                let argument = arguments.next().unwrap();
                let arg_val = self.evaluate_expression(argument, env.clone())?;
                let name_bytes = parameter.get_bytes().unwrap(); // NOTE: ideally this should never fail.
                child_env
                    .as_ref()
                    .borrow_mut()
                    .add(name_bytes.clone(), arg_val);

                parameter_count -= 1;
            }
        }
        for (index, stmt) in func_expr.body.iter().enumerate() {
            if let Right(val) = self.evaluate_stmt(stmt, child_env.clone())? {
                return Ok(val);
            }
        }
        //TODO: add the support for return stmt and returning a value from a function also.
        //For now, the function will always return a nil.
        Ok(Object::Nil)
    }

    fn evaluate_funtion_expression(
        &self,
        fe: Rc<FunctionExpression>,
        env: Env,
    ) -> Result<Object, EvaluationError> {
        if let Some(name_token) = fe.as_ref().name.as_ref() {
            if let Some(name_bytes) = name_token.get_bytes() {
                // add in the environment.
                env.as_ref().borrow_mut().add(
                    name_bytes.clone(),
                    Object::Function(Function {
                        fe: fe.clone(),
                        env: env.clone(),
                    }),
                );
            }
        }
        Ok(Object::Function(Function {
            fe: fe,
            env: env.clone(),
        }))
    }

    pub(crate) fn evaluate(&mut self) -> Result<Object, EvaluationError> {
        let env = Environment::default();
        let expression = self
            .parser
            .parse_expression(Precedence::Lowest)
            .or_else(|e| Err(EvaluationError::ParseError(e)))?;

        self.evaluate_expression(&expression, Rc::new(RefCell::new(env)))
    }

    pub(crate) fn evaluate_stmt(
        &mut self,
        stmt: &Statement,
        env: Rc<RefCell<Environment>>,
    ) -> Result<Either<Void, Object>, EvaluationError> {
        match stmt {
            Statement::Expression(e) => {
                self.evaluate_expression(e, env)?;
            }
            Statement::Print(e) => {
                let val = self.evaluate_expression(e, env)?;
                let _ = writeln!(self.writer, "{}", val);
            }
            Statement::VarDeclaration(VarDeclaration { identifier, expr }) => {
                if let Some(expr) = expr {
                    let val = self.evaluate_expression(expr, env.clone())?;
                    env.as_ref().borrow_mut().add(identifier.clone(), val);
                } else {
                    env.as_ref()
                        .borrow_mut()
                        .add(identifier.clone(), Object::Nil);
                }
            }
            Statement::Block(stmts) => {
                let child_env = Rc::new(RefCell::new(Environment {
                    values: HashMap::new(),
                    parent_env: Some(env.clone()),
                }));
                for stmt in stmts.iter() {
                    if let Right(val) = self.evaluate_stmt(&stmt, child_env.clone())? {
                        return Ok(Right(val));
                    }
                }
            }
            Statement::IfStatement(if_statement) => {
                if let Right(val) = self.evaluate_if_statement(if_statement, env.clone())? {
                    return Ok(Right(val));
                }
            }
            Statement::WhileLoop(while_loop) => {
                if let Right(val) = self.evaluate_while_statement(while_loop, env.clone())? {
                    return Ok(Right(val));
                }
            }
            Statement::Return(exp) => {
                return Ok(Right(self.evaluate_expression(exp, env.clone())?));
            }
        };
        Ok(Either::Left(Void))
    }
    fn evaluate_while_statement(
        &mut self,
        while_loop: &WhileLoop,
        env: Env,
    ) -> Result<Either<Void, Object>, EvaluationError> {
        loop {
            let mut val = true;
            if let Some(expr) = &while_loop.expr {
                let x = self.evaluate_expression(expr, env.clone())?;
                val = x.get_truthy_value();
            }
            if !val {
                break;
            }
            if let Right(val) = self.evaluate_stmt(while_loop.block.as_ref(), env.clone())? {
                return Ok(Right(val));
            }
        }

        Ok(Either::Left(Void))
    }
    fn evaluate_if_statement(
        &mut self,
        if_statement: &IfStatement,
        env: Env,
    ) -> Result<Either<Void, Object>, EvaluationError> {
        let expr = &if_statement.expr;
        let val = self.evaluate_expression(expr, env.clone())?;
        let val = val.get_truthy_value();
        if val {
            if let Right(val) = self.evaluate_stmt(&if_statement.if_block, env.clone())? {
                return Ok(Right(val));
            }
        } else if let Some(else_block) = &if_statement.else_block {
            if let Right(val) = self.evaluate_stmt(else_block, env.clone())? {
                return Ok(Right(val));
            }
        }
        Ok(Either::Left(Void))
    }

    pub(crate) fn writer(&self) -> &W {
        &self.writer
    }

    pub(crate) fn evaluate_program(&mut self) -> Result<(), EvaluationError> {
        let statements = self
            .parser
            .parse_program()
            .or_else(|e| Err(EvaluationError::ParseError(e)))?;

        let global_env = Rc::new(RefCell::new(Environment::default()));
        use native::clock;
        global_env.as_ref().borrow_mut().add(
            b"clock".as_ref().into(),
            Object::NativeFunction(Rc::new(clock)),
        );

        for stmt in statements.iter() {
            match self.evaluate_stmt(stmt, global_env.clone())? {
                Either::Left(_) => (),
                Either::Right(_) => {
                    return Err(EvaluationError::Runtime(format!(
                        "return statements can only be in functions"
                    )))
                }
            }
        }
        Ok(())
    }
}
