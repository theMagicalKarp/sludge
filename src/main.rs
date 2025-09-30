use anyhow::Result;
use std::fs;

mod ast;
mod interpreter;

use crate::ast::parser::parse_program;
use crate::interpreter::variable_scope::VariableScope;

use interpreter::*;
use std::io::{self, BufWriter, Write};

fn main() -> Result<()> {
    let file = "main.sludge";
    let contents = fs::read_to_string(file)
        .map_err(|e| anyhow::anyhow!("Failed to read program file '{}': {}", file, e))?;

    let program = parse_program(&contents).map_err(|e| anyhow::anyhow!("Parse error: {}", e))?;

    let json = serde_json::to_string_pretty(&program).expect("Failed to serialize pretty");
    println!("{}", json);

    let stdout = io::stdout();
    let handle = stdout.lock(); // lock() is recommended for efficiency
    let mut writer = BufWriter::new(handle);

    let mut interpreter = Interpreter::new(VariableScope::new(), &mut writer);
    interpreter
        .run_program(&program)
        .map_err(|e| anyhow::anyhow!("Runtime error: {}", e))?;
    writer.flush()?;
    Ok(())
}
