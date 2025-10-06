use crate::interpreter::value::Hashable;
use crate::interpreter::value::Value;

use anyhow::Error;
use anyhow::Result;
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

pub fn set(_this: &Value, args: &[Value]) -> Result<Value, Error> {
    let values: HashSet<_> = args
        .iter()
        .map(Hashable::try_from)
        .collect::<Result<_, Error>>()?;
    let values = Rc::new(RefCell::new(values));
    Ok(Value::Set { values })
}

pub fn union(this: &Value, args: &[Value]) -> Result<Value, Error> {
    match this {
        Value::Set { values } => match args.first() {
            Some(Value::Set {
                values: other_values,
            }) => {
                let values = values
                    .borrow()
                    .union(&other_values.borrow())
                    .cloned()
                    .collect();
                let values = Rc::new(RefCell::new(values));

                Ok(Value::Set { values })
            }
            _ => Ok(Value::Null),
        },
        _ => Ok(Value::Null),
    }
}

pub fn intersection(this: &Value, args: &[Value]) -> Result<Value, Error> {
    match this {
        Value::Set { values } => match args.first() {
            Some(Value::Set {
                values: other_values,
            }) => {
                let values = values
                    .borrow()
                    .intersection(&other_values.borrow())
                    .cloned()
                    .collect();
                let values = Rc::new(RefCell::new(values));

                Ok(Value::Set { values })
            }
            _ => Ok(Value::Null),
        },
        _ => Ok(Value::Null),
    }
}

pub fn difference(this: &Value, args: &[Value]) -> Result<Value, Error> {
    match this {
        Value::Set { values } => match args.first() {
            Some(Value::Set {
                values: other_values,
            }) => {
                let values = values
                    .borrow()
                    .difference(&other_values.borrow())
                    .cloned()
                    .collect();
                let values = Rc::new(RefCell::new(values));

                Ok(Value::Set { values })
            }
            _ => Ok(Value::Null),
        },
        _ => Ok(Value::Null),
    }
}

pub fn length(this: &Value, _args: &[Value]) -> Result<Value, Error> {
    match this {
        Value::Set { values } => Ok(Value::Int32(values.borrow().len() as i32)),
        _ => Ok(Value::Null),
    }
}

pub fn has(this: &Value, args: &[Value]) -> Result<Value, Error> {
    match this {
        Value::Set { values } => match args.first() {
            Some(target) => match Hashable::try_from(target) {
                Ok(hashable) => Ok(Value::Boolean(values.borrow().contains(&hashable))),
                _ => Ok(Value::Null),
            },
            _ => Ok(Value::Null),
        },
        _ => Ok(Value::Null),
    }
}

pub fn add(this: &Value, args: &[Value]) -> Result<Value, Error> {
    match this {
        Value::Set { values } => match args.first() {
            Some(value) => match Hashable::try_from(value) {
                Ok(hashable) => {
                    values.borrow_mut().insert(hashable);
                    Ok(Value::Null)
                }
                _ => Ok(Value::Null),
            },
            _ => Ok(Value::Null),
        },
        _ => Ok(Value::Null),
    }
}

pub fn remove(this: &Value, args: &[Value]) -> Result<Value, Error> {
    match this {
        Value::Set { values } => match args.first() {
            Some(value) => match Hashable::try_from(value) {
                Ok(hashable) => {
                    values.borrow_mut().remove(&hashable);
                    Ok(Value::Null)
                }
                _ => Ok(Value::Null),
            },
            _ => Ok(Value::Null),
        },
        _ => Ok(Value::Null),
    }
}
