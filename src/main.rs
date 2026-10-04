use std::io::Write;

fn repl() -> std::io::Result<()> {
    let mut input = String::new();

    loop {
        print!("forge> ");
        std::io::stdout().flush()?;

        input.clear();

        let n = std::io::stdin().read_line(&mut input)?;

        if n == 0 {
            break;
        }

        let input = input.trim();
        let tokens = forge::lexer::Lexer::new(input).tokenize();
        let statements = forge::parser::Parser::new(tokens).parse();

        dbg!(statements);
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    match args.len() {
        1 => {
            repl()?;
        }
        _ => {
            panic!("Invalid usage");
        }
    }

    Ok(())
}
