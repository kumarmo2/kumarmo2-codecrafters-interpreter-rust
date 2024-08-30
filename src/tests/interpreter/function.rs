use core::panic;

use crate::{
    interpreter::{EvaluationError, Interpreter},
    tests::test_positive_test,
};

#[test]
fn empty_body() {
    let source = include_str!("../../../lox-test/function/empty_body.lox").to_string();
    let expected = "nil\n";
    test_positive_test(source, expected);
}

#[test]
fn local_mutual_recursion() {
    let source = include_str!("../../../lox-test/function/local_mutual_recursion.lox").to_string();
    let expected = "true\nfalse\nfalse\ntrue\n";
    test_positive_test(source, expected);
}

#[test]
fn local_recursion() {
    let source = include_str!("../../../lox-test/function/local_recursion.lox").to_string();
    let expected = "21\n";
    test_positive_test(source, expected);
}

#[test]
fn mutual_recursion() {
    let source = include_str!("../../../lox-test/function/mutual_recursion.lox").to_string();
    let expected = "true\ntrue\n";
    test_positive_test(source, expected);
}

#[test]
fn nested_call_with_arguments() {
    let source =
        include_str!("../../../lox-test/function/nested_call_with_arguments.lox").to_string();
    let expected = "hello world\n";
    test_positive_test(source, expected);
}
#[test]
fn parameters() {
    let source = include_str!("../../../lox-test/function/parameters.lox").to_string();
    let expected = "0\n1\n3\n6\n10\n15\n21\n28\n36\n";
    test_positive_test(source, expected);
}

#[test]
fn print() {
    let source = include_str!("../../../lox-test/function/print.lox").to_string();
    let expected = "<fn foo>\n<native fn>\n";
    test_positive_test(source, expected);
}

#[test]
fn recursion() {
    let source = include_str!("../../../lox-test/function/recursion.lox").to_string();
    let expected = "21\n";
    test_positive_test(source, expected);
}

#[test]
fn extra_arguments() {
    let source = include_str!("../../../lox-test/function/extra_arguments.lox").to_string();
    let writer = vec![];
    let mut interpreter = Interpreter::from_source(source, writer).unwrap();
    match interpreter.evaluate_program() {
        Ok(_) => panic!("was supposed to return error but instead got success."),
        Err(err) => match err {
            EvaluationError::Runtime(s) => {
                assert_eq!(s.as_str(), "Expected 2 arguments but got 4.")
            }
            err => panic!("expected runtime error, but got {err:?}"),
        },
    };
}
