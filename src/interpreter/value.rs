use crate::ast::*;
use crate::interpreter::variable_scope::VariableScope;
use anyhow::{Error, anyhow};
use serde::Serialize;
use std::cmp::Ordering;
use std::fmt;
use std::iter::Sum;
use std::ops::{Add, Div, Mul, Neg, Rem, Sub};
use std::rc::Rc;

pub struct BuiltInFn {
    pub run: Rc<dyn Fn(Vec<Value>) -> Value>,
}

impl fmt::Debug for BuiltInFn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("<builtin fn>")
    }
}

// if you want to clone Value easily:
impl Clone for BuiltInFn {
    fn clone(&self) -> Self {
        Self {
            run: self.run.clone(),
        }
    }
}

#[derive(Serialize, Debug, Clone)]
pub enum Value {
    Null,
    Int32(i32),
    Boolean(bool),
    String(String),
    Function {
        arguments: Vec<String>,
        statement: Box<Expr>,
        #[serde(skip_serializing)]
        scope: Rc<VariableScope>,
    },
    #[serde(skip_serializing)]
    Builtin(BuiltInFn),
    Array {
        values: Vec<Value>,
    },
    Return {
        value: Box<Value>,
    },
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Null => false,
            Value::String(s) => !s.is_empty(),
            Value::Boolean(v) => *v,
            Value::Int32(n) => *n == 0,
            Value::Array { values } => !values.is_empty(),
            _ => false,
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
            Value::Array { values } => {
                write!(
                    f,
                    "[{}]",
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
