mod ast;
mod error;
mod lexer;
mod parser;
mod token;

use std::env;
use std::fs;
use std::path::PathBuf;

use error::Result;

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {}", err);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let mut args = env::args().skip(1);
    let mut mode = Mode::Ast;
    let mut path_arg = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--tokens" => mode = Mode::Tokens,
            "--ast" => mode = Mode::Ast,
            _ => {
                path_arg = Some(PathBuf::from(arg));
                for extra in args {
                    if path_arg.is_some() {
                        return Err(error::Error::parse(
                            None,
                            format!("unexpected extra argument '{}'", extra),
                        ));
                    }
                }
                break;
            }
        }
    }

    let path = path_arg.ok_or_else(|| error::Error::parse(None, "missing input file"))?;
    let source = fs::read_to_string(&path).map_err(|e| error::Error::lex(None, e.to_string()))?;

    match mode {
        Mode::Tokens => {
            let tokens = lexer::lex(&source)?;
            for token in tokens {
                println!("{:?}", token);
            }
        }
        Mode::Ast => {
            let module = parser::parse_module(&source)?;
            println!("{:#?}", module);
        }
    }

    Ok(())
}

#[derive(Debug, Clone, Copy)]
enum Mode {
    Tokens,
    Ast,
}
