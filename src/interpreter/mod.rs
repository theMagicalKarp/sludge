pub mod value;
pub mod variable_scope;

use crate::ast::*;
use crate::interpreter::value::Value;
use crate::interpreter::variable_scope::VariableScope;

use anyhow::{Result, anyhow};
use std::io::Write;
use std::rc::Rc;

pub struct Interpreter<'a, W: Write> {
    // Variable storage for user-defined and built-in variables
    // variables: HashMap<String, Value>,
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

    pub fn run_program(&mut self, program: &Program) -> Result<()> {
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

            Expr::FunctionCall { name, args } => match self.variables.get(name) {
                Some(Value::Function {
                    arguments,
                    statement,
                    scope,
                }) => {
                    let mut evaluated_args = Vec::with_capacity(args.len());
                    for arg_expr in args.iter() {
                        evaluated_args.push(self.eval_expr(arg_expr)?);
                    }

                    let mut interpreter =
                        Interpreter::new(VariableScope::branch(&scope), self.stdout);

                    for (name, value) in arguments.iter().cloned().zip(evaluated_args.into_iter()) {
                        interpreter.variables.declare(name, value);
                    }

                    interpreter.eval_expr(&statement)
                }
                _ => Err(anyhow!("Invalid function")),
            },
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
                let mut to_ret = Value::Boolean(true);
                for statement in statements.iter() {
                    match statement {
                        Statement::Return(expr) => {
                            to_ret = interpreter.eval_expr(expr)?;
                        }
                        _ => {
                            interpreter.execute_statement(statement)?;
                        }
                    }
                }

                Ok(to_ret)
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

    fn execute_statements(&mut self, statements: &[Statement]) -> Result<()> {
        for stmt in statements {
            self.execute_statement(stmt)?;
        }
        Ok(())
    }

    fn execute_statement(&mut self, stmt: &Statement) -> Result<()> {
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
            }
            Statement::Assignment { target, op, value } => {
                let new_value = self.eval_expr(value)?;
                match target {
                    AssignTarget::Identifier(name) => {
                        let final_value = match op {
                            AssignOp::Assign => new_value,
                        };
                        return match self.variables.set(name.clone(), final_value) {
                            Some(_) => Ok(()),
                            None => Err(anyhow!("'{}' is an undefined variable!", name)),
                        };
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
            }
            Statement::If {
                condition,
                then_stmt,
                else_stmt,
            } => {
                let cond_val = self.eval_expr(condition)?;
                if cond_val.is_truthy() {
                    self.eval_expr(then_stmt)?;
                } else if let Some(else_branch) = else_stmt {
                    self.eval_expr(else_branch)?;
                }
            }
            Statement::While { condition, body } => {
                while self.eval_expr(condition)?.is_truthy() {
                    self.eval_expr(body)?;
                }
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
            }
            Statement::Return(expr) => {
                self.eval_expr(expr)?;
            }
            Statement::Expression(expr) => {
                self.eval_expr(expr)?;
            }
        }
        Ok(())
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
        let mut interpreter = Interpreter::new(VariableScope::new(), &mut buffer);
        interpreter.run_program(&program)?;

        assert_eq!(String::from_utf8(buffer)?, "123\n");

        Ok(())
    }
}
