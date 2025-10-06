use crate::interpreter::value::Hashable;
use crate::interpreter::value::Value;

use anyhow::Result;
use anyhow::{Error, anyhow};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub fn dict(_this: &Value, args: &[Value]) -> Result<Value, Error> {
    let values: HashMap<Hashable, Value> = args
        .iter()
        .map(|value| {
            let (k_val, v_val) = match value {
                Value::Tuple { values } => {
                    let first = values.first().unwrap().clone();
                    let second = values.get(1).unwrap().clone();
                    Ok((first, second))
                }
                _ => Err(anyhow!("Expected Tuple!")),
            }?;
            let key = Hashable::try_from(k_val)?;
            Ok((key, v_val))
        })
        .collect::<Result<_, Error>>()?;
    let values = Rc::new(RefCell::new(values));
    Ok(Value::Dictionary { values })
}

pub fn get(this: &Value, args: &[Value]) -> Result<Value, Error> {
    match this {
        Value::Dictionary { values } => Ok(args
            .first()
            .and_then(|k| Hashable::try_from(k).ok())
            .and_then(|hk| values.borrow().get(&hk).cloned())
            .unwrap_or(Value::Null)),
        _ => Ok(Value::Null),
    }
}

pub fn set(this: &Value, args: &[Value]) -> Result<Value, Error> {
    match this {
        Value::Dictionary { values } => {
            if let [key, value, ..] = args
                && let Ok(hk) = Hashable::try_from(key)
            {
                values.borrow_mut().insert(hk, value.clone());
            }
            Ok(Value::Null)
        }
        _ => Ok(Value::Null),
    }
}

pub fn remove(this: &Value, args: &[Value]) -> Result<Value, Error> {
    match this {
        Value::Dictionary { values } => Ok(args
            .first()
            .and_then(|k| Hashable::try_from(k).ok())
            .and_then(|hk| values.borrow_mut().remove(&hk))
            .unwrap_or(Value::Null)),
        _ => Ok(Value::Null),
    }
}

pub fn items(this: &Value, _args: &[Value]) -> Result<Value, Error> {
    match this {
        Value::Dictionary { values } => {
            let values = values
                .borrow()
                .iter()
                .map(|(k, v)| Value::Tuple {
                    values: vec![k.as_value(), v.clone()],
                })
                .collect::<Vec<_>>();

            let values = Rc::new(RefCell::new(values));

            // TODO: Return iterable
            Ok(Value::List { values })
        }
        _ => Ok(Value::Null),
    }
}

pub fn values(this: &Value, _args: &[Value]) -> Result<Value, Error> {
    match this {
        Value::Dictionary { values } => {
            let values = values.borrow().values().cloned().collect::<Vec<_>>();
            let values = Rc::new(RefCell::new(values));

            // TODO: Return iterable
            Ok(Value::List { values })
        }
        _ => Ok(Value::Null),
    }
}

pub fn keys(this: &Value, _args: &[Value]) -> Result<Value, Error> {
    match this {
        Value::Dictionary { values } => {
            let values = values
                .borrow()
                .keys()
                .map(|k| k.as_value())
                .collect::<Vec<_>>();
            let values = Rc::new(RefCell::new(values));

            // TODO: Return iterable
            Ok(Value::List { values })
        }
        _ => Ok(Value::Null),
    }
}

pub fn length(this: &Value, _args: &[Value]) -> Result<Value, Error> {
    match this {
        Value::Dictionary { values } => Ok(Value::Int32(values.borrow().len() as i32)),
        _ => Ok(Value::Null),
    }
}
