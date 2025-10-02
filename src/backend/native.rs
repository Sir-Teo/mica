use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::ir::{self, InstKind, Terminator, Type, ValueId};

use super::{Backend, BackendError, BackendOptions, BackendResult};

/// Backend that lowers the typed SSA module into portable C code and relies on
/// the host C compiler to produce machine code. This approach keeps the
/// backend dependency-free while still emitting native executables.
#[derive(Debug, Default, Clone)]
pub struct NativeBackend;

/// Native artifact produced by the backend. The generated C source is exposed
/// for inspection and can be compiled into an executable through helper
/// methods.
#[derive(Debug, Clone)]
pub struct NativeArtifact {
    pub c_source: String,
    pub module_name: String,
}

impl NativeArtifact {
    /// Writes the generated C source to the provided path.
    pub fn write_source<P: AsRef<Path>>(&self, path: P) -> BackendResult<PathBuf> {
        let path = path.as_ref();
        fs::write(path, &self.c_source)
            .map_err(|err| BackendError::Internal(format!("failed to write source: {err}")))?;
        Ok(path.to_path_buf())
    }

    /// Compiles the generated C source into an executable at `out_path` using
    /// the system C compiler.
    pub fn link_executable<P: AsRef<Path>>(&self, out_path: P) -> BackendResult<PathBuf> {
        let out_path = out_path.as_ref();
        let c_path = temp_path("mica", "c");
        self.write_source(&c_path)?;

        let status = Command::new("cc")
            .arg(&c_path)
            .arg("-std=c11")
            .arg("-O2")
            .arg("-o")
            .arg(out_path)
            .status()
            .map_err(|err| BackendError::Internal(format!("failed to invoke cc: {err}")))?;

        fs::remove_file(&c_path).ok();

        if !status.success() {
            return Err(BackendError::Internal(format!(
                "cc exited with status {status}",
            )));
        }

        Ok(out_path.to_path_buf())
    }
}

impl Backend for NativeBackend {
    type Output = NativeArtifact;

    fn compile(
        &self,
        module: &ir::Module,
        _options: &BackendOptions,
    ) -> BackendResult<Self::Output> {
        let c_source = generate_c_source(module)?;
        Ok(NativeArtifact {
            c_source,
            module_name: module.name.join("_"),
        })
    }
}

fn generate_c_source(module: &ir::Module) -> BackendResult<String> {
    let mut out = String::new();
    writeln!(out, "#include <stdbool.h>").unwrap();
    writeln!(out, "#include <stdint.h>").unwrap();
    writeln!(out, "#include <stddef.h>").unwrap();
    writeln!(out).unwrap();

    // Emit prototypes to allow mutual recursion.
    for function in &module.functions {
        writeln!(out, "{};", function_signature(module, function)?).unwrap();
    }
    writeln!(out).unwrap();

    for function in &module.functions {
        writeln!(out, "{} {{", function_signature(module, function)?).unwrap();
        emit_function_body(&mut out, module, function)?;
        writeln!(out, "}}").unwrap();
        writeln!(out).unwrap();
    }

    Ok(out)
}

fn function_signature(module: &ir::Module, function: &ir::Function) -> BackendResult<String> {
    let mut signature = String::new();
    write!(
        signature,
        "{} {}(",
        c_type_return(module.type_of(function.ret_type)),
        mangle_name(&function.name)
    )
    .unwrap();

    for (index, param) in function.params.iter().enumerate() {
        if index > 0 {
            write!(signature, ", ").unwrap();
        }
        write!(
            signature,
            "{} arg{}",
            c_type_value(module.type_of(param.ty)),
            index
        )
        .unwrap();
    }

    if function.params.is_empty() {
        signature.push_str("void");
    }

    signature.push(')');
    Ok(signature)
}

fn emit_function_body(
    out: &mut String,
    module: &ir::Module,
    function: &ir::Function,
) -> BackendResult<()> {
    writeln!(out, "  int prev_block = -1;").unwrap();
    writeln!(out, "  int current_block = 0;").unwrap();

    for (index, param) in function.params.iter().enumerate() {
        let var = value_name(param.value);
        writeln!(
            out,
            "  {} {} = arg{};",
            c_type_value(module.type_of(param.ty)),
            var,
            index
        )
        .unwrap();
    }

    if function.params.is_empty() {
        // C requires a statement even if there are no params.
        writeln!(out, "  (void)prev_block;").unwrap();
    }

    writeln!(out, "  goto block0;").unwrap();

    for block in &function.blocks {
        emit_block(out, module, function, block)?;
    }

    match default_return(module, function.ret_type) {
        Some(expr) => {
            writeln!(out, "  return {};", expr).unwrap();
        }
        None => {
            writeln!(out, "  return;").unwrap();
        }
    }
    Ok(())
}

fn emit_block(
    out: &mut String,
    module: &ir::Module,
    function: &ir::Function,
    block: &ir::BasicBlock,
) -> BackendResult<()> {
    writeln!(out, "block{}:", block.id.index()).unwrap();
    writeln!(out, "  current_block = {};", block.id.index()).unwrap();

    for inst in &block.instructions {
        if let InstKind::Phi { .. } = &inst.kind {
            emit_phi(out, module, inst)?;
        }
    }

    for inst in &block.instructions {
        if matches!(inst.kind, InstKind::Phi { .. }) {
            continue;
        }
        emit_instruction(out, module, inst)?;
    }

    emit_terminator(out, module, function, block)?;
    Ok(())
}

fn emit_phi(out: &mut String, module: &ir::Module, inst: &ir::Instruction) -> BackendResult<()> {
    let ty = module.type_of(inst.ty);
    let var = value_name(inst.id);
    writeln!(out, "  {} {};", c_type_value(ty), var).unwrap();

    if let InstKind::Phi { incomings } = &inst.kind {
        writeln!(out, "  switch (prev_block) {{").unwrap();
        for (block, value) in incomings {
            writeln!(
                out,
                "    case {}: {} = {}; break;",
                block.index(),
                var,
                value_name(*value)
            )
            .unwrap();
        }
        writeln!(out, "    default: {} = {}; break;", var, default_value(ty)).unwrap();
        writeln!(out, "  }}").unwrap();
    }

    Ok(())
}

fn emit_instruction(
    out: &mut String,
    module: &ir::Module,
    inst: &ir::Instruction,
) -> BackendResult<()> {
    let ty = module.type_of(inst.ty);
    let var = value_name(inst.id);
    match &inst.kind {
        InstKind::Literal(lit) => {
            writeln!(out, "  {} {} = {};", c_type_value(ty), var, literal(lit)).unwrap();
        }
        InstKind::Binary { op, lhs, rhs } => {
            writeln!(
                out,
                "  {} {} = {} {} {};",
                c_type_value(ty),
                var,
                value_name(*lhs),
                op,
                value_name(*rhs)
            )
            .unwrap();
        }
        InstKind::Call { func, args } => {
            let name = match func {
                ir::FuncRef::Function(path) => path.segments.join("_"),
                ir::FuncRef::Method(name) => name.clone(),
            };
            let args = args
                .iter()
                .map(|arg| value_name(*arg))
                .collect::<Vec<_>>()
                .join(", ");
            if matches!(ty, Type::Unit) {
                writeln!(out, "  {}({});", mangle_name(&name), args).unwrap();
                writeln!(out, "  {} {} = 0;", c_type_value(ty), var).unwrap();
            } else {
                writeln!(
                    out,
                    "  {} {} = {}({});",
                    c_type_value(ty),
                    var,
                    mangle_name(&name),
                    args
                )
                .unwrap();
            }
        }
        InstKind::Record { .. } => {
            return Err(BackendError::Unsupported(
                "record literals are not yet supported by the native backend".into(),
            ));
        }
        InstKind::Path(_) => {
            return Err(BackendError::Unsupported(
                "path expressions are not yet supported by the native backend".into(),
            ));
        }
        InstKind::Phi { .. } => {}
    }
    Ok(())
}

fn emit_terminator(
    out: &mut String,
    module: &ir::Module,
    function: &ir::Function,
    block: &ir::BasicBlock,
) -> BackendResult<()> {
    match &block.terminator {
        Terminator::Return(Some(value)) => {
            if matches!(module.type_of(function.ret_type), Type::Unit) {
                writeln!(out, "  return;").unwrap();
            } else {
                writeln!(out, "  return {};", value_name(*value)).unwrap();
            }
        }
        Terminator::Return(None) => match default_return(module, function.ret_type) {
            Some(expr) => {
                writeln!(out, "  return {};", expr).unwrap();
            }
            None => {
                writeln!(out, "  return;").unwrap();
            }
        },
        Terminator::Jump(target) => {
            writeln!(out, "  prev_block = current_block;").unwrap();
            writeln!(out, "  current_block = {};", target.index()).unwrap();
            writeln!(out, "  goto block{};", target.index()).unwrap();
        }
        Terminator::Branch {
            condition,
            then_block,
            else_block,
        } => {
            writeln!(out, "  prev_block = current_block;").unwrap();
            writeln!(out, "  if ({}) {{", value_name(*condition)).unwrap();
            writeln!(out, "    current_block = {};", then_block.index()).unwrap();
            writeln!(out, "    goto block{};", then_block.index()).unwrap();
            writeln!(out, "  }} else {{").unwrap();
            writeln!(out, "    current_block = {};", else_block.index()).unwrap();
            writeln!(out, "    goto block{};", else_block.index()).unwrap();
            writeln!(out, "  }}").unwrap();
        }
    }
    Ok(())
}

fn c_type_value(ty: &Type) -> &'static str {
    match ty {
        Type::Int | Type::Named(_) | Type::Unknown | Type::Unit | Type::Record(_) => "int64_t",
        Type::String => "const char *",
        Type::Float => "double",
        Type::Bool => "bool",
    }
}

fn c_type_return(ty: &Type) -> &'static str {
    match ty {
        Type::Unit => "void",
        other => c_type_value(other),
    }
}

fn default_value(ty: &Type) -> &'static str {
    match ty {
        Type::Bool => "false",
        Type::Float => "0.0",
        Type::String => "NULL",
        _ => "0",
    }
}

fn default_return(module: &ir::Module, ty: ir::TypeId) -> Option<&'static str> {
    let ty = module.type_of(ty);
    match ty {
        Type::Unit => None,
        Type::Float => Some("0.0"),
        Type::Bool => Some("false"),
        Type::String => Some("NULL"),
        _ => Some("0"),
    }
}

fn literal(lit: &crate::syntax::ast::Literal) -> String {
    match lit {
        crate::syntax::ast::Literal::Int(value) => value.to_string(),
        crate::syntax::ast::Literal::Float(value) => value.to_string(),
        crate::syntax::ast::Literal::Bool(value) => value.to_string(),
        crate::syntax::ast::Literal::String(value) => format!("\"{}\"", escape_string(value)),
        crate::syntax::ast::Literal::Unit => "0".to_string(),
    }
}

fn escape_string(value: &str) -> String {
    value
        .chars()
        .flat_map(|ch| match ch {
            '\\' => "\\\\".chars().collect::<Vec<_>>(),
            '\"' => "\\\"".chars().collect(),
            '\n' => "\\n".chars().collect(),
            '\r' => "\\r".chars().collect(),
            '\t' => "\\t".chars().collect(),
            other => vec![other],
        })
        .collect()
}

fn value_name(id: ValueId) -> String {
    format!("v{}", id.index())
}

fn mangle_name(name: &str) -> String {
    name.replace("::", "_")
}

fn temp_path(prefix: &str, ext: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    path.push(format!("{}_{}.{ext}", prefix, nanos));
    path
}
