pub mod parser;

use serde::Serialize;

#[derive(Serialize, Debug, Clone)]
pub struct Callable {
    pub name: Box<Expr>,
    pub args: Vec<Expr>,
}

#[derive(Serialize, Debug, Clone)]
pub enum Expr {
    // Literal values
    Number(i32),
    String(String),

    // Variable references
    Identifier(String), // User-defined variables

    Block(Vec<Statement>),

    Function {
        arguments: Vec<AssignTarget>,
        statement: Box<Expr>,
    },

    FunctionCall(Callable),

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

    Declaration {
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
