use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{lexer, parser, resolve, check, lower, ir, backend, error, pretty};
use std::fmt::Write;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

/// Initialize panic hook for better error messages in the browser
#[wasm_bindgen(start)]
pub fn init() {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();
}

#[derive(Serialize, Deserialize)]
pub struct CompileResult {
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
}

/// Tokenize Mica source code
#[wasm_bindgen]
pub fn tokenize(source: &str) -> String {
    match tokenize_internal(source) {
        Ok(output) => serde_json::to_string(&CompileResult {
            success: true,
            output,
            error: None,
        }).unwrap_or_else(|e| format!("{{\"success\":false,\"error\":\"Serialization error: {}\"}}",  e)),
        Err(e) => serde_json::to_string(&CompileResult {
            success: false,
            output: String::new(),
            error: Some(e),
        }).unwrap_or_else(|e| format!("{{\"success\":false,\"error\":\"Serialization error: {}\"}}", e)),
    }
}

fn tokenize_internal(source: &str) -> Result<String, String> {
    let tokens = lexer::lex(source).map_err(|e| format!("Lexer error: {}", e))?;
    let mut output = String::new();
    for token in tokens.iter() {
        writeln!(output, "{:?}", token).map_err(|e| format!("Write error: {}", e))?;
    }
    Ok(output)
}

/// Parse Mica source code into AST
#[wasm_bindgen]
pub fn parse_ast(source: &str, pretty: bool) -> String {
    match parse_ast_internal(source, pretty) {
        Ok(output) => serde_json::to_string(&CompileResult {
            success: true,
            output,
            error: None,
        }).unwrap_or_else(|e| format!("{{\"success\":false,\"error\":\"Serialization error: {}\"}}", e)),
        Err(e) => serde_json::to_string(&CompileResult {
            success: false,
            output: String::new(),
            error: Some(e),
        }).unwrap_or_else(|e| format!("{{\"success\":false,\"error\":\"Serialization error: {}\"}}", e)),
    }
}

fn parse_ast_internal(source: &str, pretty_print: bool) -> Result<String, String> {
    let tokens = lexer::lex(source).map_err(|e| format!("Lexer error: {}", e))?;
    let ast = parser::parse(&tokens).map_err(|e| format!("Parser error: {}", e))?;
    
    if pretty_print {
        Ok(pretty::format_module(&ast))
    } else {
        Ok(format!("{:#?}", ast))
    }
}

/// Resolve names and check capabilities
#[wasm_bindgen]
pub fn resolve_code(source: &str) -> String {
    match resolve_internal(source) {
        Ok(output) => serde_json::to_string(&CompileResult {
            success: true,
            output,
            error: None,
        }).unwrap_or_else(|e| format!("{{\"success\":false,\"error\":\"Serialization error: {}\"}}", e)),
        Err(e) => serde_json::to_string(&CompileResult {
            success: false,
            output: String::new(),
            error: Some(e),
        }).unwrap_or_else(|e| format!("{{\"success\":false,\"error\":\"Serialization error: {}\"}}", e)),
    }
}

fn resolve_internal(source: &str) -> Result<String, String> {
    let tokens = lexer::lex(source).map_err(|e| format!("Lexer error: {}", e))?;
    let ast = parser::parse(&tokens).map_err(|e| format!("Parser error: {}", e))?;
    let resolution = resolve::resolve(&ast).map_err(|e| format!("Resolution error: {}", e))?;
    
    Ok(format!("{:#?}", resolution))
}

/// Check exhaustiveness and effects
#[wasm_bindgen]
pub fn check_code(source: &str) -> String {
    match check_internal(source) {
        Ok(output) => serde_json::to_string(&CompileResult {
            success: true,
            output,
            error: None,
        }).unwrap_or_else(|e| format!("{{\"success\":false,\"error\":\"Serialization error: {}\"}}", e)),
        Err(e) => serde_json::to_string(&CompileResult {
            success: false,
            output: String::new(),
            error: Some(e),
        }).unwrap_or_else(|e| format!("{{\"success\":false,\"error\":\"Serialization error: {}\"}}", e)),
    }
}

fn check_internal(source: &str) -> Result<String, String> {
    let tokens = lexer::lex(source).map_err(|e| format!("Lexer error: {}", e))?;
    let ast = parser::parse(&tokens).map_err(|e| format!("Parser error: {}", e))?;
    let resolution = resolve::resolve(&ast).map_err(|e| format!("Resolution error: {}", e))?;
    check::check(&ast, &resolution).map_err(|e| format!("Check error: {}", e))?;
    
    Ok("✓ All checks passed!".to_string())
}

/// Lower to HIR
#[wasm_bindgen]
pub fn lower_code(source: &str) -> String {
    match lower_internal(source) {
        Ok(output) => serde_json::to_string(&CompileResult {
            success: true,
            output,
            error: None,
        }).unwrap_or_else(|e| format!("{{\"success\":false,\"error\":\"Serialization error: {}\"}}", e)),
        Err(e) => serde_json::to_string(&CompileResult {
            success: false,
            output: String::new(),
            error: Some(e),
        }).unwrap_or_else(|e| format!("{{\"success\":false,\"error\":\"Serialization error: {}\"}}", e)),
    }
}

fn lower_internal(source: &str) -> Result<String, String> {
    let tokens = lexer::lex(source).map_err(|e| format!("Lexer error: {}", e))?;
    let ast = parser::parse(&tokens).map_err(|e| format!("Parser error: {}", e))?;
    let resolution = resolve::resolve(&ast).map_err(|e| format!("Resolution error: {}", e))?;
    check::check(&ast, &resolution).map_err(|e| format!("Check error: {}", e))?;
    let hir = lower::lower(&ast, &resolution).map_err(|e| format!("Lowering error: {}", e))?;
    
    Ok(format!("{:#?}", hir))
}

/// Generate IR
#[wasm_bindgen]
pub fn generate_ir(source: &str) -> String {
    match generate_ir_internal(source) {
        Ok(output) => serde_json::to_string(&CompileResult {
            success: true,
            output,
            error: None,
        }).unwrap_or_else(|e| format!("{{\"success\":false,\"error\":\"Serialization error: {}\"}}", e)),
        Err(e) => serde_json::to_string(&CompileResult {
            success: false,
            output: String::new(),
            error: Some(e),
        }).unwrap_or_else(|e| format!("{{\"success\":false,\"error\":\"Serialization error: {}\"}}", e)),
    }
}

fn generate_ir_internal(source: &str) -> Result<String, String> {
    let tokens = lexer::lex(source).map_err(|e| format!("Lexer error: {}", e))?;
    let ast = parser::parse(&tokens).map_err(|e| format!("Parser error: {}", e))?;
    let resolution = resolve::resolve(&ast).map_err(|e| format!("Resolution error: {}", e))?;
    check::check(&ast, &resolution).map_err(|e| format!("Check error: {}", e))?;
    let hir = lower::lower(&ast, &resolution).map_err(|e| format!("Lowering error: {}", e))?;
    let ir_module = ir::build_ir(&hir).map_err(|e| format!("IR generation error: {}", e))?;
    
    Ok(format!("{:#?}", ir_module))
}
