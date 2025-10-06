pub mod builtins;
#[cfg(test)]
mod tests;
pub mod value;
pub mod variable_scope;

use crate::ast::*;
use crate::interpreter::value::NamedBuiltin;
use crate::interpreter::value::NamedBuiltinWithInterpreter;
use crate::interpreter::value::Value;

use crate::interpreter::variable_scope::VariableScope;

use anyhow::{Result, anyhow};
use std::cell::RefCell;
use std::io::Write;
use std::rc::Rc;

pub struct Interpreter {
    variables: Rc<VariableScope>,

    stdout: Rc<RefCell<dyn Write>>,
}

impl Interpreter {
    pub fn new(variables: Rc<VariableScope>, stdout: Rc<RefCell<dyn Write>>) -> Self {
        Self { variables, stdout }
    }

    pub fn run_program(&self, program: &Program) -> Result<Value> {
        self.execute_statements(&program.statements)
    }

    fn eval_expr(&self, expr: &Expr) -> Result<Value> {
        match expr {
            Expr::Member { target, field } => {
                let target = self.eval_expr(target)?;
                match target {
                    Value::List { values } => match field.as_str() {
                        "join" => Ok(Value::BuiltinFn(Rc::new(NamedBuiltin {
                            name: "join",
                            this: Value::List { values },
                            f: builtins::list::join,
                        }))),
                        "length" => Ok(Value::BuiltinFn(Rc::new(NamedBuiltin {
                            name: "length",
                            this: Value::List { values },
                            f: builtins::list::length,
                        }))),
                        "at" => Ok(Value::BuiltinFn(Rc::new(NamedBuiltin {
                            name: "at",
                            this: Value::List { values },
                            f: builtins::list::at,
                        }))),
                        "pop" => Ok(Value::BuiltinFn(Rc::new(NamedBuiltin {
                            name: "pop",
                            this: Value::List { values },
                            f: builtins::list::pop,
                        }))),
                        "push" => Ok(Value::BuiltinFn(Rc::new(NamedBuiltin {
                            name: "push",
                            this: Value::List { values },
                            f: builtins::list::push,
                        }))),
                        "map" => Ok(Value::BuiltinFn(Rc::new(NamedBuiltinWithInterpreter {
                            name: "map",
                            this: Value::List { values },
                            interpreter: Rc::new(Interpreter::new(
                                VariableScope::branch(&self.variables),
                                self.stdout.clone(),
                            )),
                            f: builtins::list::map,
                        }))),
                        "filter" => Ok(Value::BuiltinFn(Rc::new(NamedBuiltinWithInterpreter {
                            name: "filter",
                            this: Value::List { values },
                            interpreter: Rc::new(Interpreter::new(
                                VariableScope::branch(&self.variables),
                                self.stdout.clone(),
                            )),
                            f: builtins::list::filter,
                        }))),
                        "all" => Ok(Value::BuiltinFn(Rc::new(NamedBuiltinWithInterpreter {
                            name: "all",
                            this: Value::List { values },
                            interpreter: Rc::new(Interpreter::new(
                                VariableScope::branch(&self.variables),
                                self.stdout.clone(),
                            )),
                            f: builtins::list::all,
                        }))),
                        "any" => Ok(Value::BuiltinFn(Rc::new(NamedBuiltinWithInterpreter {
                            name: "any",
                            this: Value::List { values },
                            interpreter: Rc::new(Interpreter::new(
                                VariableScope::branch(&self.variables),
                                self.stdout.clone(),
                            )),
                            f: builtins::list::any,
                        }))),
                        "sum" => Ok(Value::BuiltinFn(Rc::new(NamedBuiltin {
                            name: "sum",
                            this: Value::List { values },
                            f: builtins::list::sum,
                        }))),
                        _ => Ok(Value::Null),
                    },
                    Value::Set { values } => match field.as_str() {
                        "has" => Ok(Value::BuiltinFn(Rc::new(NamedBuiltin {
                            name: "has",
                            this: Value::Set { values },
                            f: builtins::set::has,
                        }))),
                        "union" => Ok(Value::BuiltinFn(Rc::new(NamedBuiltin {
                            name: "union",
                            this: Value::Set { values },
                            f: builtins::set::union,
                        }))),
                        "intersection" => Ok(Value::BuiltinFn(Rc::new(NamedBuiltin {
                            name: "intersection",
                            this: Value::Set { values },
                            f: builtins::set::intersection,
                        }))),
                        "difference" => Ok(Value::BuiltinFn(Rc::new(NamedBuiltin {
                            name: "difference",
                            this: Value::Set { values },
                            f: builtins::set::difference,
                        }))),
                        "add" => Ok(Value::BuiltinFn(Rc::new(NamedBuiltin {
                            name: "add",
                            this: Value::Set { values },
                            f: builtins::set::add,
                        }))),
                        "remove" => Ok(Value::BuiltinFn(Rc::new(NamedBuiltin {
                            name: "remove",
                            this: Value::Set { values },
                            f: builtins::set::remove,
                        }))),
                        "length" => Ok(Value::BuiltinFn(Rc::new(NamedBuiltin {
                            name: "length",
                            this: Value::Set { values },
                            f: builtins::set::length,
                        }))),
                        _ => Ok(Value::Null),
                    },
                    Value::Dictionary { values } => match field.as_str() {
                        "get" => Ok(Value::BuiltinFn(Rc::new(NamedBuiltin {
                            name: "get",
                            this: Value::Dictionary { values },
                            f: builtins::dict::get,
                        }))),
                        "set" => Ok(Value::BuiltinFn(Rc::new(NamedBuiltin {
                            name: "set",
                            this: Value::Dictionary { values },
                            f: builtins::dict::set,
                        }))),
                        "remove" => Ok(Value::BuiltinFn(Rc::new(NamedBuiltin {
                            name: "remove",
                            this: Value::Dictionary { values },
                            f: builtins::dict::remove,
                        }))),
                        "items" => Ok(Value::BuiltinFn(Rc::new(NamedBuiltin {
                            name: "items",
                            this: Value::Dictionary { values },
                            f: builtins::dict::items,
                        }))),
                        "keys" => Ok(Value::BuiltinFn(Rc::new(NamedBuiltin {
                            name: "keys",
                            this: Value::Dictionary { values },
                            f: builtins::dict::keys,
                        }))),
                        "values" => Ok(Value::BuiltinFn(Rc::new(NamedBuiltin {
                            name: "values",
                            this: Value::Dictionary { values },
                            f: builtins::dict::values,
                        }))),
                        "length" => Ok(Value::BuiltinFn(Rc::new(NamedBuiltin {
                            name: "length",
                            this: Value::Dictionary { values },
                            f: builtins::dict::length,
                        }))),
                        _ => Ok(Value::Null),
                    },
                    _ => Ok(Value::Null),
                }
            }
            Expr::Number(n) => Ok(Value::Int32(*n)),
            Expr::String(s) => Ok(Value::String(s.clone())),

            Expr::Tuple { values } => Ok({
                let values: Vec<_> = values
                    .iter()
                    .map(|e| self.eval_expr(e))
                    .collect::<Result<_, _>>()?;

                Value::Tuple { values }
            }),

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

            Expr::Call { target, args } => self.eval_call(target, args),
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
                let interpreter =
                    Interpreter::new(VariableScope::branch(&self.variables), self.stdout.clone());

                for statement in statements {
                    if let Ok(Value::Return { value }) = interpreter.execute_statement(statement) {
                        return Ok(Value::Return { value });
                    }
                }

                Ok(Value::Null)
            }
        }
    }

    fn eval_call(&self, target: &Expr, args: &[Expr]) -> Result<Value> {
        match self.eval_expr(target) {
            Ok(Value::BuiltinFn(f)) => {
                let evaluated_args: Vec<_> = args
                    .iter()
                    .map(|e| self.eval_expr(e))
                    .collect::<Result<_, _>>()?;
                f.call(evaluated_args.as_slice())
            }
            Ok(Value::Function {
                arguments,
                statement,
                scope,
            }) => {
                if arguments.len() != args.len() {
                    return Err(anyhow!(
                        "Function expected {} args, got {}",
                        arguments.len(),
                        args.len()
                    ));
                }

                let evaluated_args: Vec<_> = args
                    .iter()
                    .map(|e| self.eval_expr(e))
                    .collect::<Result<_, _>>()?;

                let interpreter =
                    Interpreter::new(VariableScope::branch(&scope), self.stdout.clone());

                for (param, value) in arguments.iter().cloned().zip(evaluated_args) {
                    interpreter.variables.declare(param, value);
                }

                match interpreter.eval_expr(&statement)? {
                    Value::Return { value } => Ok(*value),
                    other => Ok(other),
                }
            }
            _ => Err(anyhow!("Invalid function")),
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

    fn execute_statements(&self, statements: &[Statement]) -> Result<Value> {
        for stmt in statements {
            self.execute_statement(stmt)?;
        }
        Ok(Value::Null)
    }

    fn execute_statement(&self, stmt: &Statement) -> Result<Value> {
        match stmt {
            Statement::Print(exprs) => {
                let values: Result<Vec<_>> =
                    exprs.iter().map(|expr| self.eval_expr(expr)).collect();
                let values = values?;
                let output: Vec<String> = values.iter().map(|v| v.to_string()).collect();
                writeln!(self.stdout.borrow_mut(), "{}", output.join(" "))?;
                self.stdout.borrow_mut().flush()?;
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
