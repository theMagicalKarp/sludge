use crate::interpreter::Interpreter;
use crate::interpreter::value::Value;
use crate::interpreter::variable_scope::VariableScope;

use anyhow::Error;
use anyhow::Result;
use std::cell::RefCell;
use std::rc::Rc;

pub fn new(_this: &Value, args: &[Value]) -> Result<Value, Error> {
    Ok(Value::List {
        values: Rc::new(RefCell::new(args.to_vec())),
    })
}

pub fn join(this: &Value, args: &[Value]) -> Result<Value, Error> {
    let delimiter: String = match args.first() {
        Some(Value::String(s)) => s.clone(),
        Some(v) => v.to_string(),
        None => String::new(),
    };

    match this {
        Value::List { values } => {
            let joined = values
                .borrow()
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(&delimiter);

            Ok(Value::String(joined))
        }
        _ => Ok(Value::Null),
    }
}

pub fn pop(this: &Value, _args: &[Value]) -> Result<Value, Error> {
    match this {
        Value::List { values } => Ok(match values.borrow_mut().pop() {
            Some(v) => v,
            _ => Value::Null,
        }),
        _ => Ok(Value::Null),
    }
}

pub fn push(this: &Value, args: &[Value]) -> Result<Value, Error> {
    match args.first() {
        Some(arg) => match this {
            Value::List { values } => {
                values.borrow_mut().push(arg.clone());
                Ok(Value::Null)
            }
            _ => Ok(Value::Null),
        },
        _ => Ok(Value::Null),
    }
}

pub fn at(this: &Value, args: &[Value]) -> Result<Value, Error> {
    let position: usize = match args.first() {
        Some(Value::Int32(v)) => *v as usize,
        _ => 0,
    };

    match this {
        Value::List { values } => match values.borrow().get(position) {
            Some(v) => Ok(v.clone()),
            None => Ok(Value::Null),
        },
        _ => Ok(Value::Null),
    }
}

pub fn sum(this: &Value, _args: &[Value]) -> Result<Value, Error> {
    match this {
        Value::List { values } => Ok(values.borrow().iter().sum()),
        _ => Ok(Value::Null),
    }
}

pub fn map(interpreter: Rc<Interpreter>, this: &Value, args: &[Value]) -> Result<Value, Error> {
    if let Some(Value::Function {
        arguments,
        statement,
        scope,
    }) = args.first()
    {
        if let Value::List { values } = this {
            let values = values
                .borrow()
                .iter()
                .map(|v| {
                    let interpreter =
                        Interpreter::new(VariableScope::branch(scope), interpreter.stdout.clone());

                    interpreter
                        .variables
                        .declare(arguments.first().unwrap().clone(), v.clone());

                    match interpreter.eval_expr(statement) {
                        Ok(Value::Return { value }) => *value,
                        _ => Value::Null,
                    }
                })
                .collect();
            let values = Rc::new(RefCell::new(values));
            return Ok(Value::List { values });
        }
        return Ok(Value::Null);
    }

    Ok(Value::Null)
}

pub fn filter(interpreter: Rc<Interpreter>, this: &Value, args: &[Value]) -> Result<Value, Error> {
    if let Some(Value::Function {
        arguments,
        statement,
        scope,
    }) = args.first()
    {
        if let Value::List { values } = this {
            let values = values
                .borrow()
                .iter()
                .filter(|&v| {
                    let interpreter =
                        Interpreter::new(VariableScope::branch(scope), interpreter.stdout.clone());

                    interpreter
                        .variables
                        .declare(arguments.first().unwrap().clone(), v.clone());

                    match interpreter.eval_expr(statement) {
                        Ok(Value::Return { value }) => value.is_truthy(),
                        _ => false,
                    }
                })
                .cloned()
                .collect();
            let values = Rc::new(RefCell::new(values));
            return Ok(Value::List { values });
        }
        return Ok(Value::Null);
    }

    Ok(Value::Null)
}

pub fn all(interpreter: Rc<Interpreter>, this: &Value, args: &[Value]) -> Result<Value, Error> {
    if let Some(Value::Function {
        arguments,
        statement,
        scope,
    }) = args.first()
    {
        if let Value::List { values } = this {
            let result = values.borrow().iter().all(|v| {
                let interpreter =
                    Interpreter::new(VariableScope::branch(scope), interpreter.stdout.clone());

                interpreter
                    .variables
                    .declare(arguments.first().unwrap().clone(), v.clone());

                match interpreter.eval_expr(statement) {
                    Ok(Value::Return { value }) => value.is_truthy(),
                    _ => false,
                }
            });

            return Ok(Value::Boolean(result));
        }
        return Ok(Value::Null);
    }

    Ok(Value::Null)
}

pub fn any(interpreter: Rc<Interpreter>, this: &Value, args: &[Value]) -> Result<Value, Error> {
    if let Some(Value::Function {
        arguments,
        statement,
        scope,
    }) = args.first()
    {
        if let Value::List { values } = this {
            let result = values.borrow().iter().any(|v| {
                let interpreter =
                    Interpreter::new(VariableScope::branch(scope), interpreter.stdout.clone());

                interpreter
                    .variables
                    .declare(arguments.first().unwrap().clone(), v.clone());

                match interpreter.eval_expr(statement) {
                    Ok(Value::Return { value }) => value.is_truthy(),
                    _ => false,
                }
            });

            return Ok(Value::Boolean(result));
        }
        return Ok(Value::Null);
    }

    Ok(Value::Null)
}

pub fn length(this: &Value, _args: &[Value]) -> Result<Value, Error> {
    match this {
        Value::List { values } => Ok(Value::Int32(values.borrow().len() as i32)),
        _ => Ok(Value::Null),
    }
}

// TODO:
// insert
// truncate
// has
// remove
// reverse
// sort
// chunks
// flatten
