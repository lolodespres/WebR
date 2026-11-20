//! La commande `webru`.
//!
//! Pour l'instant une seule sous-commande, pour voir le lexer travailler :
//!
//! ```text
//! cargo webru tokens examples/bienvenue.wru
//! ```

use std::env;
use std::fs;
use std::process::ExitCode;

use webru_lexer::Span;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    match args.as_slice() {
        [command, path] if command == "tokens" => print_tokens(path),
        _ => {
            eprintln!("usage : webru tokens <fichier.wru>");
            ExitCode::from(2)
        }
    }
}

fn print_tokens(path: &str) -> ExitCode {
    let source = match fs::read_to_string(path) {
        Ok(source) => source,
        Err(err) => {
            eprintln!("impossible de lire {path} : {err}");
            return ExitCode::FAILURE;
        }
    };
    match webru_lexer::tokenize(&source) {
        Ok(tokens) => {
            for token in tokens {
                println!(
                    "{:>4}:{:<3} {:?}",
                    token.span.line, token.span.col, token.kind
                );
            }
            ExitCode::SUCCESS
        }
        Err(err) => {
            report_error(path, &source, err.span, &err.message);
            ExitCode::FAILURE
        }
    }
}

/// Affiche l'erreur avec la ligne fautive et un `^` sous la colonne.
fn report_error(path: &str, source: &str, span: Span, message: &str) {
    eprintln!("{path}:{}:{} : erreur : {message}", span.line, span.col);
    if let Some(line) = source.lines().nth(span.line.saturating_sub(1)) {
        eprintln!("     | {line}");
        eprintln!("     | {}^", " ".repeat(span.col.saturating_sub(1)));
    }
}
