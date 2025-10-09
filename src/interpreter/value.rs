use crate::ast::*;
use crate::interpreter::Interpreter;
use crate::interpreter::variable_scope::VariableScope;

use anyhow::{Error, anyhow};
use serde::Serialize;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::collections::HashSet;
use std::iter::Sum;
use std::ops::{Add, Div, Mul, Neg, Rem, Sub};
use std::rc::Rc;

pub trait BuiltinFn: std::fmt::Debug {
    fn call(&self, args: &[Value]) -> Result<Value, Error>;
}

#[derive(Clone)]
pub struct NamedBuiltin<F> {
    pub name: &'static str,
    pub this: Value,
    pub f: F,
}

impl<F> std::fmt::Debug for NamedBuiltin<F> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.debug_tuple("Builtin").field(&self.name).finish()
    }
}

impl<F> BuiltinFn for NamedBuiltin<F>
where
    F: Fn(&Value, &[Value]) -> Result<Value, Error>,
{
    fn call(&self, args: &[Value]) -> Result<Value, Error> {
        (self.f)(&self.this, args)
    }
}

#[derive(Clone)]
pub struct NamedBuiltinWithInterpreter<F> {
    pub name: &'static str,
    pub this: Value,
    pub interpreter: Rc<Interpreter>,
    pub f: F,
}

impl<F> std::fmt::Debug for NamedBuiltinWithInterpreter<F> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.debug_tuple("Builtin").field(&self.name).finish()
    }
}

impl<F> BuiltinFn for NamedBuiltinWithInterpreter<F>
where
    F: Fn(Rc<Interpreter>, &Value, &[Value]) -> Result<Value, Error>,
{
    fn call(&self, args: &[Value]) -> Result<Value, Error> {
        (self.f)(self.interpreter.clone(), &self.this, args)
    }
}

#[derive(Clone, Debug)]
pub enum Value {
    Null,
    Int32(i32),
    Boolean(bool),
    String(String),
    Function {
        arguments: Vec<String>,
        statement: Box<Expr>,
        scope: Rc<VariableScope>,
    },
    List {
        values: Rc<RefCell<Vec<Value>>>,
    },
    Dictionary {
        values: Rc<RefCell<HashMap<Hashable, Value>>>,
    },
    Tuple {
        values: Vec<Value>,
    },
    Set {
        values: Rc<RefCell<HashSet<Hashable>>>,
    },
    Return {
        value: Box<Value>,
    },
    BuiltinFn(Rc<dyn BuiltinFn>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub enum Hashable {
    Null,
    Int32(i32),
    Boolean(bool),
    String(String),
}

impl Hashable {
    pub fn as_value(&self) -> Value {
        match self {
            Hashable::Null => Value::Null,
            Hashable::Int32(i) => Value::Int32(*i),
            Hashable::Boolean(b) => Value::Boolean(*b),
            Hashable::String(s) => Value::String(s.clone()),
        }
    }
}

impl TryFrom<Value> for Hashable {
    type Error = Error; // use your error type

    fn try_from(v: Value) -> Result<Self, Self::Error> {
        match v {
            Value::Null => Ok(Hashable::Null),
            Value::Int32(i) => Ok(Hashable::Int32(i)),
            Value::Boolean(b) => Ok(Hashable::Boolean(b)),
            Value::String(s) => Ok(Hashable::String(s)),
            _ => Err(anyhow!("invalid key")),
        }
    }
}

impl TryFrom<&Value> for Hashable {
    type Error = Error; // use your error type

    fn try_from(v: &Value) -> Result<Self, Self::Error> {
        match v {
            Value::Null => Ok(Hashable::Null),
            Value::Int32(i) => Ok(Hashable::Int32(*i)),
            Value::Boolean(b) => Ok(Hashable::Boolean(*b)),
            Value::String(s) => Ok(Hashable::String(s.clone())),
            _ => Err(anyhow!("invalid key")),
        }
    }
}

impl std::fmt::Display for Hashable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Hashable::Null => write!(f, "NULL"),
            Hashable::Int32(n) => write!(f, "{n}"),
            Hashable::Boolean(n) => write!(f, "{n}"),
            Hashable::String(n) => write!(f, "{n}"),
        }
    }
}

impl Value {
    pub fn to_bool(&self) -> Result<bool, Error> {
        match self {
            Value::Boolean(v) => Ok(*v),
            v => Err(anyhow!("Expcted boolean got: {}", v)),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        use Value::*;
        match (self, other) {
            (Null, Null) => true,
            (Int32(a), Int32(b)) => a == b,
            (Boolean(a), Boolean(b)) => a == b,
            (String(a), String(b)) => a == b,
            _ => false,
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        use Value::*;
        match (self, other) {
            (Int32(a), Int32(b)) => Some(a.cmp(b)),
            (Boolean(a), Boolean(b)) => Some(a.cmp(b)),
            (String(a), String(b)) => Some(a.cmp(b)),
            _ => None,
        }
    }
}

impl Value {
    pub fn pow(self, exp: Value) -> Result<Value, Error> {
        match (self, exp) {
            (Value::Int32(base), Value::Int32(exp)) => {
                if exp < 0 {
                    Err(anyhow!("Negative exponents not supported for Int32"))
                } else {
                    Ok(Value::Int32(base.pow(exp as u32)))
                }
            }
            (a, b) => Err(anyhow!("Cannot exponentiate {:?} by {:?}", a, b)),
        }
    }
}

impl Add for Value {
    type Output = Result<Value, Error>;
    fn add(self, rhs: Value) -> Self::Output {
        match (self, rhs) {
            (Value::Int32(a), Value::Int32(b)) => Ok(Value::Int32(a + b)),
            (Value::String(a), Value::String(b)) => Ok(Value::String(a + &b)),
            (a, b) => Err(anyhow!(
                "Addition not supported between {:?} and {:?}",
                a,
                b
            )),
        }
    }
}

impl Sub for Value {
    type Output = Result<Value, Error>;
    fn sub(self, rhs: Value) -> Self::Output {
        match (self, rhs) {
            (Value::Int32(a), Value::Int32(b)) => Ok(Value::Int32(a - b)),
            (a, b) => Err(anyhow!(
                "Subtraction not supported between {:?} and {:?}",
                a,
                b
            )),
        }
    }
}

impl Sum for Value {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut total: Option<Value> = None;

        for v in iter {
            total = match total {
                Some(total) => (total + v).ok(),
                None => Some(v),
            };
        }
        match total {
            Some(total) => total,
            None => Value::Null,
        }
    }
}

impl<'a> Sum<&'a Value> for Value {
    fn sum<I: Iterator<Item = &'a Value>>(iter: I) -> Self {
        let mut total: Option<Value> = None;

        for v in iter {
            total = match total {
                Some(total) => (total + v.clone()).ok(),
                None => Some(v.clone()),
            };
        }
        match total {
            Some(total) => total,
            None => Value::Null,
        }
    }
}

impl Mul for Value {
    type Output = Result<Value, Error>;
    fn mul(self, rhs: Value) -> Self::Output {
        match (self, rhs) {
            (Value::Int32(a), Value::Int32(b)) => Ok(Value::Int32(a * b)),
            (a, b) => Err(anyhow!(
                "Multiplication not supported between {:?} and {:?}",
                a,
                b
            )),
        }
    }
}

impl Div for Value {
    type Output = Result<Value, Error>;
    fn div(self, rhs: Value) -> Self::Output {
        match (self, rhs) {
            (Value::Int32(_), Value::Int32(0)) => Err(anyhow!("Division by zero")),
            (Value::Int32(a), Value::Int32(b)) => Ok(Value::Int32(a / b)),
            (a, b) => Err(anyhow!(
                "Division not supported between {:?} and {:?}",
                a,
                b
            )),
        }
    }
}

impl Rem for Value {
    type Output = Result<Value, Error>;

    fn rem(self, rhs: Value) -> Self::Output {
        match (self, rhs) {
            (Value::Int32(_), Value::Int32(0)) => Err(anyhow!("Modulo by zero")),
            (Value::Int32(a), Value::Int32(b)) => Ok(Value::Int32(a % b)),
            (a, b) => Err(anyhow!("Modulo not supported between {:?} and {:?}", a, b)),
        }
    }
}

impl Neg for Value {
    type Output = Result<Value, Error>;

    fn neg(self) -> Self::Output {
        match self {
            Value::Int32(a) => Ok(Value::Int32(-a)),
            a => Err(anyhow!("Negation not supported for {:?}", a)),
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Null => write!(f, "NULL"),
            Value::Int32(n) => write!(f, "{n}"),
            Value::Boolean(n) => write!(f, "{n}"),
            Value::String(n) => write!(f, "{n}"),
            Value::List { values } => {
                write!(
                    f,
                    "list({})",
                    values
                        .borrow()
                        .iter()
                        .map(|v| v.to_string())
                        .collect::<Vec<String>>()
                        .join(", ")
                )
            }
            Value::Dictionary { values } => {
                write!(
                    f,
                    "dict({})",
                    values
                        .borrow()
                        .iter()
                        .map(|(k, v)| format!("({k}, {v})"))
                        .collect::<Vec<String>>()
                        .join(", ")
                )
            }
            Value::Set { values } => {
                write!(
                    f,
                    "set({})",
                    values
                        .borrow()
                        .iter()
                        .map(|k| format!("{k}"))
                        .collect::<Vec<String>>()
                        .join(", ")
                )
            }
            Value::Tuple { values } => {
                write!(
                    f,
                    "tuple({})",
                    values
                        .iter()
                        .map(|v| v.to_string())
                        .collect::<Vec<String>>()
                        .join(", ")
                )
            }
            _ => Ok(()),
        }
    }
}
