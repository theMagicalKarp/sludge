use anyhow::{Context, Result, anyhow};
use clap::{Parser, Subcommand};
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;
use std::{
    cell::RefCell,
    fs,
    io::{BufWriter, Write},
    path::PathBuf,
    rc::Rc,
};
use yansi::Paint;

mod ast;
mod interpreter;

use crate::ast::Statement;
use crate::ast::parser::{parse_program, parse_stmt, underline_error};
use crate::interpreter::Interpreter;
use crate::interpreter::variable_scope::VariableScope;

#[derive(Parser, Debug)]
#[command(name = "sludge", version, about = "Sludge language CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Parse a file and execute it
    Run { file: PathBuf },
    /// Start an interactive Read–Eval–Print loop
    Repl,
    /// Parse a file and print its AST as pretty JSON
    Ast { file: PathBuf },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Run { file } => run_file(&file),
        Commands::Repl => run_repl(),
        Commands::Ast { file } => print_ast(&file),
    }
}

fn run_file(path: &PathBuf) -> Result<()> {
    let contents = fs::read_to_string(path)
        .with_context(|| format!("Failed to read program file '{}'.", path.display()))?;

    let program = parse_program(&contents).map_err(|e| anyhow!("Parse error: {}", e))?;

    let writer = Rc::new(RefCell::new(BufWriter::new(std::io::stdout())));
    let interpreter = Interpreter::new(VariableScope::new(), writer.clone());

    interpreter
        .run_program(&program)
        .map_err(|e| anyhow!("Runtime error: {}", e))?;

    writer.borrow_mut().flush().ok();
    Ok(())
}

fn print_ast(path: &PathBuf) -> Result<()> {
    let contents = fs::read_to_string(path)
        .with_context(|| format!("Failed to read program file '{}'.", path.display()))?;

    let program = parse_program(&contents).map_err(|e| anyhow!("Parse error: {}", e))?;

    // Requires your AST nodes to derive serde::Serialize
    let json = serde_json::to_string_pretty(&program)
        .map_err(|e| anyhow!("AST is not serializable (did you derive Serialize?): {}", e))?;
    println!("{}", json);
    Ok(())
}

fn run_repl() -> Result<()> {
    let mut rl = DefaultEditor::new()?;
    let writer = Rc::new(RefCell::new(BufWriter::new(std::io::stdout())));
    let interpreter = Interpreter::new(VariableScope::new(), writer.clone());
    let prompt = Paint::cyan(">>> ").to_string();

    println!(
        "{}",
        Paint::new("🛢️ Welcome to sludge REPL. Ctrl-D to exit, :help for commands.").bold()
    );

    loop {
        let line = rl.readline(&prompt);
        match line {
            Ok(input) => {
                let trimmed = input.trim();
                if trimmed.is_empty() {
                    continue;
                }
                match parse_stmt(trimmed) {
                    Ok(stmts) => {
                        for st in stmts {
                            match st? {
                                Statement::Expression(e) => {
                                    match interpreter.execute_statement(&Statement::Expression(e)) {
                                        Ok(val) => println!("{val}"),
                                        Err(e) => println!("Eval error: {e}"),
                                    }
                                }
                                other => {
                                    if let Err(e) = interpreter.execute_statement(&other) {
                                        println!("Eval error: {e}");
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        println!("Parse error:\n{}", underline_error(trimmed, &e));
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                // Ctrl-C
                println!("^C");
                continue;
            }
            Err(ReadlineError::Eof) => break, // Ctrl-D
            Err(err) => {
                eprintln!("Readline error: {err}");
                break;
            }
        }
    }

    println!("Goodbye! 👋");
    writer.borrow_mut().flush().ok();
    Ok(())
}
