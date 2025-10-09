use crate::interpreter::value::{Hashable, Value};

use anyhow::{Context, Error, Result, bail};
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

fn expect_set(this: &Value, fname: &str) -> Result<Rc<RefCell<HashSet<Hashable>>>> {
    match this {
        Value::Set { values } => Ok(values.clone()),
        other => bail!("{fname}: receiver is not a set (got {other})"),
    }
}

fn expect_n_args(args: &[Value], n: usize, fname: &str) -> Result<()> {
    if args.len() != n {
        bail!("{fname}: expected {n} argument(s), got {}", args.len());
    }
    Ok(())
}

fn expect_hashable(v: &Value, fname: &str) -> Result<Hashable> {
    Hashable::try_from(v.clone())
        .with_context(|| format!("{fname}: value is not hashable (got {v})"))
}

pub fn set(_this: &Value, args: &[Value]) -> Result<Value, Error> {
    let mut hs: HashSet<Hashable> = HashSet::with_capacity(args.len());
    for (i, v) in args.iter().enumerate() {
        let h = expect_hashable(v, "set").with_context(|| format!("set: at position {i}"))?;
        hs.insert(h);
    }
    Ok(Value::Set {
        values: Rc::new(RefCell::new(hs)),
    })
}

pub fn union(this: &Value, args: &[Value]) -> Result<Value, Error> {
    expect_n_args(args, 1, "union")?;
    let a = expect_set(this, "union")?;
    let b = match &args[0] {
        Value::Set { values } => values.clone(),
        other => bail!("union: argument must be a set (got {other})"),
    };

    let out: HashSet<Hashable> = a.borrow().union(&b.borrow()).cloned().collect();
    Ok(Value::Set {
        values: Rc::new(RefCell::new(out)),
    })
}

pub fn intersection(this: &Value, args: &[Value]) -> Result<Value, Error> {
    expect_n_args(args, 1, "intersection")?;
    let a = expect_set(this, "intersection")?;
    let b = match &args[0] {
        Value::Set { values } => values.clone(),
        other => bail!("intersection: argument must be a set (got {other})"),
    };

    let out: HashSet<Hashable> = a.borrow().intersection(&b.borrow()).cloned().collect();
    Ok(Value::Set {
        values: Rc::new(RefCell::new(out)),
    })
}

pub fn difference(this: &Value, args: &[Value]) -> Result<Value, Error> {
    expect_n_args(args, 1, "difference")?;
    let a = expect_set(this, "difference")?;
    let b = match &args[0] {
        Value::Set { values } => values.clone(),
        other => bail!("difference: argument must be a set (got {other})"),
    };

    let out: HashSet<Hashable> = a.borrow().difference(&b.borrow()).cloned().collect();
    Ok(Value::Set {
        values: Rc::new(RefCell::new(out)),
    })
}

pub fn length(this: &Value, _args: &[Value]) -> Result<Value, Error> {
    let s = expect_set(this, "length")?;
    Ok(Value::Int32(s.borrow().len() as i32))
}

pub fn has(this: &Value, args: &[Value]) -> Result<Value, Error> {
    expect_n_args(args, 1, "has")?;
    let s = expect_set(this, "has")?;
    let needle = expect_hashable(&args[0], "has")?;
    Ok(Value::Boolean(s.borrow().contains(&needle)))
}

pub fn add(this: &Value, args: &[Value]) -> Result<Value, Error> {
    expect_n_args(args, 1, "add")?;
    let s = expect_set(this, "add")?;
    let h = expect_hashable(&args[0], "add")?;
    let inserted = s.borrow_mut().insert(h);
    Ok(Value::Boolean(inserted))
}

pub fn remove(this: &Value, args: &[Value]) -> Result<Value, Error> {
    expect_n_args(args, 1, "remove")?;
    let s = expect_set(this, "remove")?;
    let h = expect_hashable(&args[0], "remove")?;
    let existed = s.borrow_mut().remove(&h);
    Ok(Value::Boolean(existed))
}
