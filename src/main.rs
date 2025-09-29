use std::env;
use std::fs;
use std::path::PathBuf;

use mica::{
    Result, check, error, lexer, lower, parser, pretty,
    resolve::{self, CapabilityScope, PathKind, SymbolCategory, SymbolScope},
};

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {}", err);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let mut args = env::args().skip(1);
    let mut mode = Mode::Ast;
    let mut pretty = false;
    let mut path_arg = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--tokens" => mode = Mode::Tokens,
            "--ast" => mode = Mode::Ast,
            "--check" => mode = Mode::Check,
            "--pretty" => pretty = true,
            "--resolve" => mode = Mode::Resolve,
            "--lower" => mode = Mode::Lower,
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
            if pretty {
                println!("{}", pretty::module_to_string(&module));
            } else {
                println!("{:#?}", module);
            }
        }
        Mode::Check => {
            let module = parser::parse_module(&source)?;
            let diags = check::check_exhaustiveness(&module);
            if diags.is_empty() {
                println!("ok");
            } else {
                for d in diags {
                    println!("warning: {}", d.message);
                }
            }
        }
        Mode::Resolve => {
            let module = parser::parse_module(&source)?;
            let resolved = resolve::resolve_module(&module);

            println!("module: {}", resolved.module_path.join("."));

            if !resolved.imports.is_empty() {
                println!("imports:");
                for import in &resolved.imports {
                    match &import.alias {
                        Some(alias) => println!("  use {} as {}", import.path.join("::"), alias),
                        None => println!("  use {}", import.path.join("::")),
                    }
                }
            }

            if !resolved.adts.is_empty() {
                println!("types:");
                for (adt, variants) in resolved.adts.iter() {
                    if variants.is_empty() {
                        println!("  type {}", adt);
                    } else {
                        println!("  type {} = {}", adt, variants.join(" | "));
                    }
                }
            }

            let mut functions = resolved
                .symbols
                .iter()
                .filter_map(|symbol| match &symbol.category {
                    SymbolCategory::Function { is_public } => Some((symbol, *is_public)),
                    _ => None,
                })
                .collect::<Vec<_>>();
            functions.sort_by(|(a, _), (b, _)| a.name.cmp(&b.name));

            if !functions.is_empty() {
                println!("functions:");
                for (symbol, is_public) in functions {
                    let visibility = if is_public { "pub " } else { "" };
                    let scope = match &symbol.scope {
                        SymbolScope::Module(path) => path.join("::"),
                        SymbolScope::Function {
                            module_path,
                            function,
                        } => {
                            format!("{}::{}", module_path.join("::"), function)
                        }
                        SymbolScope::TypeAlias {
                            module_path,
                            type_name,
                        } => {
                            format!("{}::{}", module_path.join("::"), type_name)
                        }
                    };
                    println!("  {visibility}fn {} (scope: {})", symbol.name, scope);
                }
            }

            if !resolved.capabilities.is_empty() {
                println!("capabilities:");
                for binding in &resolved.capabilities {
                    let scope = match &binding.scope {
                        CapabilityScope::Function {
                            module_path,
                            function,
                        } => {
                            format!("fn {}::{}", module_path.join("::"), function)
                        }
                        CapabilityScope::TypeAlias {
                            module_path,
                            type_name,
                        } => {
                            format!("type {}::{}", module_path.join("::"), type_name)
                        }
                    };
                    println!("  {scope} requires {}", binding.name);
                }
            }

            if !resolved.resolved_paths.is_empty() {
                println!("paths:");
                for path in &resolved.resolved_paths {
                    let kind = match path.kind {
                        PathKind::Type => "type",
                        PathKind::Value => "value",
                        PathKind::Variant => "variant",
                    };
                    if let Some(symbol) = &path.resolved {
                        println!(
                            "  {} -> {} ({:?})",
                            path.segments.join("::"),
                            symbol.name,
                            symbol.category
                        );
                    } else {
                        println!("  {} -> <unresolved {}>", path.segments.join("::"), kind);
                    }
                }
            }
        }
        Mode::Lower => {
            let module = parser::parse_module(&source)?;
            let h = lower::lower_module(&module);
            println!("{}", lower::hir_to_string(&h));
        }
    }

    Ok(())
}

#[derive(Debug, Clone, Copy)]
enum Mode {
    Tokens,
    Ast,
    Check,
    Resolve,
    Lower,
}
