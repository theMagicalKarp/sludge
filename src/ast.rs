use anyhow::{Error, anyhow};
use serde::Serialize;
use std::cmp::Ordering;
use std::ops::{Add, Div, Mul, Neg, Rem, Sub};

#[derive(Serialize, Debug, Clone)]
pub enum Value {
    Int32(i32),
    Boolean(bool),
    String(String),
    Function {
        arg_assignments: Vec<String>,
        statements: Vec<Statement>,
    },
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::String(s) => !s.is_empty(),
            Value::Boolean(v) => *v,
            Value::Int32(n) => *n == 0,
            _ => false,
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        use Value::*;
        match (self, other) {
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
            Value::Int32(n) => write!(f, "{n}"),
            Value::Boolean(n) => write!(f, "{n}"),
            Value::String(n) => write!(f, "{n}"),
            _ => Ok(()),
        }
    }
}

#[derive(Serialize, Debug, Clone)]
pub enum Expr {
    // Literal values
    Number(i32),
    String(String),

    // Variable references
    Identifier(String), // User-defined variables

    Function {
        arg_assignments: Vec<AssignTarget>,
        statements: Vec<Statement>,
    },

    FunctionCall {
        name: String,
        args: Vec<Expr>,
    },

    // Binary operations: arithmetic, comparison, logical
    BinaryOp {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },

    // Unary operations: negation, logical not
    UnaryOp {
        op: UnOp,
        operand: Box<Expr>,
    },
}

#[derive(Serialize, Debug, Clone)]
pub enum BinOp {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,

    // Comparison
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,

    // Logical
    And,
    Or,
}

#[derive(Serialize, Debug, Clone)]
pub enum UnOp {
    Neg, // Arithmetic negation: -x
    Not, // Logical negation: !x
}

#[derive(Serialize, Debug, Clone)]
pub enum Statement {
    // Variable assignment: x = 5, $1 = "hello", y += 3
    Assignment {
        target: AssignTarget,
        op: AssignOp,
        value: Expr,
    },

    // Print statement: print, print $1, print x, y, z
    Print(Vec<Expr>),

    Return(Expr),

    // Control flow
    If {
        condition: Expr,
        then_stmt: Box<Statement>,
        else_stmt: Option<Box<Statement>>,
    },

    While {
        condition: Expr,
        body: Box<Statement>,
    },

    For {
        init: Option<Box<Statement>>,
        condition: Option<Expr>,
        update: Option<Box<Statement>>,
        body: Box<Statement>,
    },

    // Block statement: { stmt1; stmt2; stmt3 }
    Block(Vec<Statement>),

    // Expression as statement (for side effects)
    Expression(Expr),
}

// Assignment targets: variables or field references
#[derive(Serialize, Debug, Clone)]
pub enum AssignTarget {
    Identifier(String), // Regular variable: x = 5
}

// Assignment operators
#[derive(Serialize, Debug, Clone)]
pub enum AssignOp {
    Assign, // =
}

#[derive(Debug, Clone, Serialize)]
pub struct Program {
    // pub rules: Vec<SludgeRule>, // All rules in the program
    pub statements: Vec<Statement>,
}
