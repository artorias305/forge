use std::{fs, io::Write};

use clap::Parser;
use forge::{interpreter::Interpreter, lexer::Lexer, parser::Parser as ForgeParser};

#[derive(Debug, clap::Parser)]
#[command(
    name = "forge",
    version,
    about = "A dynamically typed interpreted programming language",
    long_about = "Run Forge source files in order using one shared interpreter, or start the REPL when no files are provided."
)]
struct Cli {
    #[arg(value_name = "FILE")]
    files: Vec<String>,
}

fn run_source(
    source_name: &str,
    source: &str,
    interpreter: &mut Interpreter,
) -> Result<(), String> {
    let tokens = Lexer::new(source)
        .tokenize()
        .map_err(|error| format!("{source_name}: {error}"))?;
    let statements = ForgeParser::new(tokens)
        .parse()
        .map_err(|error| format!("{source_name}: {error}"))?;
    interpreter
        .execute(&statements)
        .map_err(|error| format!("{source_name}: {error}"))
}

fn repl() -> Result<(), Box<dyn std::error::Error>> {
    let mut input = String::new();
    let mut interpreter = Interpreter::default();

    loop {
        print!("forge> ");
        std::io::stdout().flush()?;
        input.clear();

        if std::io::stdin().read_line(&mut input)? == 0 {
            break;
        }

        if let Err(error) = run_source("<stdin>", input.trim(), &mut interpreter) {
            eprintln!("{error}");
        }
    }

    Ok(())
}

fn run_files(files: &[String]) -> Result<(), String> {
    let mut interpreter = Interpreter::default();

    for file in files {
        let source = fs::read_to_string(file)
            .map_err(|error| format!("{file}: failed to read file: {error}"))?;
        run_source(file, &source, &mut interpreter)?;
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    if cli.files.is_empty() {
        repl()?;
    } else if let Err(error) = run_files(&cli.files) {
        return Err(error.into());
    }

    Ok(())
}
