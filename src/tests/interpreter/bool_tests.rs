use crate::interpreter::Interpreter;

#[test]
fn equality() {
    let source = include_str!("../../../lox-test/bool/equality.lox").to_string();
    let writer = vec![];
    let mut interpreter = Interpreter::from_source(source, writer).unwrap();
    interpreter.evaluate_program().unwrap();
    assert_eq!(interpreter.writer(), b"true\nfalse\nfalse\ntrue\nfalse\nfalse\nfalse\nfalse\nfalse\nfalse\ntrue\ntrue\nfalse\ntrue\ntrue\ntrue\ntrue\ntrue\n")
}

#[test]
fn not() {
    let source = include_str!("../../../lox-test/bool/not.lox").to_string();
    let writer = vec![];
    let mut interpreter = Interpreter::from_source(source, writer).unwrap();
    interpreter.evaluate_program().unwrap();
    assert_eq!(interpreter.writer(), b"false\ntrue\ntrue\n");
}
