use core::panic;

use crate::interpreter::{EvaluationError, Interpreter};

#[test]
fn add() {
    let source = include_str!("../../../lox-test/operator/add.lox").to_string();
    let writer = vec![];
    let mut interpreter = Interpreter::from_source(source, writer).unwrap();
    interpreter.evaluate_program().unwrap();
    assert_eq!(interpreter.writer(), b"579\nstring\n");
}

#[test]
fn add_bool_nil() {
    let source = include_str!("../../../lox-test/operator/add_bool_nil.lox").to_string();
    errorneous_test(source, "Operands must be two numbers or two strings.");
}

#[test]
fn add_bool_num() {
    let source = include_str!("../../../lox-test/operator/add_bool_num.lox").to_string();
    errorneous_test(source, "Operands must be two numbers or two strings.");
}

#[test]
fn add_bool_string() {
    let source = include_str!("../../../lox-test/operator/add_bool_string.lox").to_string();
    errorneous_test(source, "Operands must be two numbers or two strings.");
}

#[test]
fn add_num_nil() {
    let source = include_str!("../../../lox-test/operator/add_num_nil.lox").to_string();
    errorneous_test(source, "Operands must be two numbers or two strings.");
}

#[test]
fn add_string_nil() {
    let source = include_str!("../../../lox-test/operator/add_string_nil.lox").to_string();
    errorneous_test(source, "Operands must be two numbers or two strings.");
}

fn errorneous_test(source: String, expected: &str) {
    let writer = vec![];
    let mut interpreter = Interpreter::from_source(source, writer).unwrap();
    match interpreter.evaluate_program() {
        Err(EvaluationError::Runtime(err)) => {
            assert_eq!(expected, err.as_str())
        }
        got => panic!("expected error, but got: {got:?}"),
    }
}
