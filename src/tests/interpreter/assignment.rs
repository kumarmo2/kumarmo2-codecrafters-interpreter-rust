use core::panic;

use crate::{
    interpreter::{EvaluationError, Interpreter},
    parser::ParseError,
};

#[test]
fn test_associativity() {
    let source = include_str!("../../../lox-test/assignment/associativity.lox").to_string();
    let writer = vec![];
    let mut interpreter = Interpreter::from_source(source, writer).unwrap();
    interpreter.evaluate_program().unwrap();
    assert_eq!(interpreter.writer(), b"c\nc\nc\n");
}

#[test]
fn test_local() {
    let source = include_str!("../../../lox-test/assignment/local.lox").to_string();
    let writer = vec![];
    let mut interpreter = Interpreter::from_source(source, writer).unwrap();
    interpreter.evaluate_program().unwrap();
    assert_eq!(interpreter.writer(), b"before\nafter\narg\narg\n");
}

#[test]
fn test_global() {
    let source = include_str!("../../../lox-test/assignment/global.lox").to_string();
    let writer = vec![];
    let mut interpreter = Interpreter::from_source(source, writer).unwrap();
    interpreter.evaluate_program().unwrap();
    assert_eq!(interpreter.writer(), b"before\nafter\narg\narg\n");
}

#[test]
fn test_infix_operator() {
    let source = include_str!("../../../lox-test/assignment/infix_operator.lox").to_string();
    let writer = vec![];
    let mut interpreter = Interpreter::from_source(source, writer).unwrap();
    match interpreter.evaluate_program() {
        Err(EvaluationError::ParseError(parse_err)) => match parse_err {
            ParseError::InvalidAssignmentTarget => (),
            _ => panic!("expected InvalidAssignmentTarget"),
        },
        _ => panic!("expected error"),
    };
}

#[test]
fn test_grouping() {
    let source = include_str!("../../../lox-test/assignment/grouping.lox").to_string();
    let writer = vec![];
    let mut interpreter = Interpreter::from_source(source, writer).unwrap();
    match interpreter.evaluate_program() {
        Err(EvaluationError::ParseError(parse_err)) => match parse_err {
            ParseError::InvalidAssignmentTarget => (),
            _ => panic!("expected InvalidAssignmentTarget"),
        },
        _ => panic!("expected error"),
    };
}

#[test]
fn test_prefix_operator() {
    let source = include_str!("../../../lox-test/assignment/prefix_operator.lox").to_string();
    let writer = vec![];
    let mut interpreter = Interpreter::from_source(source, writer).unwrap();
    match interpreter.evaluate_program() {
        Err(EvaluationError::ParseError(parse_err)) => match parse_err {
            ParseError::InvalidAssignmentTarget => (),
            _ => panic!("expected InvalidAssignmentTarget"),
        },
        _ => panic!("expected error"),
    };
}

#[test]
fn test_syntax() {
    let source = include_str!("../../../lox-test/assignment/syntax.lox").to_string();
    let writer = vec![];
    let mut interpreter = Interpreter::from_source(source, writer).unwrap();
    interpreter.evaluate_program().unwrap();
    assert_eq!(interpreter.writer(), b"var\nvar\n");
}

#[test]
fn test_undefined() {
    let source = include_str!("../../../lox-test/assignment/undefined.lox").to_string();
    let writer = vec![];
    let mut interpreter = Interpreter::from_source(source, writer).unwrap();

    match interpreter.evaluate_program() {
        Err(EvaluationError::UndefinedVariable { .. }) => (),
        t => panic!("expected error, but got: {t:?}"),
    };
}
