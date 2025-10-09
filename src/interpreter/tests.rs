use crate::ast::parser::parse_program;
use crate::interpreter::Interpreter;
use crate::interpreter::VariableScope;
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn test_basic() -> anyhow::Result<()> {
    let buffer: Rc<RefCell<Vec<u8>>> = Rc::new(RefCell::new(Vec::new()));
    let program = parse_program({
        "
                let x = 123
                print(x)
            "
    })?;
    Interpreter::new(VariableScope::new(), buffer.clone()).run_program(&program)?;

    assert_eq!(String::from_utf8(buffer.borrow().to_vec())?, "123\n");

    Ok(())
}

#[test]
fn test_bools() -> anyhow::Result<()> {
    let buffer: Rc<RefCell<Vec<u8>>> = Rc::new(RefCell::new(Vec::new()));
    let program = parse_program({
        "
            print(true)
            print(false)
            print(true && true)
            print(true && false)
            print(false && false)
            print(true || true)
            print(true || false)
            print(false || false)
        "
    })?;
    Interpreter::new(VariableScope::new(), buffer.clone()).run_program(&program)?;

    assert_eq!(
        String::from_utf8(buffer.borrow().to_vec())?,
        [
            "true", "false", "true", "false", "false", "true", "true", "false", ""
        ]
        .join("\n")
    );

    Ok(())
}

#[test]
fn test_variable_scope() -> anyhow::Result<()> {
    let buffer: Rc<RefCell<Vec<u8>>> = Rc::new(RefCell::new(Vec::new()));
    let program = parse_program({
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
            "
    })?;
    Interpreter::new(VariableScope::new(), buffer.clone()).run_program(&program)?;

    assert_eq!(
        String::from_utf8(buffer.borrow().to_vec())?,
        [
            "1", "1", "4", "4", "4", "2", "42", "3", "3", "100", "6", "7", "100", "2", ""
        ]
        .join("\n")
    );

    Ok(())
}

#[test]
fn test_operators() -> anyhow::Result<()> {
    let buffer: Rc<RefCell<Vec<u8>>> = Rc::new(RefCell::new(Vec::new()));
    let program = parse_program({
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
            "
    })?;
    Interpreter::new(VariableScope::new(), buffer.clone()).run_program(&program)?;

    assert_eq!(
        String::from_utf8(buffer.borrow().to_vec())?,
        [
            "3", "8", "9", "12", "2", "-42", "-44", "2", "-2", "-5", "300", "14", "14", ""
        ]
        .join("\n")
    );

    Ok(())
}

#[test]
fn test_compare() -> anyhow::Result<()> {
    let buffer: Rc<RefCell<Vec<u8>>> = Rc::new(RefCell::new(Vec::new()));
    let program = parse_program({
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
            "
    })?;
    Interpreter::new(VariableScope::new(), buffer.clone()).run_program(&program)?;

    let actual = String::from_utf8(buffer.borrow().to_vec())?;

    let expected = [
        "false", "true", // equality
        "false", "true", // inequality
        "true", "false", "true", "false", // < / <=
        "true", "false", "true", "false", // > / >=
        "",
    ]
    .join("\n");

    assert_eq!(actual, expected);

    Ok(())
}

#[test]
fn test_conditional() -> anyhow::Result<()> {
    let buffer: Rc<RefCell<Vec<u8>>> = Rc::new(RefCell::new(Vec::new()));
    let program = parse_program({
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
            "#
    })?;

    Interpreter::new(VariableScope::new(), buffer.clone()).run_program(&program)?;

    let actual = String::from_utf8(buffer.borrow().to_vec())?;

    // expected output lines (with trailing newline accounted for)
    let expected = [
        "1", // from a < 200
        // (no "2")
        "3", // from else-if
        "4", // from nested if
        "5", // from else
        "",  // trailing newline
    ]
    .join("\n");

    assert_eq!(actual, expected);

    Ok(())
}

#[test]
fn test_functions() -> anyhow::Result<()> {
    let buffer: Rc<RefCell<Vec<u8>>> = Rc::new(RefCell::new(Vec::new()));
    let program = parse_program({
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
            "#
    })?;

    Interpreter::new(VariableScope::new(), buffer.clone()).run_program(&program)?;

    let actual = String::from_utf8(buffer.borrow().to_vec())?;

    let expected = ["170", ""].join("\n");
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn test_recursion() -> anyhow::Result<()> {
    let buffer: Rc<RefCell<Vec<u8>>> = Rc::new(RefCell::new(Vec::new()));
    let program = parse_program({
        r#"
                let factorial = fn(n) {
                    if (n == 0 || n == 1) {
                        return 1
                    }

                    return n * factorial(n-1)
                }

                print(factorial(12))
        "#
    })?;

    Interpreter::new(VariableScope::new(), buffer.clone()).run_program(&program)?;

    let actual = String::from_utf8(buffer.borrow().to_vec())?;

    let expected = ["479001600", ""].join("\n");
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn test_function_curry() -> anyhow::Result<()> {
    let buffer: Rc<RefCell<Vec<u8>>> = Rc::new(RefCell::new(Vec::new()));
    let program = parse_program({
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
            "#
    })?;

    Interpreter::new(VariableScope::new(), buffer.clone()).run_program(&program)?;

    let actual = String::from_utf8(buffer.borrow().to_vec())?;

    let expected = ["52", "47", "42", ""].join("\n");
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn test_list() -> anyhow::Result<()> {
    let buffer: Rc<RefCell<Vec<u8>>> = Rc::new(RefCell::new(Vec::new()));
    let program = parse_program({
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
            "#
    })?;

    Interpreter::new(VariableScope::new(), buffer.clone()).run_program(&program)?;

    let actual = String::from_utf8(buffer.borrow().to_vec())?;

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
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn test_list_functional() -> anyhow::Result<()> {
    let buffer: Rc<RefCell<Vec<u8>>> = Rc::new(RefCell::new(Vec::new()));
    let program = parse_program({
        r#"
                let result = list(1,2,3,4,5,6,7,8).filter(fn(item) {
                    return item % 2 == 0
                }).map(fn(item) {
                    return item * item
                })

                print(result)
            "#
    })?;

    Interpreter::new(VariableScope::new(), buffer.clone()).run_program(&program)?;

    let actual = String::from_utf8(buffer.borrow().to_vec())?;

    let expected = ["list(4, 16, 36, 64)", ""].join("\n");
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn test_list_predicates() -> anyhow::Result<()> {
    let buffer: Rc<RefCell<Vec<u8>>> = Rc::new(RefCell::new(Vec::new()));
    let program = parse_program({
        r#"
                let is_positive = fn(item) {
                    return item > 0
                }

                print(list(1,2,3,4,5).all(is_positive))
                print(list(1,2,3,4,5).any(is_positive))

                print(list(1,2,-3,4,5).all(is_positive))
                print(list(1,2,-3,4,5).any(is_positive))

            "#
    })?;

    Interpreter::new(VariableScope::new(), buffer.clone()).run_program(&program)?;

    let actual = String::from_utf8(buffer.borrow().to_vec())?;

    let expected = ["true", "true", "false", "true", ""].join("\n");
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn test_set() -> anyhow::Result<()> {
    let buffer: Rc<RefCell<Vec<u8>>> = Rc::new(RefCell::new(Vec::new()));
    let program = parse_program({
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
            "#
    })?;

    Interpreter::new(VariableScope::new(), buffer.clone()).run_program(&program)?;

    let actual = String::from_utf8(buffer.borrow().to_vec())?;

    let expected = [
        "3", "true", "true", "true", "false", "false", // x = set(1,2,3)
        "3", "false", "false", "true", "true", "true", // y = set(3,4,5)
        "5", "true", "true", "true", "true", "true", // x.union(y)
        "1", "false", "false", "true", "false", "false", // x.intersection(y)
        "2", "true", "true", "false", "false", "false", // x.difference(y)
        "",      // end of program
    ]
    .join("\n");
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn test_dict() -> anyhow::Result<()> {
    let buffer: Rc<RefCell<Vec<u8>>> = Rc::new(RefCell::new(Vec::new()));
    let program = parse_program({
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
            "#
    })?;

    Interpreter::new(VariableScope::new(), buffer.clone()).run_program(&program)?;

    let actual = String::from_utf8(buffer.borrow().to_vec())?;

    let expected = ["3", "1", "2", "3", "2", "NULL", "3", "4", ""].join("\n");
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn test_set_mutations() -> anyhow::Result<()> {
    let buffer: Rc<RefCell<Vec<u8>>> = Rc::new(RefCell::new(Vec::new()));
    let program = parse_program({
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
            "#
    })?;

    Interpreter::new(VariableScope::new(), buffer.clone()).run_program(&program)?;

    let actual = String::from_utf8(buffer.borrow().to_vec())?;

    let expected = [
        "3", "true", "true", "true", "false", "false", // x = set(1,2,3)
        "3", "true", "true", "true", "false", "false", // x.add(1)
        "4", "true", "true", "true", "false", "true", // x.add(5)
        "3", "true", "false", "true", "false", "true", // x.remove(2)
        "3", "true", "false", "true", "false", "true", // x.remove(4)
        "",     // end of program
    ]
    .join("\n");
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn test_ensure_return_short_circuit() -> anyhow::Result<()> {
    let buffer: Rc<RefCell<Vec<u8>>> = Rc::new(RefCell::new(Vec::new()));
    let program = parse_program({
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

        "#
    })?;

    Interpreter::new(VariableScope::new(), buffer.clone()).run_program(&program)?;

    let actual = String::from_utf8(buffer.borrow().to_vec())?;

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
    assert_eq!(actual, expected);
    Ok(())
}
