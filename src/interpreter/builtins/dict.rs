use crate::interpreter::value::{Hashable, Value};

use anyhow::{Context, Error, Result, bail};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

fn expect_dict(this: &Value, fname: &str) -> Result<Rc<RefCell<HashMap<Hashable, Value>>>> {
    match this {
        Value::Dictionary { values } => Ok(values.clone()),
        other => bail!("{fname}: receiver is not a dictionary (got {other})"),
    }
}

fn expect_n_args(args: &[Value], n: usize, fname: &str) -> Result<()> {
    if args.len() != n {
        bail!("{fname}: expected {n} argument(s), got {}", args.len());
    }
    Ok(())
}

fn expect_hashable_key(v: &Value, fname: &str) -> Result<Hashable> {
    Hashable::try_from(v.clone()).with_context(|| format!("{fname}: key is not hashable (got {v})"))
}

fn expect_tuple2(v: &Value, fname: &str) -> Result<(Value, Value)> {
    match v {
        Value::Tuple { values } if values.len() == 2 => Ok((values[0].clone(), values[1].clone())),
        Value::Tuple { values } => bail!(
            "{fname}: expected Tuple of length 2, got length {}",
            values.len()
        ),
        other => bail!("{fname}: expected Tuple(key, value), got {other}"),
    }
}

pub fn dict(_this: &Value, args: &[Value]) -> Result<Value, Error> {
    let mut map: HashMap<Hashable, Value> = HashMap::with_capacity(args.len());

    for (i, arg) in args.iter().enumerate() {
        let (k_val, v_val) = expect_tuple2(arg, "dict")
            .with_context(|| format!("dict: invalid entry at position {i}"))?;
        let key = expect_hashable_key(&k_val, "dict")
            .with_context(|| format!("dict: bad key at position {i}"))?;
        map.insert(key, v_val);
    }

    Ok(Value::Dictionary {
        values: Rc::new(RefCell::new(map)),
    })
}

pub fn get(this: &Value, args: &[Value]) -> Result<Value, Error> {
    expect_n_args(args, 1, "get")?;
    let values = expect_dict(this, "get")?;

    let key = expect_hashable_key(&args[0], "get")?;
    match values.borrow().get(&key) {
        Some(v) => Ok(v.clone()),
        None => Ok(Value::Null),
    }
}

pub fn set(this: &Value, args: &[Value]) -> Result<Value, Error> {
    expect_n_args(args, 2, "set")?;
    let values = expect_dict(this, "set")?;

    let key = expect_hashable_key(&args[0], "set")?;
    let val = args[1].clone();
    values.borrow_mut().insert(key, val);

    Ok(Value::Null)
}

pub fn remove(this: &Value, args: &[Value]) -> Result<Value, Error> {
    expect_n_args(args, 1, "remove")?;
    let values = expect_dict(this, "remove")?;

    let key = expect_hashable_key(&args[0], "remove")?;
    match values.borrow_mut().remove(&key) {
        Some(v) => Ok(v),
        None => Ok(Value::Null),
    }
}

pub fn items(this: &Value, _args: &[Value]) -> Result<Value, Error> {
    let values = expect_dict(this, "items")?;
    let list = values
        .borrow()
        .iter()
        .map(|(k, v)| Value::Tuple {
            values: vec![k.as_value(), v.clone()],
        })
        .collect::<Vec<_>>();
    Ok(Value::List {
        values: Rc::new(RefCell::new(list)),
    })
}

pub fn values(this: &Value, _args: &[Value]) -> Result<Value, Error> {
    let values = expect_dict(this, "values")?;
    let list = values.borrow().values().cloned().collect::<Vec<_>>();
    Ok(Value::List {
        values: Rc::new(RefCell::new(list)),
    })
}

pub fn keys(this: &Value, _args: &[Value]) -> Result<Value, Error> {
    let values = expect_dict(this, "keys")?;
    let list = values
        .borrow()
        .keys()
        .map(|k| k.as_value())
        .collect::<Vec<_>>();
    Ok(Value::List {
        values: Rc::new(RefCell::new(list)),
    })
}

pub fn length(this: &Value, _args: &[Value]) -> Result<Value, Error> {
    let values = expect_dict(this, "length")?;
    Ok(Value::Int32(values.borrow().len() as i32))
}
