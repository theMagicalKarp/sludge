pub mod parser;

use serde::Serialize;

#[derive(Serialize, Debug, Clone)]
pub enum Expr {
    Number(i32),
    String(String),
    Boolean(bool),

    Tuple {
        values: Vec<Expr>,
    },

    BinaryOp {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },

    UnaryOp {
        op: UnOp,
        operand: Box<Expr>,
    },

    Identifier(String),
    Member {
        target: Box<Expr>,
        field: String,
    },

    Block(Vec<Statement>),

    Function {
        arguments: Vec<AssignTarget>,
        statement: Box<Expr>,
    },

    Call {
        target: Box<Expr>,
        args: Vec<Expr>,
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
    Assignment {
        target: AssignTarget,
        op: AssignOp,
        value: Expr,
    },

    Declaration {
        target: AssignTarget,
        op: AssignOp,
        value: Expr,
    },

    Print(Vec<Expr>),

    Return(Expr),

    If {
        condition: Expr,
        then_stmt: Box<Expr>,
        else_stmt: Option<Box<Expr>>,
    },

    While {
        condition: Expr,
        body: Box<Expr>,
    },

    For {
        init: Option<Box<Statement>>,
        condition: Option<Expr>,
        update: Option<Box<Statement>>,
        body: Box<Expr>,
    },

    Expression(Expr),
}

#[derive(Serialize, Debug, Clone)]
pub enum AssignTarget {
    Identifier(String),
}

// Assignment operators
#[derive(Serialize, Debug, Clone)]
pub enum AssignOp {
    Assign,
}

#[derive(Debug, Clone, Serialize)]
pub struct Program {
    pub statements: Vec<Statement>,
}
