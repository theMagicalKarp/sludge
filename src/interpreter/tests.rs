use crate::ast::parser::parse_program;
use crate::interpreter::{Interpreter, VariableScope};
use std::cell::RefCell;
use std::rc::Rc;

fn run_and_capture(src: &str) -> anyhow::Result<String> {
    let buffer: Rc<RefCell<Vec<u8>>> = Rc::new(RefCell::new(Vec::new()));
    let program = parse_program(src)?;
    Interpreter::new(VariableScope::new(), buffer.clone()).run_program(&program)?;
    Ok(String::from_utf8(buffer.borrow().to_vec())?)
}

fn run_and_capture_err(src: &str) -> String {
    let buffer: Rc<RefCell<Vec<u8>>> = Rc::new(RefCell::new(Vec::new()));
    match parse_program(src)
        .and_then(|p| Interpreter::new(VariableScope::new(), buffer.clone()).run_program(&p))
    {
        Ok(_) => panic!("expected program to error, but it succeeded"),
        Err(e) => format!("{}", e),
    }
}

#[test]
fn test_basic() -> anyhow::Result<()> {
    let out = run_and_capture(
        "
            let x = 123
            print(x)
        ",
    )?;
    assert_eq!(out, "123\n");
    Ok(())
}

#[test]
fn test_bools() -> anyhow::Result<()> {
    let out = run_and_capture(
        "
            print(true)
            print(false)
            print(true && true)
            print(true && false)
            print(false && false)
            print(true || true)
            print(true || false)
            print(false || false)
        ",
    )?;

    assert_eq!(
        out,
        [
            "true", "false", "true", "false", "false", "true", "true", "false", ""
        ]
        .join("\n")
    );
    Ok(())
}

#[test]
fn test_boolean_short_circuit_and_or() -> anyhow::Result<()> {
    // RHS should not be evaluated if LHS determines result
    let out = run_and_capture(
        r#"
            print(true || (1 / 0))   // true; RHS skipped
            print(false && (1 / 0))  // false; RHS skipped
            print(false || (2 == 2)) // true; RHS evaluated
            print(true && (2 == 2))  // true; RHS evaluated
        "#,
    )?;

    assert_eq!(out, ["true", "false", "true", "true", ""].join("\n"));
    Ok(())
}

#[test]
fn test_variable_scope() -> anyhow::Result<()> {
    let out = run_and_capture(
        "
            let x = 1
            print(x) // 1
            {
                print(x) // 1
                x = 4
                print(x) // 4
            }
            print(x) // 4

            {
                print(x) // 4
                x = 2
                print(x) // 2
                let x = 42
                print(x) // 42
                x = 3
                print(x) // 3
                {
                    print(x) // 3
                    x = 100
                    print(x) // 100
                    let x = 6
                    print(x) // 6
                    x = 7
                    print(x) // 7
                }
                print(x) // 100
            }
            print(x) // 2
        ",
    )?;

    assert_eq!(
        out,
        [
            "1", "1", "4", "4", "4", "2", "42", "3", "3", "100", "6", "7", "100", "2", ""
        ]
        .join("\n")
    );
    Ok(())
}

#[test]
fn test_operators() -> anyhow::Result<()> {
    let out = run_and_capture(
        "
            print(1 + 2) // 3
            print(2 * 4) // 8
            print(1 + 2 * 4) // 9
            print((1+2)*4) // 12
            print(10/5) // 2
            print(-42) // -42
            print(-42 - 2) // -44
            print(-12/-6) // 2
            print(-12/6) // -2
            print(-(12/6 + 3)) // -5
            print(3 * 2 * 5 * 10) // 300
            print((1*2) + (3 * 4)) // 14
            print(  ( 1    * 2) +(  3 *4)    ) // 14
        ",
    )?;

    assert_eq!(
        out,
        [
            "3", "8", "9", "12", "2", "-42", "-44", "2", "-2", "-5", "300", "14", "14", ""
        ]
        .join("\n")
    );
    Ok(())
}

#[test]
fn test_compare() -> anyhow::Result<()> {
    let out = run_and_capture(
        "
            // equality
            print(1 == 2) // false
            print(2 == 2) // true

            // inequality
            print(3 != 3) // false
            print(3 != 2) // true

            // less-than / less-or-equal
            print(1 <  2) // true
            print(2 <  1) // false
            print(2 <= 2) // true
            print(3 <= 2) // false

            // greater-than / greater-or-equal
            print(3 >  2) // true
            print(2 >  3) // false
            print(2 >= 2) // true
            print(1 >= 2) // false
        ",
    )?;

    let expected = [
        "false", "true", // equality
        "false", "true", // inequality
        "true", "false", "true", "false", // < / <=
        "true", "false", "true", "false", // > / >=
        "",
    ]
    .join("\n");

    assert_eq!(out, expected);
    Ok(())
}

#[test]
fn test_conditional() -> anyhow::Result<()> {
    let out = run_and_capture(
        r#"
            let a = 100

            // Simple if (true)
            if (a < 200) {
                print("1")
            }

            // Simple if (false)
            if (a == 1) {
                print("2")
            }

            // if / else if / else chain
            if (a == 50) {
                print("wrong-branch")
            } else if (a == 100) {
                print("3")
            } else {
                print("also-wrong")
            }

            // Nested conditionals
            if (a < 200) {
                if (a > 50) {
                    print("4")
                } else {
                    print("wrong-nested")
                }
            }

            if (false) {
                print("wrong branch")
            } else {
                print("5")
            }
        "#,
    )?;

    let expected = ["1", "3", "4", "5", ""].join("\n");
    assert_eq!(out, expected);
    Ok(())
}

#[test]
fn test_functions() -> anyhow::Result<()> {
    let out = run_and_capture(
        r#"
            let foo = fn(a, b, c) {
                return (a + b) * c
            }

            let bar = fn(x, y) {
                let n = 42
                return foo(x, x, y) + foo(y, x, x) + n
            }

            let qux = fn(a, b, c, d, e, f) {
                let z = foo(a, b, c)
                return z + bar(e, f) + d
            }

            print(qux(1, 2, 3, 4, 5, 6))
        "#,
    )?;

    assert_eq!(out, ["170", ""].join("\n"));
    Ok(())
}

#[test]
fn test_recursion() -> anyhow::Result<()> {
    let out = run_and_capture(
        r#"
            let factorial = fn(n) {
                if (n == 0 || n == 1) {
                    return 1
                }
                return n * factorial(n-1)
            }
            print(factorial(12))
        "#,
    )?;

    assert_eq!(out, ["479001600", ""].join("\n"));
    Ok(())
}

#[test]
fn test_function_curry() -> anyhow::Result<()> {
    let out = run_and_capture(
        r#"
            let foo = fn(a) {
                return fn(b) {
                    return a + b
                }
            }

            let addTen = foo(10)
            let addFive = foo(5)

            print(addTen(42))
            print(addFive(42))
            print(foo(2)(40))
        "#,
    )?;

    assert_eq!(out, ["52", "47", "42", ""].join("\n"));
    Ok(())
}

#[test]
fn test_list() -> anyhow::Result<()> {
    let out = run_and_capture(
        r#"
            let x = list(3,2,1)

            print(x.sum())
            print(x.join(">"))
            print(x.at(0))
            print(x.at(1))
            print(x.at(2))
            print(x.pop())
            print(x)
            x.push(42)
            print(x)
        "#,
    )?;

    let expected = [
        "6",
        "3>2>1",
        "3",
        "2",
        "1",
        "1",
        "list(3, 2)",
        "list(3, 2, 42)",
        "",
    ]
    .join("\n");
    assert_eq!(out, expected);
    Ok(())
}

#[test]
fn test_list_functional() -> anyhow::Result<()> {
    let out = run_and_capture(
        r#"
            let result = list(1,2,3,4,5,6,7,8).filter(fn(item) {
                return item % 2 == 0
            }).map(fn(item) {
                return item * item
            })

            print(result)
        "#,
    )?;

    assert_eq!(out, ["list(4, 16, 36, 64)", ""].join("\n"));
    Ok(())
}

#[test]
fn test_list_predicates() -> anyhow::Result<()> {
    let out = run_and_capture(
        r#"
            let is_positive = fn(item) {
                return item > 0
            }

            print(list(1,2,3,4,5).all(is_positive))
            print(list(1,2,3,4,5).any(is_positive))

            print(list(1,2,-3,4,5).all(is_positive))
            print(list(1,2,-3,4,5).any(is_positive))
        "#,
    )?;

    assert_eq!(out, ["true", "true", "false", "true", ""].join("\n"));
    Ok(())
}

#[test]
fn test_set() -> anyhow::Result<()> {
    let out = run_and_capture(
        r#"
            let x = set(1, 2, 1, 3)
            print(x.length()) // 3
            print(x.has(1)) // true
            print(x.has(2)) // true
            print(x.has(3)) // true
            print(x.has(4)) // false
            print(x.has(5)) // false

            let y = set(3, 4, 5, 5)
            print(y.length()) // 3
            print(y.has(1)) // false
            print(y.has(2)) // false
            print(y.has(3)) // true
            print(y.has(4)) // true
            print(y.has(5)) // true

            let z = x.union(y)
            print(z.length()) // 5
            print(z.has(1)) // true
            print(z.has(2)) // true
            print(z.has(3)) // true
            print(z.has(4)) // true
            print(z.has(5)) // true

            let z = x.intersection(y)
            print(z.length()) // 1
            print(z.has(1)) // false
            print(z.has(2)) // false
            print(z.has(3)) // true
            print(z.has(4)) // false
            print(z.has(5)) // false

            let z = x.difference(y)
            print(z.length()) // 2
            print(z.has(1)) // true
            print(z.has(2)) // true
            print(z.has(3)) // false
            print(z.has(4)) // false
            print(z.has(5)) // false
        "#,
    )?;

    let expected = [
        "3", "true", "true", "true", "false", "false", // x = set(1,2,3)
        "3", "false", "false", "true", "true", "true", // y = set(3,4,5)
        "5", "true", "true", "true", "true", "true", // x.union(y)
        "1", "false", "false", "true", "false", "false", // x.intersection(y)
        "2", "true", "true", "false", "false", "false", // x.difference(y)
        "",      // end of program
    ]
    .join("\n");
    assert_eq!(out, expected);
    Ok(())
}

#[test]
fn test_dict() -> anyhow::Result<()> {
    let out = run_and_capture(
        r#"
            let x = dict(("one", 1), ("two", 2), ("three", 3))
            print(x.length())
            print(x.get("one"))
            print(x.get("two"))
            print(x.get("three"))
            x.remove("three")
            print(x.length())
            print(x.get("three"))
            x.set("four", 4)
            print(x.length())
            print(x.get("four"))
        "#,
    )?;

    let expected = ["3", "1", "2", "3", "2", "NULL", "3", "4", ""].join("\n");
    assert_eq!(out, expected);
    Ok(())
}

#[test]
fn test_set_mutations() -> anyhow::Result<()> {
    let out = run_and_capture(
        r#"
            let x = set(1, 2, 1, 3)
            print(x.length()) // 3
            print(x.has(1)) // true
            print(x.has(2)) // true
            print(x.has(3)) // true
            print(x.has(4)) // false
            print(x.has(5)) // false

            x.add(1)
            print(x.length()) // 3
            print(x.has(1)) // true
            print(x.has(2)) // true
            print(x.has(3)) // true
            print(x.has(4)) // false
            print(x.has(5)) // false

            x.add(5)
            print(x.length()) // 4
            print(x.has(1)) // true
            print(x.has(2)) // true
            print(x.has(3)) // true
            print(x.has(4)) // false
            print(x.has(5)) // true

            x.remove(2)
            print(x.length()) // 3
            print(x.has(1)) // true
            print(x.has(2)) // false
            print(x.has(3)) // true
            print(x.has(4)) // false
            print(x.has(5)) // true

            x.remove(4)
            print(x.length()) // 3
            print(x.has(1)) // true
            print(x.has(2)) // false
            print(x.has(3)) // true
            print(x.has(4)) // false
            print(x.has(5)) // true
        "#,
    )?;

    let expected = [
        "3", "true", "true", "true", "false", "false", // x = set(1,2,3)
        "3", "true", "true", "true", "false", "false", // x.add(1)
        "4", "true", "true", "true", "false", "true", // x.add(5)
        "3", "true", "false", "true", "false", "true", // x.remove(2)
        "3", "true", "false", "true", "false", "true", // x.remove(4)
        "",     // end of program
    ]
    .join("\n");
    assert_eq!(out, expected);
    Ok(())
}

#[test]
fn test_ensure_return_short_circuit() -> anyhow::Result<()> {
    let out = run_and_capture(
        r#"
            let f = fn() {
                for (let i = 0; i < 10; i = i + 1) {
                    if (i > 3) {
                        return "should happen"
                    }
                    print(i)
                }
                return "should not happen"
            }

            let g = fn() {
                let i = 0
                while (i < 10) {
                    print(i)
                    if (i > 3) {
                        return "should happen"
                    }
                    i = i + 1
                }
                return "should not happen"
            }

            let isGreaterThanTen = fn(a) {
                if (a > 10) {
                    return "is greater than 10"
                } else if (a == 10) {
                    return "is not greater than 10, but is 10"
                } else {
                    if (a == -10) {
                        return "is not greater than 10, but is -10"
                    }
                }
                return "is not greater than 10"
            }

            print(f())
            print(g())
            print(isGreaterThanTen(1))
            print(isGreaterThanTen(10))
            print(isGreaterThanTen(11))
            print(isGreaterThanTen(-10))
        "#,
    )?;

    let expected = [
        "0",
        "1",
        "2",
        "3",
        "should happen",
        "0",
        "1",
        "2",
        "3",
        "4",
        "should happen",
        "is not greater than 10",
        "is not greater than 10, but is 10",
        "is greater than 10",
        "is not greater than 10, but is -10",
        "",
    ]
    .join("\n");

    assert_eq!(out, expected);
    Ok(())
}

#[test]
fn test_unknown_identifier_errors() {
    let err = run_and_capture_err(
        r#"
            print(does_not_exist)
        "#,
    );
    // From interpreter: "undefined variable '...'"
    assert!(
        err.contains("undefined variable") && err.contains("does_not_exist"),
        "unexpected error: {err}"
    );
}

#[test]
fn test_member_unknown_and_wrong_receiver_errors() {
    // Unknown member on list
    let err1 = run_and_capture_err(
        r#"
            let xs = list(1,2,3)
            print(xs.whoopsies())
        "#,
    );
    assert!(
        err1.contains("unknown member") && err1.contains("list"),
        "unexpected error: {err1}"
    );

    // Member access on a number (unsupported receiver type)
    let err2 = run_and_capture_err(
        r#"
            let fortyTwo = 42
            print(fortyTwo.length())
        "#,
    );
    assert!(
        err2.contains("member access not supported") || err2.contains("has no members"),
        "unexpected error: {err2}"
    );
}

#[test]
fn test_function_must_return_error() {
    // Your stricter rule: user functions must `return`
    let err = run_and_capture_err(
        r#"
            let f = fn(a) {
                a + 1 // no explicit return
            }
            print(f(2))
        "#,
    );
    assert!(err.contains("must `return`"), "unexpected error: {err}");
}

#[test]
fn test_lambda_must_return_in_map_error() {
    // HOFs (map/filter/all/any) are strict about explicit `return`
    let err = run_and_capture_err(
        r#"
            let xs = list(1,2,3)
            // lambda missing `return` should error
            let ys = xs.map(fn(item) { item + 1 })
            print(ys)
        "#,
    );
    assert!(
        err.contains("map") && err.contains("must `return`"),
        "unexpected error: {err}"
    );
}

#[test]
fn test_lambda_must_return_in_filter_error() {
    let err = run_and_capture_err(
        r#"
            let xs = list(1,2,3)
            let ys = xs.filter(fn(item) { item > 1 })
            print(ys)
        "#,
    );
    assert!(
        err.contains("filter") && err.contains("must `return`"),
        "unexpected error: {err}"
    );
}

#[test]
fn test_if_and_while_branches_only_evaluate_taken_paths() -> anyhow::Result<()> {
    // This sanity test ensures the interpreter doesn't evaluate both branches of if
    // (this complements the boolean short-circuit tests)
    let out = run_and_capture(
        r#"
            let i = 0
            if (true) {
                print("then")
            } else {
                print(1 / 0) // must not run
            }

            // While condition false at start: body must not run
            while (false) {
                print("nope")
            }

            print("done")
        "#,
    )?;

    assert_eq!(out, ["then", "done", ""].join("\n"));
    Ok(())
}

#[test]
fn test_call_target_not_callable_error() {
    let err = run_and_capture_err(
        r#"
            let x = 42
            print(x(1,2,3))
        "#,
    );
    assert!(
        err.contains("not callable") || err.contains("call target"),
        "unexpected error: {err}"
    );
}

#[test]
fn test_conditions_require_bools() {
    let err = run_and_capture_err(
        r#"
            if (1) {
                print("hello world")
            }
        "#,
    );
    assert!(err.contains("Expcted boolean got: 1"));
}
