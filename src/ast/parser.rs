use crate::ast::*;
use anyhow::{Result, anyhow};
use pest::Parser;
use pest::iterators::{Pair, Pairs};
use pest::pratt_parser::{Assoc, Op, PrattParser};

#[derive(pest_derive::Parser)]
#[grammar = "grammar.pest"]
pub struct SludgeParser;

lazy_static::lazy_static! {
    static ref PRATT_PARSER: PrattParser<Rule> = {
        use Rule::*;
        use Assoc::*;

        PrattParser::new()
            // Lowest precedence first
            .op(Op::infix(logical_or, Left)) // ||
            .op(Op::infix(logical_and, Left)) // &&
            .op(Op::infix(eq, Left) | Op::infix(ne, Left)) // == !=
            .op(Op::infix(le, Left) | Op::infix(ge, Left) | Op::infix(lt, Left) | Op::infix(gt, Left)) // <= >= < >
            .op(Op::infix(add, Left) | Op::infix(subtract, Left)) // + -
            .op(Op::infix(multiply, Left) | Op::infix(divide, Left) | Op::infix(modulo, Left)) // * / %
            .op(Op::infix(power, Right)) // ^ ** (right-associative)
            // Highest precedence
            .op(Op::prefix(logical_not) | Op::prefix(unary_minus)) // ! - (unary)
    };
}

pub fn parse_program(input: &str) -> Result<Program> {
    let mut pairs = SludgeParser::parse(Rule::program, input)?;
    let program_pair = pairs.next().unwrap();

    let mut statements = Vec::new();
    for pair in program_pair.into_inner() {
        match pair.as_rule() {
            Rule::EOI => {}
            _ => {
                statements.push(parse_statement(pair)?);
            }
        };
    }

    Ok(Program { statements })
}

fn parse_exprs(pairs: Pairs<Rule>) -> Result<Expr> {
    PRATT_PARSER
        .map_primary(parse_expr)
        .map_infix(|lhs, op, rhs| {
            // Handle binary operations
            let bin_op = match op.as_rule() {
                Rule::add => BinOp::Add,
                Rule::subtract => BinOp::Sub,
                Rule::multiply => BinOp::Mul,
                Rule::divide => BinOp::Div,
                Rule::modulo => BinOp::Mod,
                Rule::power => BinOp::Pow,
                Rule::eq => BinOp::Eq,
                Rule::ne => BinOp::Ne,
                Rule::le => BinOp::Le,
                Rule::ge => BinOp::Ge,
                Rule::lt => BinOp::Lt,
                Rule::gt => BinOp::Gt,
                Rule::logical_and => BinOp::And,
                Rule::logical_or => BinOp::Or,
                _ => return Err(anyhow!("Unexpected infix op: {:?}", op)),
            };
            Ok(Expr::BinaryOp {
                op: bin_op,
                left: Box::new(lhs?),
                right: Box::new(rhs?),
            })
        })
        .map_prefix(|op, rhs| {
            let un_op = match op.as_rule() {
                Rule::unary_minus => UnOp::Neg,
                Rule::logical_not => UnOp::Not,
                _ => return Err(anyhow!("Unexpected prefix op: {:?}", op)),
            };
            Ok(Expr::UnaryOp {
                op: un_op,
                operand: Box::new(rhs?),
            })
        })
        .parse(pairs)
}

fn parse_expr(primary: Pair<Rule>) -> Result<Expr> {
    match primary.as_rule() {
        Rule::number => Ok(Expr::Number(primary.as_str().parse().unwrap())),
        Rule::string => {
            let s = primary.as_str();
            // Remove surrounding quotes
            Ok(Expr::String(s[1..s.len() - 1].to_string()))
        }
        Rule::identifier => Ok(Expr::Identifier(primary.as_str().to_string())),
        Rule::function_literal => {
            let inner = primary.into_inner();
            let mut arguments = Vec::new();
            let mut statement = Box::new(Expr::Number(42));
            for node in inner {
                if node.as_rule() == Rule::param_list {
                    for arg_pair in node.into_inner() {
                        if arg_pair.as_rule() == Rule::param {
                            for x in arg_pair.into_inner() {
                                arguments.push(AssignTarget::Identifier(x.as_str().to_string()))
                            }
                        }
                    }
                } else if node.as_rule() == Rule::block {
                    statement = Box::new(parse_expr(node)?);
                }
            }

            Ok(Expr::Function {
                arguments,
                statement,
            })
        }
        Rule::function_call => {
            let mut inner = primary.into_inner();
            let name = inner.next().unwrap().as_str().to_string();
            let mut args = Vec::new();

            for node in inner {
                if node.as_rule() == Rule::arg_list {
                    for arg_pair in node.into_inner() {
                        if arg_pair.as_rule() == Rule::expr {
                            args.push(parse_exprs(arg_pair.into_inner())?);
                        }
                    }
                }
            }

            Ok(Expr::FunctionCall { name, args })
        }
        Rule::block => {
            let mut statements = Vec::new();
            for inner in primary.into_inner() {
                statements.push(parse_statement(inner)?);
            }
            Ok(Expr::Block(statements))
        }
        Rule::expr => {
            // Parenthesized expression
            parse_exprs(primary.into_inner())
        }
        _ => Err(anyhow!("Unexpected primary: {:?}", primary)),
    }
}

fn parse_statement(pair: Pair<Rule>) -> Result<Statement> {
    match pair.as_rule() {
        Rule::print_stmt => {
            let mut exprs = Vec::new();
            for inner in pair.into_inner() {
                if inner.as_rule() == Rule::print_args {
                    for arg_pair in inner.into_inner() {
                        if arg_pair.as_rule() == Rule::expr {
                            exprs.push(parse_exprs(arg_pair.into_inner())?);
                        }
                    }
                }
            }
            Ok(Statement::Print(exprs))
        }
        Rule::assignment => {
            let mut inner = pair.into_inner();
            let target_pair = inner.next().unwrap();
            let op_pair = inner.next().unwrap();
            let value_pair = inner.next().unwrap();

            let target = match target_pair.as_rule() {
                Rule::identifier => AssignTarget::Identifier(target_pair.as_str().to_string()),
                _ => return Err(anyhow!("Invalid assignment target")),
            };

            let op = match op_pair.as_rule() {
                Rule::assign => AssignOp::Assign,
                _ => return Err(anyhow!("Invalid assignment operator")),
            };

            let value = parse_exprs(value_pair.into_inner())?;

            Ok(Statement::Assignment { target, op, value })
        }

        Rule::declaration => {
            let mut inner = pair.into_inner();
            let target_pair = inner.next().unwrap();
            let op_pair = inner.next().unwrap();
            let value_pair = inner.next().unwrap();
            let target = match target_pair.as_rule() {
                Rule::identifier => AssignTarget::Identifier(target_pair.as_str().to_string()),
                _ => return Err(anyhow!("Invalid assignment target")),
            };

            let op = match op_pair.as_rule() {
                Rule::assign => AssignOp::Assign,
                _ => return Err(anyhow!("Invalid assignment operator")),
            };

            let value = parse_expr(value_pair)?;

            Ok(Statement::Declaration { target, op, value })
        }

        Rule::if_stmt => {
            let mut inner = pair.into_inner();
            let condition = parse_expr(inner.next().unwrap())?;

            let then_stmt = Box::new(parse_expr(inner.next().unwrap())?);
            let else_stmt = if let Some(else_pair) = inner.next() {
                Some(Box::new(parse_expr(else_pair)?))
            } else {
                None
            };
            Ok(Statement::If {
                condition,
                then_stmt,
                else_stmt,
            })
        }
        Rule::while_stmt => {
            let mut inner = pair.into_inner();
            let condition_pair = inner.next().unwrap();
            let condition = parse_exprs(condition_pair.into_inner())?;
            let body = Box::new(parse_expr(inner.next().unwrap())?);
            Ok(Statement::While { condition, body })
        }
        Rule::for_stmt => {
            let inner = pair.into_inner();

            let mut init = None;
            let mut condition = None;
            let mut update = None;
            let mut body: Option<Box<Expr>> = None;

            for part in inner {
                match part.as_rule() {
                    Rule::assignment => {
                        if init.is_none() {
                            init = Some(Box::new(parse_statement(part)?));
                        } else {
                            update = Some(Box::new(parse_statement(part)?));
                        }
                    }
                    Rule::declaration => {
                        if init.is_none() {
                            init = Some(Box::new(parse_statement(part)?));
                        } else {
                            update = Some(Box::new(parse_statement(part)?));
                        }
                    }
                    Rule::expr => {
                        condition = Some(parse_exprs(part.into_inner())?);
                    }
                    Rule::block => {
                        body = Some(Box::new(parse_expr(part)?));
                    }
                    _ => {}
                }
            }

            match body {
                Some(body) => Ok(Statement::For {
                    init,
                    condition,
                    update,
                    body,
                }),
                None => Err(anyhow!("Body undefined in for loop")),
            }
        }
        Rule::return_stmt => Ok(Statement::Return(parse_exprs(pair.into_inner())?)),
        Rule::expr_stmt => Ok(Statement::Expression(parse_exprs(pair.into_inner())?)),
        _ => Err(anyhow!("Unsupported statement type: {:?}", pair.as_rule())),
    }
}
