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

use anyhow::{Context, Result, anyhow, bail};
use std::cell::RefCell;
use std::io::Write;
use std::rc::Rc;

pub struct Interpreter {
    pub(crate) variables: Rc<VariableScope>,
    pub(crate) stdout: Rc<RefCell<dyn Write>>,
}

impl Interpreter {
    pub fn new(variables: Rc<VariableScope>, stdout: Rc<RefCell<dyn Write>>) -> Self {
        Self { variables, stdout }
    }

    pub fn run_program(&self, program: &Program) -> Result<Value> {
        self.execute_statements(&program.statements)
    }

    fn type_name(v: &Value) -> &'static str {
        match v {
            Value::Null => "null",
            Value::Boolean(_) => "boolean",
            Value::Int32(_) => "int",
            Value::String(_) => "string",
            Value::Tuple { .. } => "tuple",
            Value::List { .. } => "list",
            Value::Set { .. } => "set",
            Value::Dictionary { .. } => "dict",
            Value::Function { .. } => "function",
            Value::BuiltinFn(_) => "builtin",
            Value::Return { .. } => "return",
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
            BinOp::And => Ok(Value::Boolean(left.to_bool()? && right.to_bool()?)),
            BinOp::Or => Ok(Value::Boolean(left.to_bool()? || right.to_bool()?)),
        }
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
                        other => bail!("unknown member '{}' on type list", other),
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
                        other => bail!("unknown member '{}' on type set", other),
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
                        other => bail!("unknown member '{}' on type dict", other),
                    },
                    other => bail!(
                        "member access not supported: type '{}' has no members",
                        Self::type_name(&other)
                    ),
                }
            }

            Expr::Number(n) => Ok(Value::Int32(*n)),
            Expr::String(s) => Ok(Value::String(s.clone())),
            Expr::Boolean(b) => Ok(Value::Boolean(*b)),

            Expr::Tuple { values } => Ok({
                let values: Vec<_> = values
                    .iter()
                    .map(|e| self.eval_expr(e))
                    .collect::<Result<_, _>>()?;
                Value::Tuple { values }
            }),

            Expr::Identifier(name) => self
                .variables
                .get(name)
                .ok_or_else(|| anyhow!("undefined variable '{}'", name)),

            Expr::BinaryOp { op, left, right } => match op {
                BinOp::And | BinOp::Or => self.eval_logical_op(op, left, right),
                _ => {
                    let lval = self.eval_expr(left)?;
                    let rval = self.eval_expr(right)?;
                    self.eval_binary_op(op, &lval, &rval)
                }
            },

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
        match self.eval_expr(target)? {
            Value::BuiltinFn(f) => {
                let evaluated_args: Vec<_> = args
                    .iter()
                    .map(|e| self.eval_expr(e))
                    .collect::<Result<_, _>>()?;
                f.call(evaluated_args.as_slice())
            }
            Value::Function {
                arguments,
                statement,
                scope,
            } => {
                if arguments.len() != args.len() {
                    bail!(
                        "function expected {} argument(s), got {}",
                        arguments.len(),
                        args.len()
                    );
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

                let result = interpreter
                    .eval_expr(&statement)
                    .with_context(|| "function evaluation failed")?;

                match result {
                    Value::Return { value } => Ok(*value),
                    other => bail!(
                        "function must `return` a value (got {} of type {})",
                        other,
                        Self::type_name(&other)
                    ),
                }
            }
            other => bail!(
                "call target is not callable (got type {})",
                Self::type_name(&other)
            ),
        }
    }

    fn eval_logical_op(&self, op: &BinOp, left: &Expr, right: &Expr) -> Result<Value> {
        let lval = self.eval_expr(left)?;
        let lbool = lval.to_bool()?;

        match op {
            BinOp::And => {
                // short-circuit: if left is false, return immediately
                if !lbool {
                    return Ok(Value::Boolean(false));
                }
                let rval = self.eval_expr(right)?;
                Ok(Value::Boolean(rval.to_bool()?))
            }
            BinOp::Or => {
                // short-circuit: if left is true, return immediately
                if lbool {
                    return Ok(Value::Boolean(true));
                }
                let rval = self.eval_expr(right)?;
                Ok(Value::Boolean(rval.to_bool()?))
            }
            _ => unreachable!("eval_logical_op called with non-logical operator"),
        }
    }

    fn eval_unary_op(&self, op: &UnOp, operand: &Value) -> Result<Value> {
        match op {
            UnOp::Neg => -operand.clone(),
            UnOp::Not => Ok(Value::Boolean(!operand.to_bool()?)),
        }
    }

    fn execute_statements(&self, statements: &[Statement]) -> Result<Value> {
        for stmt in statements {
            self.execute_statement(stmt)?;
        }
        Ok(Value::Null)
    }

    pub fn execute_statement(&self, stmt: &Statement) -> Result<Value> {
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
                if cond_val.to_bool()? {
                    return self.eval_expr(then_stmt);
                } else if let Some(else_branch) = else_stmt {
                    return self.eval_expr(else_branch);
                }

                Ok(Value::Null)
            }
            Statement::While { condition, body } => {
                while self.eval_expr(condition)?.to_bool()? {
                    if let Value::Return { value } = self.eval_expr(body)? {
                        return Ok(Value::Return { value });
                    }
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
                        && !self.eval_expr(cond)?.to_bool()?
                    {
                        break;
                    };

                    if let Value::Return { value } = self.eval_expr(body)? {
                        return Ok(Value::Return { value });
                    }

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
