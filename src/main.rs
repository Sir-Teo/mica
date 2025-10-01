use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use mica::{
    Result, backend, check, error, ir, lexer, lower, parser, pretty,
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
    let mut output_path: Option<PathBuf> = None;
    let mut path_arg = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--tokens" => mode = Mode::Tokens,
            "--ast" => mode = Mode::Ast,
            "--check" => mode = Mode::Check,
            "--pretty" => pretty = true,
            "--resolve" => mode = Mode::Resolve,
            "--lower" => mode = Mode::Lower,
            "--ir" => mode = Mode::Ir,
            "--llvm" | "--emit-llvm" => mode = Mode::Llvm,
            "--build" => mode = Mode::Build { output: None },
            "--run" => mode = Mode::Run { output: None },
            "--out" => {
                let value = args
                    .next()
                    .ok_or_else(|| error::Error::parse(None, "expected output path after --out"))?;
                output_path = Some(PathBuf::from(value));
            }
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

    match &mut mode {
        Mode::Build { output } | Mode::Run { output } => {
            if output_path.is_some() {
                *output = output_path.clone();
            }
        }
        _ => {}
    }

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
            let result = check::check_module(&module);
            if result.diagnostics.is_empty() {
                println!("ok");
            } else {
                for d in result.diagnostics {
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
        Mode::Ir => {
            let module = parser::parse_module(&source)?;
            let hir = lower::lower_module(&module);
            let typed = ir::lower_module(&hir);
            let backend = backend::text::TextBackend::default();
            let output = backend::run(&backend, &typed, &backend::BackendOptions::default())
                .map_err(|err| error::Error::parse(None, err.to_string()))?;
            println!("{}", output);
        }
        Mode::Llvm => {
            let module = parser::parse_module(&source)?;
            let hir = lower::lower_module(&module);
            let typed = ir::lower_module(&hir);
            let backend = backend::llvm::LlvmBackend::default();
            let output = backend::run(&backend, &typed, &backend::BackendOptions::default())
                .map_err(|err| error::Error::parse(None, err.to_string()))?;
            println!("{}", output.as_str());
        }
        Mode::Build { output } => {
            let module = parser::parse_module(&source)?;
            let hir = lower::lower_module(&module);
            let typed = ir::lower_module(&hir);
            let backend = backend::native::NativeBackend::default();
            let artifact = backend::run(&backend, &typed, &backend::BackendOptions::default())
                .map_err(|err| error::Error::parse(None, err.to_string()))?;
            let mut default_path = path.clone();
            default_path.set_extension("bin");
            let target_path = output.unwrap_or(default_path);
            artifact
                .link_executable(&target_path)
                .map_err(|err| error::Error::parse(None, err.to_string()))?;
            println!("built {}", target_path.display());
        }
        Mode::Run { output } => {
            let module = parser::parse_module(&source)?;
            let hir = lower::lower_module(&module);
            let typed = ir::lower_module(&hir);
            let backend = backend::native::NativeBackend::default();
            let artifact = backend::run(&backend, &typed, &backend::BackendOptions::default())
                .map_err(|err| error::Error::parse(None, err.to_string()))?;
            let (exe_path, cleanup_path);
            if let Some(path) = output {
                artifact
                    .link_executable(&path)
                    .map_err(|err| error::Error::parse(None, err.to_string()))?;
                exe_path = path.clone();
                cleanup_path = None;
            } else {
                let mut path_buf = env::temp_dir();
                let nanos = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos();
                path_buf.push(format!("mica-run-{nanos}"));
                artifact
                    .link_executable(&path_buf)
                    .map_err(|err| error::Error::parse(None, err.to_string()))?;
                exe_path = path_buf;
                cleanup_path = Some(exe_path.clone());
            }

            let status = Command::new(&exe_path)
                .status()
                .map_err(|err| error::Error::parse(None, err.to_string()))?;
            if !status.success() {
                return Err(error::Error::parse(
                    None,
                    format!("program exited with status {status}"),
                ));
            }

            if let Some(path) = cleanup_path {
                fs::remove_file(&path).ok();
            }
        }
    }

    Ok(())
}

#[derive(Debug, Clone)]
enum Mode {
    Tokens,
    Ast,
    Check,
    Resolve,
    Lower,
    Ir,
    Llvm,
    Build { output: Option<PathBuf> },
    Run { output: Option<PathBuf> },
}
