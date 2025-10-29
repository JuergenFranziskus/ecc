use ecc::{lexer::Lexer, parser::Parser};
use std::process::Command;

fn main() {
    const SRC_FILE: &str = "main.c";
    let src = invoke_preprocessor(SRC_FILE).unwrap();
    println!("--------------------------------------------------");
    print!("{src}");
    println!("--------------------------------------------------\n\n");

    let (tokens, files) = Lexer::new(&src).lex();
    for &token in &tokens {
        let file = &files[token.at.file];
        println!(
            "{} {}:{}\t{:?}",
            file, token.at.line, token.at.column, token.kind
        );
    }

    let (ast, parse_errs) = Parser::new(&tokens).parse();
    if !parse_errs.is_empty() {
        eprintln!("Encountered {} parsing errors:", parse_errs.len());
    }
    for parse_err in &parse_errs {
        eprintln!("    {parse_err:?}");
    }
    let Ok(ast) = ast else {
        eprintln!("Cannot continue compilation process");
        return;
    };

    println!("{ast:#?}");
}

fn invoke_preprocessor(file: &str) -> Result<String, ()> {
    let out = Command::new("gcc")
        .arg("-E")
        .arg("-xc")
        .arg("-std=c23")
        .arg("-nostdinc")
        .arg("-undef")
        .arg(file)
        .arg("-")
        .output()
        .unwrap();
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        eprintln!("Preprocessor failed:");
        eprintln!("{stderr}");
        return Err(());
    }

    match String::from_utf8(out.stdout) {
        Ok(out) => Ok(out),
        Err(err) => {
            eprintln!("Preprocessor output is not UTF-8: {err}");
            Err(())
        }
    }
}
