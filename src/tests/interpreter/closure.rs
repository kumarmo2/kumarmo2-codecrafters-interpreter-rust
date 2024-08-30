use crate::tests::test_positive_tests;

#[test]
fn test_closure_positive_tests() {
    let tuple = [
        (
            include_str!("../../../lox-test/closure/close_over_function_parameter.lox").to_string(),
            "param\n",
        ),
        (
            include_str!("../../../lox-test/closure/assign_to_shadowed_later.lox").to_string(),
            "global\ninner\nassigned\nassigned\n",
        ),
        (
            include_str!("../../../lox-test/closure/assign_to_closure.lox").to_string(),
            "local\nafter f\nafter f\nafter g\n",
        ),
        (
            include_str!("../../../lox-test/closure/close_over_later_variable.lox").to_string(),
            "b\na\n",
        ),
        (
            include_str!("../../../lox-test/closure/closed_closure_in_function.lox").to_string(),
            "local\n",
        ),
        (
            include_str!("../../../lox-test/closure/nested_closure.lox").to_string(),
            "a\nb\nc\n",
        ),
        (
            include_str!("../../../lox-test/closure/open_closure_in_function.lox").to_string(),
            "local\n",
        ),
        (
            include_str!("../../../lox-test/closure/reference_closure_multiple_times.lox")
                .to_string(),
            "a\na\n",
        ),
        (
            include_str!("../../../lox-test/closure/reuse_closure_slot.lox").to_string(),
            "a\n",
        ),
        (
            include_str!("../../../lox-test/closure/shadow_closure_with_local.lox").to_string(),
            "closure\nshadow\nclosure\n",
        ),
        (
            include_str!("../../../lox-test/closure/unused_closure.lox").to_string(),
            "ok\n",
        ),
        (
            include_str!("../../../lox-test/closure/unused_later_closure.lox").to_string(),
            "a\n",
        ),
    ];
    let sources = tuple.clone().map(|t| t.0).into_iter();
    let expecteds = tuple.map(|t| t.1).into_iter();
    test_positive_tests(sources, expecteds);
}
