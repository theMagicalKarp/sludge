use crate::ast::Expr;
use crate::interpreter::Interpreter;
use crate::interpreter::value::Value;
use crate::interpreter::variable_scope::VariableScope;

use anyhow::{Context, Error, Result, anyhow, bail};
use std::cell::RefCell;
use std::rc::Rc;

fn expect_list(this: &Value, fname: &str) -> Result<Rc<RefCell<Vec<Value>>>> {
    match this {
        Value::List { values } => Ok(values.clone()),
        other => bail!("{}: receiver is not a list (got {})", fname, other),
    }
}

fn expect_n_args_at_least(args: &[Value], n: usize, fname: &str) -> Result<()> {
    if args.len() < n {
        bail!(
            "{}: expected at least {} argument(s), got {}",
            fname,
            n,
            args.len()
        );
    }
    Ok(())
}

fn expect_index(args: &[Value], idx: usize, fname: &str) -> Result<usize> {
    let v = args
        .get(idx)
        .ok_or_else(|| anyhow!("{}: missing index argument at position {}", fname, idx))?;
    match v {
        Value::Int32(i) if *i >= 0 => Ok(*i as usize),
        Value::Int32(i) => bail!("{}: index must be non-negative, got {}", fname, i),
        other => bail!("{}: index must be Int32, got {}", fname, other),
    }
}

fn expect_callable<'a>(
    args: &'a [Value],
    fname: &str,
) -> Result<(&'a Vec<String>, &'a Expr, &'a Rc<VariableScope>)> {
    // matches your Value::Function shape
    match args.first() {
        Some(Value::Function {
            arguments,
            statement,
            scope,
        }) => Ok((arguments, statement, scope)),
        Some(other) => bail!(
            "{}: first argument must be a function, got {}",
            fname,
            other
        ),
        None => bail!("{}: missing function argument", fname),
    }
}

fn expect_return(from: Value, fname: &str) -> Result<Value> {
    match from {
        Value::Return { value } => Ok(*value),
        other => bail!("{fname}: function must `return` a value (got {other})"),
    }
}

pub fn new(_this: &Value, args: &[Value]) -> Result<Value, Error> {
    Ok(Value::List {
        values: Rc::new(RefCell::new(args.to_vec())),
    })
}

pub fn join(this: &Value, args: &[Value]) -> Result<Value, Error> {
    expect_n_args_at_least(args, 1, "join")?;
    let delimiter = match &args[0] {
        Value::String(s) => s.clone(),
        other => other.to_string(),
    };

    let values = expect_list(this, "join")?;
    let joined = values
        .borrow()
        .iter()
        .map(|v| v.to_string())
        .collect::<Vec<_>>()
        .join(&delimiter);

    Ok(Value::String(joined))
}

pub fn pop(this: &Value, _args: &[Value]) -> Result<Value, Error> {
    let values = expect_list(this, "pop")?;
    let mut borrow = values.borrow_mut();
    match borrow.pop() {
        Some(v) => Ok(v),
        None => bail!("pop: cannot pop from an empty list"),
    }
}

pub fn push(this: &Value, args: &[Value]) -> Result<Value, Error> {
    expect_n_args_at_least(args, 1, "push")?;
    let values = expect_list(this, "push")?;
    values.borrow_mut().extend_from_slice(args);
    Ok(Value::Int32(values.borrow().len() as i32))
}

pub fn at(this: &Value, args: &[Value]) -> Result<Value, Error> {
    let idx = expect_index(args, 0, "at")?;
    let values = expect_list(this, "at")?;
    match values.borrow().get(idx) {
        Some(v) => Ok(v.clone()),
        None => bail!(
            "at: index {} out of bounds (len = {})",
            idx,
            values.borrow().len()
        ),
    }
}

pub fn sum(this: &Value, _args: &[Value]) -> Result<Value, Error> {
    let values = expect_list(this, "sum")?;
    Ok(values.borrow().iter().sum())
}
pub fn map(interpreter: Rc<Interpreter>, this: &Value, args: &[Value]) -> Result<Value, Error> {
    let (fn_args, stmt, scope) = expect_callable(args, "map")?;
    let param = fn_args
        .first()
        .ok_or_else(|| anyhow!("map: function must accept at least 1 parameter"))?;
    let values = expect_list(this, "map")?;

    let out: Result<Vec<Value>> = values
        .borrow()
        .iter()
        .map(|v| {
            let child = Interpreter::new(VariableScope::branch(scope), interpreter.stdout.clone());
            child.variables.declare(param.clone(), v.clone());

            let evaluated = child
                .eval_expr(stmt)
                .with_context(|| "map: function evaluation failed")?;
            let ret = expect_return(evaluated, "map")?;
            Ok(ret)
        })
        .collect();

    Ok(Value::List {
        values: Rc::new(RefCell::new(out?)),
    })
}

pub fn filter(interpreter: Rc<Interpreter>, this: &Value, args: &[Value]) -> Result<Value, Error> {
    let (fn_args, stmt, scope) = expect_callable(args, "filter")?;
    let param = fn_args
        .first()
        .ok_or_else(|| anyhow!("filter: function must accept at least 1 parameter"))?;
    let values = expect_list(this, "filter")?;

    let mut out = Vec::new();
    for v in values.borrow().iter() {
        let child = Interpreter::new(VariableScope::branch(scope), interpreter.stdout.clone());
        child.variables.declare(param.clone(), v.clone());

        let evaluated = child
            .eval_expr(stmt)
            .with_context(|| "filter: function evaluation failed")?;
        let ret = expect_return(evaluated, "filter")?;
        if ret.to_bool()? {
            out.push(v.clone());
        }
    }

    Ok(Value::List {
        values: Rc::new(RefCell::new(out)),
    })
}

pub fn all(interpreter: Rc<Interpreter>, this: &Value, args: &[Value]) -> Result<Value, Error> {
    let (fn_args, stmt, scope) = expect_callable(args, "all")?;
    let param = fn_args
        .first()
        .ok_or_else(|| anyhow!("all: function must accept at least 1 parameter"))?;
    let values = expect_list(this, "all")?;

    for v in values.borrow().iter() {
        let child = Interpreter::new(VariableScope::branch(scope), interpreter.stdout.clone());
        child.variables.declare(param.clone(), v.clone());

        let evaluated = child
            .eval_expr(stmt)
            .with_context(|| "all: function evaluation failed")?;
        let ret = expect_return(evaluated, "all")?;
        if !ret.to_bool()? {
            return Ok(Value::Boolean(false));
        }
    }
    Ok(Value::Boolean(true))
}

pub fn any(interpreter: Rc<Interpreter>, this: &Value, args: &[Value]) -> Result<Value, Error> {
    let (fn_args, stmt, scope) = expect_callable(args, "any")?;
    let param = fn_args
        .first()
        .ok_or_else(|| anyhow!("any: function must accept at least 1 parameter"))?;
    let values = expect_list(this, "any")?;

    for v in values.borrow().iter() {
        let child = Interpreter::new(VariableScope::branch(scope), interpreter.stdout.clone());
        child.variables.declare(param.clone(), v.clone());

        let evaluated = child
            .eval_expr(stmt)
            .with_context(|| "any: function evaluation failed")?;
        let ret = expect_return(evaluated, "any")?;
        if ret.to_bool()? {
            return Ok(Value::Boolean(true));
        }
    }
    Ok(Value::Boolean(false))
}

pub fn length(this: &Value, _args: &[Value]) -> Result<Value, Error> {
    let values = expect_list(this, "length")?;
    Ok(Value::Int32(values.borrow().len() as i32))
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
