pub mod value;
pub mod variable_scope;

use crate::ast::*;
use crate::interpreter::value::Value;
use crate::interpreter::variable_scope::VariableScope;

use anyhow::{Result, anyhow};
use std::io::Write;
use std::rc::Rc;

pub struct Interpreter<'a, W: Write> {
    variables: Rc<VariableScope>,

    stdout: &'a mut W,

    // Built-in variable state
    output_field_separator: String,  // OFS - Output field separator
    output_record_separator: String, // ORS - Output record separator
}

impl<'a, W: Write> Interpreter<'a, W> {
    pub fn new(variables: Rc<VariableScope>, stdout: &'a mut W) -> Self {
        Self {
            variables,
            stdout,
            output_field_separator: " ".to_string(),
            output_record_separator: "\n".to_string(),
        }
    }

    pub fn run_program(&mut self, program: &Program) -> Result<Value> {
        self.execute_statements(&program.statements)
    }

    fn eval_expr(&mut self, expr: &Expr) -> Result<Value> {
        match expr {
            Expr::Number(n) => Ok(Value::Int32(*n)),
            Expr::String(s) => Ok(Value::String(s.clone())),

            Expr::Identifier(name) => Ok(self
                .variables
                .get(name)
                .unwrap_or(Value::String("".to_string()))),

            Expr::BinaryOp { op, left, right } => {
                let lval = self.eval_expr(left)?;
                let rval = self.eval_expr(right)?;
                self.eval_binary_op(op, &lval, &rval)
            }

            Expr::UnaryOp { op, operand } => {
                let val = self.eval_expr(operand)?;
                self.eval_unary_op(op, &val)
            }

            Expr::FunctionCall { name, args } => {
                let Some(Value::Function {
                    arguments,
                    statement,
                    scope,
                }) = self.variables.get(name)
                else {
                    return Err(anyhow!("Invalid function: {name}"));
                };

                if arguments.len() != args.len() {
                    return Err(anyhow!(
                        "Function {name} expected {} args, got {}",
                        arguments.len(),
                        args.len()
                    ));
                }

                let evaluated_args: Vec<_> = args
                    .iter()
                    .map(|e| self.eval_expr(e))
                    .collect::<Result<_, _>>()?;

                let mut interpreter = Interpreter::new(VariableScope::branch(&scope), self.stdout);

                for (param, value) in arguments.iter().cloned().zip(evaluated_args) {
                    interpreter.variables.declare(param, value);
                }

                match interpreter.eval_expr(&statement)? {
                    Value::Return { value } => Ok(*value),
                    other => Ok(other),
                }
            }
            Expr::Function {
                arguments,
                statement,
            } => Ok(Value::Function {
                arguments: arguments
                    .iter()
                    .map(|argument| match argument {
                        AssignTarget::Identifier(name) => name.to_string(),
                    })
                    .collect(),
                scope: VariableScope::branch(&self.variables),
                statement: statement.clone(),
            }),
            Expr::Block(statements) => {
                let mut interpreter =
                    Interpreter::new(VariableScope::branch(&self.variables), self.stdout);

                for statement in statements {
                    if let Ok(Value::Return { value }) = interpreter.execute_statement(statement) {
                        return Ok(Value::Return { value });
                    }
                }

                Ok(Value::Null)
            }
        }
    }

    fn eval_binary_op(&self, op: &BinOp, left: &Value, right: &Value) -> Result<Value> {
        match op {
            BinOp::Add => left.clone() + right.clone(),
            BinOp::Sub => left.clone() - right.clone(),
            BinOp::Mul => left.clone() * right.clone(),
            BinOp::Div => left.clone() / right.clone(),
            BinOp::Mod => left.clone() % right.clone(),
            BinOp::Pow => left.clone().pow(right.clone()),

            BinOp::Eq => Ok(Value::Boolean(left == right)),
            BinOp::Ne => Ok(Value::Boolean(left != right)),
            BinOp::Lt => Ok(Value::Boolean(left < right)),
            BinOp::Le => Ok(Value::Boolean(left <= right)),
            BinOp::Gt => Ok(Value::Boolean(left > right)),
            BinOp::Ge => Ok(Value::Boolean(left >= right)),
            BinOp::And => Ok(Value::Boolean(left.is_truthy() && right.is_truthy())),
            BinOp::Or => Ok(Value::Boolean(left.is_truthy() || right.is_truthy())),
        }
    }

    fn eval_unary_op(&self, op: &UnOp, operand: &Value) -> Result<Value> {
        match op {
            UnOp::Neg => -operand.clone(),
            UnOp::Not => Ok(Value::Boolean(!operand.is_truthy())),
        }
    }

    fn execute_statements(&mut self, statements: &[Statement]) -> Result<Value> {
        for stmt in statements {
            self.execute_statement(stmt)?;
        }
        Ok(Value::Null)
    }

    // fn execute_statement(&mut self, stmt: &Statement) -> Result<()> {
    fn execute_statement(&mut self, stmt: &Statement) -> Result<Value> {
        match stmt {
            Statement::Print(exprs) => {
                let values: Result<Vec<_>> =
                    exprs.iter().map(|expr| self.eval_expr(expr)).collect();
                let values = values?;
                let output: Vec<String> = values.iter().map(|v| v.to_string()).collect();
                write!(
                    self.stdout,
                    "{}{}",
                    output.join(&self.output_field_separator),
                    self.output_record_separator
                )?;
                self.stdout.flush()?;
                Ok(Value::Null)
            }
            Statement::Assignment { target, op, value } => {
                let new_value = self.eval_expr(value)?;
                match target {
                    AssignTarget::Identifier(name) => {
                        let final_value = match op {
                            AssignOp::Assign => new_value,
                        };
                        match self.variables.set(name.clone(), final_value) {
                            Some(_) => Ok(Value::Null),
                            None => Err(anyhow!("'{}' is an undefined variable!", name)),
                        }
                    }
                }
            }
            Statement::Declaration { target, op, value } => {
                let new_value = self.eval_expr(value)?;
                match target {
                    AssignTarget::Identifier(name) => {
                        let final_value = match op {
                            AssignOp::Assign => new_value,
                        };
                        self.variables.declare(name.clone(), final_value);
                    }
                }
                Ok(Value::Null)
            }
            Statement::If {
                condition,
                then_stmt,
                else_stmt,
            } => {
                let cond_val = self.eval_expr(condition)?;
                if cond_val.is_truthy() {
                    return self.eval_expr(then_stmt);
                } else if let Some(else_branch) = else_stmt {
                    return self.eval_expr(else_branch);
                }

                Ok(Value::Null)
            }
            Statement::While { condition, body } => {
                while self.eval_expr(condition)?.is_truthy() {
                    self.eval_expr(body)?;
                }
                Ok(Value::Null)
            }
            Statement::For {
                init,
                condition,
                update,
                body,
            } => {
                if let Some(init_stmt) = init {
                    self.execute_statement(init_stmt)?;
                }

                loop {
                    if let Some(cond) = condition
                        && !self.eval_expr(cond)?.is_truthy()
                    {
                        break;
                    }

                    self.eval_expr(body)?;

                    if let Some(update_stmt) = update {
                        self.execute_statement(update_stmt)?;
                    }
                }
                Ok(Value::Null)
            }
            Statement::Return(expr) => Ok(Value::Return {
                value: Box::new(self.eval_expr(expr)?),
            }),
            Statement::Expression(expr) => self.eval_expr(expr),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::parser::parse_program;

    #[test]
    fn test_basic() -> anyhow::Result<()> {
        let mut buffer: Vec<u8> = Vec::new();
        let program = parse_program({
            "
                let x = 123;
                print(x);
            "
        })?;
        Interpreter::new(VariableScope::new(), &mut buffer).run_program(&program)?;

        assert_eq!(String::from_utf8(buffer)?, "123\n");

        Ok(())
    }

    #[test]
    fn test_variable_scope() -> anyhow::Result<()> {
        let mut buffer: Vec<u8> = Vec::new();
        let program = parse_program({
            "
                let x = 1;
                print(x); // 1
                {
                    print(x); // 1
                    x = 4;
                    print(x); // 4
                }
                print(x); // 4

                {
                    print(x); // 4
                    x = 2;
                    print(x); // 2
                    let x = 42;
                    print(x); // 42
                    x = 3;
                    print(x); // 3
                    {
                        print(x); // 3
                        x = 100;
                        print(x); // 100
                        let x = 6;
                        print(x); // 6
                        x = 7;
                        print(x); // 7
                    }
                    print(x); // 100
                }
                print(x); // 2
            "
        })?;
        Interpreter::new(VariableScope::new(), &mut buffer).run_program(&program)?;

        assert_eq!(
            String::from_utf8(buffer)?,
            [
                "1", "1", "4", "4", "4", "2", "42", "3", "3", "100", "6", "7", "100", "2", ""
            ]
            .join("\n")
        );

        Ok(())
    }

    #[test]
    fn test_operators() -> anyhow::Result<()> {
        let mut buffer: Vec<u8> = Vec::new();
        let program = parse_program({
            "
                print(1 + 2); // 3
                print(2 * 4); // 8
                print(1 + 2 * 4); // 9
                print((1+2)*4); // 12
                print(10/5); // 2
                print(-42); // -42
                print(-42 - 2); // -44
                print(-12/-6); // 2
                print(-12/6); // -2
                print(-(12/6 + 3)); // -5
                print(3 * 2 * 5 * 10); // 300
                print((1*2) + (3 * 4)); // 14
                print(  ( 1    * 2) +(  3 *4)    ); // 14
            "
        })?;
        Interpreter::new(VariableScope::new(), &mut buffer).run_program(&program)?;

        assert_eq!(
            String::from_utf8(buffer)?,
            [
                "3", "8", "9", "12", "2", "-42", "-44", "2", "-2", "-5", "300", "14", "14", ""
            ]
            .join("\n")
        );

        Ok(())
    }

    #[test]
    fn test_compare() -> anyhow::Result<()> {
        let mut buffer: Vec<u8> = Vec::new();
        let program = parse_program({
            "
                // equality
                print(1 == 2); // false
                print(2 == 2); // true

                // inequality
                print(3 != 3); // false
                print(3 != 2); // true

                // less-than / less-or-equal
                print(1 <  2); // true
                print(2 <  1); // false
                print(2 <= 2); // true
                print(3 <= 2); // false

                // greater-than / greater-or-equal
                print(3 >  2); // true
                print(2 >  3); // false
                print(2 >= 2); // true
                print(1 >= 2); // false
            "
        })?;
        Interpreter::new(VariableScope::new(), &mut buffer).run_program(&program)?;

        let actual = String::from_utf8(buffer)?;

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
        let mut buffer: Vec<u8> = Vec::new();
        let program = parse_program({
            r#"
                let a = 100;

                // Simple if (true)
                if (a < 200) {
                    print("1");
                }

                // Simple if (false)
                if (a == 1) {
                    print("2");
                }

                // if / else if / else chain
                if (a == 50) {
                    print("wrong-branch");
                } else if (a == 100) {
                    print("3");
                } else {
                    print("also-wrong");
                }

                // Nested conditionals
                if (a < 200) {
                    if (a > 50) {
                        print("4");
                    } else {
                        print("wrong-nested");
                    }
                }

                if (false) {
                    print("wrong branch");
                } else {
                    print("5");
                }
            "#
        })?;

        Interpreter::new(VariableScope::new(), &mut buffer).run_program(&program)?;

        let actual = String::from_utf8(buffer)?;

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
        let mut buffer: Vec<u8> = Vec::new();
        let program = parse_program({
            r#"
                let foo = fn(a, b, c) {
                    return (a + b) * c;
                };

                let bar = fn(x, y) {
                    let n = 42;
                    return foo(x, x, y) + foo(y, x, x) + n;
                };

                let qux = fn(a, b, c, d, e, f) {
                    let z = foo(a, b, c);
                    return z + bar(e, f) + d;
                };

                print(qux(1, 2, 3, 4, 5, 6));
            "#
        })?;

        Interpreter::new(VariableScope::new(), &mut buffer).run_program(&program)?;

        let actual = String::from_utf8(buffer)?;

        let expected = ["170", ""].join("\n");
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_recursion() -> anyhow::Result<()> {
        let mut buffer: Vec<u8> = Vec::new();
        let program = parse_program({
            r#"
                let factorial = fn(n) {
                    if (n == 0 || n == 1) {
                        return 1;
                    }

                    return n * factorial(n-1);
                };

                print(factorial(12));
            "#
        })?;

        Interpreter::new(VariableScope::new(), &mut buffer).run_program(&program)?;

        let actual = String::from_utf8(buffer)?;

        let expected = ["479001600", ""].join("\n");
        assert_eq!(actual, expected);
        Ok(())
    }
}
