use ecc::lexer::Lexer;
use std::process::Command;

fn main() {
    const SRC_FILE: &str = "main.c";
    let src = invoke_preprocessor(SRC_FILE).unwrap();
    println!("--------------------------------------------------");
    print!("{src}");
    println!("--------------------------------------------------\n\n");

    let (tokens, files) = Lexer::new(&src).lex();
    for token in tokens {
        let file = &files[token.at.file];
        println!(
            "{} {}:{}\t{:?}",
            file, token.at.line, token.at.column, token.kind
        );
    }
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
