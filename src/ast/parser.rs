use crate::ast::*;
use anyhow::{Result, anyhow};
use pest::Parser;
use pest::error::InputLocation;
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
            .op(Op::infix(add, Left) | Op::infix(subtract, Left))  // + -
            .op(Op::infix(multiply, Left) | Op::infix(divide, Left) | Op::infix(modulo, Left)) // * / %
            .op(Op::infix(power, Right))           // ^ or **
            // Highest precedence
            .op(Op::prefix(logical_not) | Op::prefix(unary_minus)) // ! -
            .op(Op::postfix(member_access) | Op::postfix(call_suffix))
    };
}

pub fn underline_error(input: &str, err: &pest::error::Error<Rule>) -> String {
    if let InputLocation::Span((start, end)) = err.location.clone() {
        let mut out = String::new();
        out.push_str(input);
        out.push('\n');
        for i in 0..input.len() {
            if i >= start && i < end {
                out.push('^');
            } else if input.is_char_boundary(i) {
                out.push(' ');
            }
        }
        out
    } else {
        err.to_string()
    }
}

pub fn parse_program(input: &str) -> Result<Program> {
    let mut pairs = SludgeParser::parse(Rule::program, input)?;
    let program_pair = pairs.next().unwrap();

    let mut statements = Vec::new();
    for pair in program_pair.into_inner() {
        match pair.as_rule() {
            Rule::EOI => {}
            _ => statements.push(parse_statement(pair)?),
        };
    }

    Ok(Program { statements })
}

pub fn parse_stmt(
    input: &str,
) -> Result<impl Iterator<Item = Result<Statement>>, Box<pest::error::Error<Rule>>> {
    let pairs = SludgeParser::parse(Rule::statement, input)?;
    Ok(pairs.map(|pair| parse_statement(pair)))
}

fn parse_exprs(pairs: Pairs<Rule>) -> Result<Expr> {
    PRATT_PARSER
        .map_primary(parse_expr)
        .map_infix(|lhs, op, rhs| {
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
        .map_postfix(|lhs, postfix| {
            let target = Box::new(lhs?);
            match postfix.as_rule() {
                Rule::call_suffix => {
                    let args = postfix
                        .into_inner()
                        .map(parse_expr)
                        .collect::<Result<Vec<_>, _>>()?;
                    Ok(Expr::Call { target, args })
                }
                Rule::member_access => {
                    let field = postfix
                        .into_inner()
                        .next()
                        .ok_or_else(|| anyhow!("Missing field name in member access"))?
                        .as_str()
                        .to_string();
                    Ok(Expr::Member { target, field })
                }
                _ => Err(anyhow!("Unexpected postfix: {:?}", postfix)),
            }
        })
        .parse(pairs)
}

fn parse_expr(primary: Pair<Rule>) -> Result<Expr> {
    match primary.as_rule() {
        Rule::number => Ok(Expr::Number(primary.as_str().parse()?)),
        Rule::boolean => {
            let text = primary.as_str();
            match text {
                "true" => Ok(Expr::Boolean(true)),
                "false" => Ok(Expr::Boolean(false)),
                _ => Err(anyhow!("Invalid boolean literal: {}", text)),
            }
        }
        Rule::string => {
            let s = primary.as_str();
            Ok(Expr::String(s[1..s.len() - 1].to_string()))
        }
        Rule::identifier => Ok(Expr::Identifier(primary.as_str().to_string())),
        Rule::function_literal => {
            let inner = primary.into_inner();
            let mut arguments = Vec::new();
            let mut statement: Option<Box<Expr>> = None;
            for node in inner {
                if node.as_rule() == Rule::param {
                    arguments.push(AssignTarget::Identifier(node.as_str().to_string()));
                } else if node.as_rule() == Rule::block {
                    statement = Some(Box::new(parse_expr(node)?));
                }
            }

            match statement {
                Some(statement) => Ok(Expr::Function {
                    arguments,
                    statement,
                }),
                None => Err(anyhow!("Function literal missing body")),
            }
        }
        Rule::tuple_expr => {
            let values = primary
                .into_inner()
                .map(parse_expr)
                .collect::<Result<Vec<_>>>()?;
            Ok(Expr::Tuple { values })
        }
        Rule::block => {
            let mut statements = Vec::new();
            for inner in primary.into_inner() {
                statements.push(parse_statement(inner)?);
            }
            Ok(Expr::Block(statements))
        }
        Rule::expr => parse_exprs(primary.into_inner()),
        _ => Err(anyhow!("Unexpected primary: {:?}", primary.as_rule())),
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
                            exprs.push(
                                parse_exprs(arg_pair.into_inner()).map_err(|e| {
                                    anyhow!("Failed to parse print argument: {}", e)
                                })?,
                            );
                        }
                    }
                }
            }
            Ok(Statement::Print(exprs))
        }

        Rule::assignment => {
            let mut inner = pair.into_inner();
            let target_pair = inner
                .next()
                .ok_or_else(|| anyhow!("Missing assignment target"))?;
            let op_pair = inner
                .next()
                .ok_or_else(|| anyhow!("Missing assignment operator"))?;
            let value_pair = inner
                .next()
                .ok_or_else(|| anyhow!("Missing assignment value"))?;

            let target = match target_pair.as_rule() {
                Rule::identifier => AssignTarget::Identifier(target_pair.as_str().to_string()),
                other => {
                    return Err(anyhow!(
                        "Invalid assignment target: expected identifier, got {:?}",
                        other
                    ));
                }
            };

            let op = match op_pair.as_rule() {
                Rule::assign => AssignOp::Assign,
                other => {
                    return Err(anyhow!(
                        "Invalid assignment operator: expected '=', got {:?}",
                        other
                    ));
                }
            };

            let value = parse_exprs(value_pair.into_inner())
                .map_err(|e| anyhow!("Failed to parse assignment value: {}", e))?;

            Ok(Statement::Assignment { target, op, value })
        }

        Rule::declaration => {
            let mut inner = pair.into_inner();
            let target_pair = inner
                .next()
                .ok_or_else(|| anyhow!("Missing declaration target"))?;
            let op_pair = inner
                .next()
                .ok_or_else(|| anyhow!("Missing declaration operator"))?;
            let value_pair = inner
                .next()
                .ok_or_else(|| anyhow!("Missing declaration value"))?;

            let target = match target_pair.as_rule() {
                Rule::identifier => AssignTarget::Identifier(target_pair.as_str().to_string()),
                other => {
                    return Err(anyhow!(
                        "Invalid declaration target: expected identifier, got {:?}",
                        other
                    ));
                }
            };

            let op = match op_pair.as_rule() {
                Rule::assign => AssignOp::Assign,
                other => {
                    return Err(anyhow!(
                        "Invalid declaration operator: expected '=', got {:?}",
                        other
                    ));
                }
            };

            let value = parse_expr(value_pair)
                .map_err(|e| anyhow!("Failed to parse declaration value: {}", e))?;

            Ok(Statement::Declaration { target, op, value })
        }

        Rule::if_stmt => {
            let mut inner = pair.into_inner();

            let condition_pair = inner
                .next()
                .ok_or_else(|| anyhow!("Missing condition in if-statement"))?;
            let condition = parse_expr(condition_pair)
                .map_err(|e| anyhow!("Failed to parse if-condition: {}", e))?;

            let then_pair = inner
                .next()
                .ok_or_else(|| anyhow!("Missing 'then' block in if-statement"))?;
            let then_stmt = Box::new(
                parse_expr(then_pair)
                    .map_err(|e| anyhow!("Failed to parse 'then' block: {}", e))?,
            );

            let else_stmt = if let Some(else_pair) = inner.next() {
                match else_pair.as_rule() {
                    Rule::if_stmt => Some(Box::new(Expr::Block(vec![parse_statement(else_pair)?]))),
                    Rule::block => Some(Box::new(parse_expr(else_pair)?)),
                    other => {
                        return Err(anyhow!(
                            "Invalid else clause: expected 'if' or block, got {:?}",
                            other
                        ));
                    }
                }
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

            let condition_pair = inner
                .next()
                .ok_or_else(|| anyhow!("Missing condition in while loop"))?;
            let condition = parse_exprs(condition_pair.into_inner())
                .map_err(|e| anyhow!("Failed to parse while condition: {}", e))?;

            let body_pair = inner
                .next()
                .ok_or_else(|| anyhow!("Missing body in while loop"))?;
            let body = Box::new(
                parse_expr(body_pair).map_err(|e| anyhow!("Failed to parse while body: {}", e))?,
            );

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
                    Rule::assignment | Rule::declaration => {
                        if init.is_none() {
                            init = Some(Box::new(
                                parse_statement(part)
                                    .map_err(|e| anyhow!("Failed to parse for-init: {}", e))?,
                            ));
                        } else {
                            update = Some(Box::new(
                                parse_statement(part)
                                    .map_err(|e| anyhow!("Failed to parse for-update: {}", e))?,
                            ));
                        }
                    }
                    Rule::expr => {
                        condition = Some(
                            parse_exprs(part.into_inner())
                                .map_err(|e| anyhow!("Failed to parse for-condition: {}", e))?,
                        );
                    }
                    Rule::block => {
                        body = Some(Box::new(
                            parse_expr(part)
                                .map_err(|e| anyhow!("Failed to parse for-body: {}", e))?,
                        ));
                    }
                    other => {
                        return Err(anyhow!("Unexpected element in for loop: {:?}", other));
                    }
                }
            }

            match body {
                Some(body) => Ok(Statement::For {
                    init,
                    condition,
                    update,
                    body,
                }),
                None => Err(anyhow!("For loop missing body")),
            }
        }

        Rule::return_stmt => Ok(Statement::Return(
            parse_exprs(pair.into_inner())
                .map_err(|e| anyhow!("Failed to parse return value: {}", e))?,
        )),

        Rule::expr_stmt => Ok(Statement::Expression(
            parse_exprs(pair.into_inner())
                .map_err(|e| anyhow!("Failed to parse expression statement: {}", e))?,
        )),

        other => Err(anyhow!(
            "Unsupported or unexpected statement type: {:?}",
            other
        )),
    }
}
