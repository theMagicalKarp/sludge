use anyhow::Result;
use std::fs;

mod ast;
mod interpreter;
mod parser;

use crate::interpreter::variable_scope::VariableScope;
use interpreter::*;
use parser::*;

fn main() -> Result<()> {
    let file = "main.sludge";
    let contents = fs::read_to_string(file)
        .map_err(|e| anyhow::anyhow!("Failed to read program file '{}': {}", file, e))?;

    let program = parse_program(&contents).map_err(|e| anyhow::anyhow!("Parse error: {}", e))?;

    let json = serde_json::to_string_pretty(&program).expect("Failed to serialize pretty");
    println!("{}", json);

    let mut interpreter = Interpreter::new(VariableScope::new());
    interpreter
        .run_program(&program)
        .map_err(|e| anyhow::anyhow!("Runtime error: {}", e))?;
    Ok(())
}
