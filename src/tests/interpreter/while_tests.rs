use crate::interpreter::Interpreter;

#[test]
fn while_syntax() {
    let source = include_str!("../../../lox-test/while/syntax.lox").to_string();
    let writer = vec![];
    let mut interpreter = Interpreter::from_source(source, writer).unwrap();
    interpreter.evaluate_program().unwrap();
    assert_eq!(interpreter.writer(), b"1\n2\n3\n0\n1\n2\n");
}
