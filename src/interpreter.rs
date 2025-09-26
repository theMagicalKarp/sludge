use crate::ast::*;
use anyhow::{Result, anyhow};

use std::collections::HashMap;

pub struct Interpreter {
    // Variable storage for user-defined and built-in variables
    variables: HashMap<String, Value>,

    // Built-in variable state
    output_field_separator: String,  // OFS - Output field separator
    output_record_separator: String, // ORS - Output record separator
}

impl Interpreter {
    pub fn new() -> Self {
        let variables = HashMap::new();
        Self {
            variables,
            output_field_separator: " ".to_string(),
            output_record_separator: "\n".to_string(),
        }
    }
    pub fn with_vars(vars: HashMap<String, Value>) -> Self {
        Self {
            variables: vars,
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
                .cloned()
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
                let fun = self.variables.get(name).unwrap();
                let mut scope = Interpreter::with_vars(
                    self.variables
                        .iter()
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect(),
                );
                match fun {
                    Value::Function {
                        arg_assignments,
                        statements,
                    } => {
                        let mut to_ret = Value::Boolean(true);
                        for (name, arg_expression) in arg_assignments.iter().zip(args.iter()) {
                            let v = scope.eval_expr(arg_expression)?;
                            scope.variables.insert(name.clone(), v);
                        }
                        for stmt in statements.clone() {
                            match stmt {
                                Statement::Return(expr) => {
                                    to_ret = scope.eval_expr(&expr)?;
                                }
                                _ => {
                                    scope.execute_statement(&stmt)?;
                                }
                            };
                        }

                        Ok(to_ret)
                    }
                    _ => Err(anyhow!("Invalid function")),
                }
            }
            Expr::Function {
                arg_assignments,
                statements,
            } => Ok(Value::Function {
                arg_assignments: arg_assignments
                    .iter()
                    .map(|x| match x {
                        AssignTarget::Identifier(name) => name.to_string(),
                    })
                    .collect(),
                statements: statements.clone(),
            }),
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
                print!(
                    "{}{}",
                    output.join(&self.output_field_separator),
                    self.output_record_separator
                );
            }
            Statement::Assignment { target, op, value } => {
                let new_value = self.eval_expr(value)?;
                match target {
                    AssignTarget::Identifier(name) => {
                        let final_value = match op {
                            AssignOp::Assign => new_value,
                        };
                        self.variables.insert(name.clone(), final_value);
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
                    self.execute_statement(then_stmt)?;
                } else if let Some(else_branch) = else_stmt {
                    self.execute_statement(else_branch)?;
                }
            }
            Statement::While { condition, body } => {
                while self.eval_expr(condition)?.is_truthy() {
                    self.execute_statement(body)?;
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

                    self.execute_statement(body)?;

                    if let Some(update_stmt) = update {
                        self.execute_statement(update_stmt)?;
                    }
                }
            }
            Statement::Return(expr) => {
                self.eval_expr(expr)?;
            }
            Statement::Block(statements) => {
                self.execute_statements(statements)?;
            }
            Statement::Expression(expr) => {
                self.eval_expr(expr)?;
            }
        }
        Ok(())
    }
}
