use anyhow::Result;
use std::fs;

mod ast;
mod interpreter;

use crate::ast::parser::parse_program;
use crate::interpreter::variable_scope::VariableScope;

use interpreter::*;
use std::cell::RefCell;
use std::io::{BufWriter, Write};
use std::rc::Rc;

fn main() -> Result<()> {
    let file = "main.sludge";
    let contents = fs::read_to_string(file)
        .map_err(|e| anyhow::anyhow!("Failed to read program file '{}': {}", file, e))?;

    let program = parse_program(&contents).map_err(|e| anyhow::anyhow!("Parse error: {}", e))?;

    let json = serde_json::to_string_pretty(&program).expect("Failed to serialize pretty");
    println!("{}", json);

    let writer = Rc::new(RefCell::new(BufWriter::new(std::io::stdout())));

    let interpreter = Interpreter::new(VariableScope::new(), writer.clone());
    interpreter
        .run_program(&program)
        .map_err(|e| anyhow::anyhow!("Runtime error: {}", e))?;

    writer.borrow_mut().flush()?;

    Ok(())
}
