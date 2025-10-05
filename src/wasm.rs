use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::{check, interpreter, ir, lexer, lower, parser, pretty, resolve};
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
        })
        .unwrap_or_else(|e| {
            format!(
                "{{\"success\":false,\"error\":\"Serialization error: {}\"}}",
                e
            )
        }),
        Err(e) => serde_json::to_string(&CompileResult {
            success: false,
            output: String::new(),
            error: Some(e),
        })
        .unwrap_or_else(|e| {
            format!(
                "{{\"success\":false,\"error\":\"Serialization error: {}\"}}",
                e
            )
        }),
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
        })
        .unwrap_or_else(|e| {
            format!(
                "{{\"success\":false,\"error\":\"Serialization error: {}\"}}",
                e
            )
        }),
        Err(e) => serde_json::to_string(&CompileResult {
            success: false,
            output: String::new(),
            error: Some(e),
        })
        .unwrap_or_else(|e| {
            format!(
                "{{\"success\":false,\"error\":\"Serialization error: {}\"}}",
                e
            )
        }),
    }
}

fn parse_ast_internal(source: &str, pretty_print: bool) -> Result<String, String> {
    let ast = parser::parse_module(source).map_err(|e| format!("Parser error: {}", e))?;

    if pretty_print {
        Ok(pretty::module_to_string(&ast))
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
        })
        .unwrap_or_else(|e| {
            format!(
                "{{\"success\":false,\"error\":\"Serialization error: {}\"}}",
                e
            )
        }),
        Err(e) => serde_json::to_string(&CompileResult {
            success: false,
            output: String::new(),
            error: Some(e),
        })
        .unwrap_or_else(|e| {
            format!(
                "{{\"success\":false,\"error\":\"Serialization error: {}\"}}",
                e
            )
        }),
    }
}

fn resolve_internal(source: &str) -> Result<String, String> {
    let ast = parser::parse_module(source).map_err(|e| format!("Parser error: {}", e))?;
    let resolution = resolve::resolve_module(&ast);

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
        })
        .unwrap_or_else(|e| {
            format!(
                "{{\"success\":false,\"error\":\"Serialization error: {}\"}}",
                e
            )
        }),
        Err(e) => serde_json::to_string(&CompileResult {
            success: false,
            output: String::new(),
            error: Some(e),
        })
        .unwrap_or_else(|e| {
            format!(
                "{{\"success\":false,\"error\":\"Serialization error: {}\"}}",
                e
            )
        }),
    }
}

fn check_internal(source: &str) -> Result<String, String> {
    let ast = parser::parse_module(source).map_err(|e| format!("Parser error: {}", e))?;
    let check_result = check::check_module(&ast);

    if check_result.diagnostics.is_empty() {
        Ok("✓ All checks passed!".to_string())
    } else {
        let mut output = String::from("Diagnostics:\n");
        for diag in &check_result.diagnostics {
            writeln!(output, "  - {}", diag.message).unwrap();
        }
        Ok(output)
    }
}

/// Lower to HIR
#[wasm_bindgen]
pub fn lower_code(source: &str) -> String {
    match lower_internal(source) {
        Ok(output) => serde_json::to_string(&CompileResult {
            success: true,
            output,
            error: None,
        })
        .unwrap_or_else(|e| {
            format!(
                "{{\"success\":false,\"error\":\"Serialization error: {}\"}}",
                e
            )
        }),
        Err(e) => serde_json::to_string(&CompileResult {
            success: false,
            output: String::new(),
            error: Some(e),
        })
        .unwrap_or_else(|e| {
            format!(
                "{{\"success\":false,\"error\":\"Serialization error: {}\"}}",
                e
            )
        }),
    }
}

fn lower_internal(source: &str) -> Result<String, String> {
    let ast = parser::parse_module(source).map_err(|e| format!("Parser error: {}", e))?;
    let hir = lower::lower_module(&ast);

    Ok(lower::hir_to_string(&hir))
}

/// Generate IR
#[wasm_bindgen]
pub fn generate_ir(source: &str) -> String {
    match generate_ir_internal(source) {
        Ok(output) => serde_json::to_string(&CompileResult {
            success: true,
            output,
            error: None,
        })
        .unwrap_or_else(|e| {
            format!(
                "{{\"success\":false,\"error\":\"Serialization error: {}\"}}",
                e
            )
        }),
        Err(e) => serde_json::to_string(&CompileResult {
            success: false,
            output: String::new(),
            error: Some(e),
        })
        .unwrap_or_else(|e| {
            format!(
                "{{\"success\":false,\"error\":\"Serialization error: {}\"}}",
                e
            )
        }),
    }
}

fn generate_ir_internal(source: &str) -> Result<String, String> {
    let ast = parser::parse_module(source).map_err(|e| format!("Parser error: {}", e))?;
    let hir = lower::lower_module(&ast);
    let ir_module = ir::lower_module(&hir);

    Ok(format!("{:#?}", ir_module))
}

/// Run Mica code
#[wasm_bindgen]
pub fn run_code(source: &str) -> String {
    match run_internal(source) {
        Ok(output) => serde_json::to_string(&CompileResult {
            success: true,
            output,
            error: None,
        })
        .unwrap_or_else(|e| {
            format!(
                "{{\"success\":false,\"error\":\"Serialization error: {}\"}}",
                e
            )
        }),
        Err(e) => serde_json::to_string(&CompileResult {
            success: false,
            output: String::new(),
            error: Some(e),
        })
        .unwrap_or_else(|e| {
            format!(
                "{{\"success\":false,\"error\":\"Serialization error: {}\"}}",
                e
            )
        }),
    }
}

fn run_internal(source: &str) -> Result<String, String> {
    let ast = parser::parse_module(source).map_err(|e| format!("Parser error: {}", e))?;
    let hir = lower::lower_module(&ast);
    let ir_module = ir::lower_module(&hir);

    let mut interp = interpreter::Interpreter::new(ir_module);
    interp.run()
}
