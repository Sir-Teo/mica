use std::collections::{BTreeSet, HashMap};
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::ir::{self, InstKind, Terminator, Type, ValueId};

use super::{Backend, BackendError, BackendOptions, BackendResult};

type RecordNameMap = HashMap<ir::TypeId, String>;

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

        let output = Command::new("cc")
            .arg(&c_path)
            .arg("-std=c11")
            .arg("-O2")
            .arg("-o")
            .arg(out_path)
            .output()
            .map_err(|err| BackendError::Internal(format!("failed to invoke cc: {err}")))?;

        fs::remove_file(&c_path).ok();

        if !output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(BackendError::Internal(format!(
                "cc exited with status {}. stdout: {} stderr: {}",
                output.status,
                stdout.trim(),
                stderr.trim()
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
    writeln!(out, "#include <stdlib.h>").unwrap();
    writeln!(out, "#include <stdio.h>").unwrap();
    writeln!(out, "#include <string.h>").unwrap();
    writeln!(out, "#include <time.h>").unwrap();
    writeln!(out).unwrap();

    let record_names = collect_record_names(module);
    let capabilities = collect_capabilities(module);
    emit_runtime_support(&mut out, &capabilities)?;
    emit_record_definitions(&mut out, module, &record_names)?;

    // Emit prototypes to allow mutual recursion.
    for function in &module.functions {
        writeln!(
            out,
            "{};",
            function_signature(module, function, &record_names)?
        )
        .unwrap();
    }
    writeln!(out).unwrap();

    for function in &module.functions {
        writeln!(
            out,
            "{} {{",
            function_signature(module, function, &record_names)?
        )
        .unwrap();
        emit_function_body(&mut out, module, function, &record_names)?;
        writeln!(out, "}}").unwrap();
        writeln!(out).unwrap();
    }

    Ok(out)
}

fn emit_record_definitions(
    out: &mut String,
    module: &ir::Module,
    record_names: &RecordNameMap,
) -> BackendResult<()> {
    if record_names.is_empty() {
        return Ok(());
    }

    let mut entries: Vec<_> = record_names.iter().collect();
    entries.sort_by_key(|(id, _)| id.index());

    for (type_id, name) in entries {
        let Type::Record(record) = module.type_of(*type_id) else {
            continue;
        };
        writeln!(out, "typedef struct {name} {{").unwrap();
        for field in &record.fields {
            let field_ty = c_type_value(module, field.ty, record_names);
            writeln!(out, "  {} {};", field_ty, sanitize_identifier(&field.name)).unwrap();
        }
        writeln!(out, "}} {name};").unwrap();
        writeln!(out).unwrap();
    }

    Ok(())
}

fn function_signature(
    module: &ir::Module,
    function: &ir::Function,
    record_names: &RecordNameMap,
) -> BackendResult<String> {
    let mut signature = String::new();
    let ret_type = c_type_return(module, function.ret_type, record_names);
    write!(signature, "{} {}(", ret_type, mangle_name(&function.name)).unwrap();

    for (index, param) in function.params.iter().enumerate() {
        if index > 0 {
            write!(signature, ", ").unwrap();
        }
        write!(
            signature,
            "{} arg{}",
            c_type_value(module, param.ty, record_names),
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
    record_names: &RecordNameMap,
) -> BackendResult<()> {
    writeln!(out, "  int prev_block = -1;").unwrap();
    writeln!(out, "  int current_block = 0;").unwrap();

    for (index, param) in function.params.iter().enumerate() {
        let var = value_name(param.value);
        writeln!(
            out,
            "  {} {} = arg{};",
            c_type_value(module, param.ty, record_names),
            var,
            index
        )
        .unwrap();
    }

    emit_runtime_capability_guards(out, module, function)?;

    if function.params.is_empty() {
        // C requires a statement even if there are no params.
        writeln!(out, "  (void)prev_block;").unwrap();
    }

    writeln!(out, "  goto block0;").unwrap();

    for block in &function.blocks {
        emit_block(out, module, function, block, record_names)?;
    }

    match default_return(module, function.ret_type, record_names) {
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
    record_names: &RecordNameMap,
) -> BackendResult<()> {
    writeln!(out, "block{}:", block.id.index()).unwrap();
    writeln!(out, "  current_block = {};", block.id.index()).unwrap();

    for inst in &block.instructions {
        if let InstKind::Phi { .. } = &inst.kind {
            emit_phi(out, module, inst, record_names)?;
        }
    }

    for inst in &block.instructions {
        if matches!(inst.kind, InstKind::Phi { .. }) {
            continue;
        }
        emit_instruction(out, module, inst, record_names)?;
    }

    emit_terminator(out, module, function, block, record_names)?;
    Ok(())
}

fn emit_phi(
    out: &mut String,
    module: &ir::Module,
    inst: &ir::Instruction,
    record_names: &RecordNameMap,
) -> BackendResult<()> {
    let ty = inst.ty;
    let var = value_name(inst.id);
    writeln!(out, "  {} {};", c_type_value(module, ty, record_names), var).unwrap();

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
        writeln!(
            out,
            "    default: {} = {}; break;",
            var,
            default_value(module, ty, record_names)
        )
        .unwrap();
        writeln!(out, "  }}").unwrap();
    }

    Ok(())
}

fn emit_instruction(
    out: &mut String,
    module: &ir::Module,
    inst: &ir::Instruction,
    record_names: &RecordNameMap,
) -> BackendResult<()> {
    let ty = inst.ty;
    let var = value_name(inst.id);
    match &inst.kind {
        InstKind::Literal(lit) => {
            writeln!(
                out,
                "  {} {} = {};",
                c_type_value(module, ty, record_names),
                var,
                literal(lit)
            )
            .unwrap();
        }
        InstKind::Binary { op, lhs, rhs } => {
            writeln!(
                out,
                "  {} {} = {} {} {};",
                c_type_value(module, ty, record_names),
                var,
                value_name(*lhs),
                op,
                value_name(*rhs)
            )
            .unwrap();
        }
        InstKind::Call { func, args } => {
            if let ir::FuncRef::Method(name) = func
                && emit_runtime_method(out, module, name, args, ty, &var, record_names)?
            {
                return Ok(());
            }
            let name = match func {
                ir::FuncRef::Function(path) => path.segments.join("_"),
                ir::FuncRef::Method(name) => name.clone(),
            };
            let args = args
                .iter()
                .map(|arg| value_name(*arg))
                .collect::<Vec<_>>()
                .join(", ");
            if matches!(module.type_of(ty), Type::Unit) {
                writeln!(out, "  {}({});", mangle_name(&name), args).unwrap();
                writeln!(
                    out,
                    "  {} {} = 0;",
                    c_type_value(module, ty, record_names),
                    var
                )
                .unwrap();
            } else {
                writeln!(
                    out,
                    "  {} {} = {}({});",
                    c_type_value(module, ty, record_names),
                    var,
                    mangle_name(&name),
                    args
                )
                .unwrap();
            }
        }
        InstKind::Record { fields, .. } => {
            let Type::Record(record) = module.type_of(ty) else {
                return Err(BackendError::Internal(
                    "record literal lowered with non-record type".into(),
                ));
            };
            let mut initializers = Vec::with_capacity(record.fields.len());
            for field in &record.fields {
                let Some((_, value)) = fields.iter().find(|(name, _)| name == &field.name) else {
                    let record_name = record.name.as_deref().unwrap_or("<anonymous record>");
                    return Err(BackendError::Unsupported(format!(
                        "record literal for '{record_name}' is missing field '{}'",
                        field.name
                    )));
                };
                initializers.push(format!(
                    ".{} = {}",
                    sanitize_identifier(&field.name),
                    value_name(*value)
                ));
            }
            for (name, _) in fields {
                if record.field(name).is_none() {
                    return Err(BackendError::Unsupported(format!(
                        "record literal references unknown field '{}'",
                        name
                    )));
                }
            }
            let type_name = c_type_value(module, ty, record_names);
            if initializers.is_empty() {
                writeln!(out, "  {} {} = ({}){{0}};", type_name, var, type_name).unwrap();
            } else {
                writeln!(
                    out,
                    "  {} {} = ({}){{ {} }};",
                    type_name,
                    var,
                    type_name,
                    initializers.join(", ")
                )
                .unwrap();
            }
        }
        InstKind::Path(path) => {
            return Err(BackendError::Unsupported(format!(
                "path expression '{}' cannot be lowered by the native backend yet",
                path.segments.join("::")
            )));
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
    record_names: &RecordNameMap,
) -> BackendResult<()> {
    match &block.terminator {
        Terminator::Return(Some(value)) => {
            if matches!(module.type_of(function.ret_type), Type::Unit) {
                writeln!(out, "  return;").unwrap();
            } else {
                writeln!(out, "  return {};", value_name(*value)).unwrap();
            }
        }
        Terminator::Return(None) => match default_return(module, function.ret_type, record_names) {
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

fn c_type_value(module: &ir::Module, ty: ir::TypeId, record_names: &RecordNameMap) -> String {
    match module.type_of(ty) {
        Type::Int | Type::Named(_) | Type::Unknown | Type::Unit => "int64_t".into(),
        Type::String => "const char *".into(),
        Type::Float => "double".into(),
        Type::Bool => "bool".into(),
        Type::Record(_) => record_names
            .get(&ty)
            .cloned()
            .unwrap_or_else(|| format!("record_{}", ty.index())),
    }
}

fn c_type_return(module: &ir::Module, ty: ir::TypeId, record_names: &RecordNameMap) -> String {
    match module.type_of(ty) {
        Type::Unit => "void".into(),
        _ => c_type_value(module, ty, record_names),
    }
}

fn default_value(module: &ir::Module, ty: ir::TypeId, record_names: &RecordNameMap) -> String {
    match module.type_of(ty) {
        Type::Bool => "false".into(),
        Type::Float => "0.0".into(),
        Type::String => "NULL".into(),
        Type::Record(_) => format!("({}){{0}}", c_type_value(module, ty, record_names)),
        _ => "0".into(),
    }
}

fn default_return(
    module: &ir::Module,
    ty: ir::TypeId,
    record_names: &RecordNameMap,
) -> Option<String> {
    match module.type_of(ty) {
        Type::Unit => None,
        Type::Float => Some("0.0".into()),
        Type::Bool => Some("false".into()),
        Type::String => Some("NULL".into()),
        Type::Record(_) => Some(format!("({}){{0}}", c_type_value(module, ty, record_names))),
        _ => Some("0".into()),
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
    sanitize_identifier(&name.replace("::", "_"))
}

fn sanitize_identifier(name: &str) -> String {
    let mut result = String::new();
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            result.push(ch);
        } else {
            result.push('_');
        }
    }
    if result.is_empty() {
        result.push('_');
    }
    if result.chars().next().unwrap().is_ascii_digit() {
        result.insert(0, '_');
    }
    result
}

fn emit_runtime_capability_guards(
    out: &mut String,
    module: &ir::Module,
    function: &ir::Function,
) -> BackendResult<()> {
    if function.effect_row.is_empty() {
        return Ok(());
    }

    writeln!(out, "  mica_runtime_initialize();").unwrap();
    let mut seen = BTreeSet::new();
    for effect in &function.effect_row {
        let name = module.effects.name(*effect);
        if seen.insert(name) {
            writeln!(
                out,
                "  mica_runtime_require_capability(\"{}\");",
                escape_string(name)
            )
            .unwrap();
        }
    }
    Ok(())
}

fn emit_runtime_method(
    out: &mut String,
    module: &ir::Module,
    name: &str,
    args: &[ir::ValueId],
    ty: ir::TypeId,
    var: &str,
    record_names: &RecordNameMap,
) -> BackendResult<bool> {
    match name {
        "println" | "write_line" => {
            if args.len() < 2 {
                return Err(BackendError::Internal(format!(
                    "method '{name}' expected value argument"
                )));
            }
            writeln!(out, "  mica_runtime_initialize();").unwrap();
            writeln!(
                out,
                "  mica_runtime_require_capability(\"{}\");",
                escape_string("io")
            )
            .unwrap();
            writeln!(
                out,
                "  mica_runtime_io_write_line({});",
                value_name(args[1])
            )
            .unwrap();
            writeln!(
                out,
                "  {} {} = 0;",
                c_type_value(module, ty, record_names),
                var
            )
            .unwrap();
            Ok(true)
        }
        "now_millis" => {
            writeln!(out, "  mica_runtime_initialize();").unwrap();
            writeln!(
                out,
                "  mica_runtime_require_capability(\"{}\");",
                escape_string("time")
            )
            .unwrap();
            writeln!(
                out,
                "  {} {} = mica_runtime_time_now_millis();",
                c_type_value(module, ty, record_names),
                var
            )
            .unwrap();
            Ok(true)
        }
        _ => Ok(false),
    }
}

fn emit_runtime_support(out: &mut String, capabilities: &[String]) -> BackendResult<()> {
    if capabilities.is_empty() {
        out.push_str("static const size_t MICA_RUNTIME_CAPABILITY_COUNT = 0;\n");
        out.push_str("static const char *MICA_RUNTIME_CAPABILITY_NAMES[1] = { NULL };\n");
        out.push_str("static int MICA_RUNTIME_CAPABILITY_AVAILABLE[1] = { 0 };\n");
    } else {
        let count = capabilities.len();
        let names = capabilities
            .iter()
            .map(|cap| format!("\"{}\"", escape_string(cap)))
            .collect::<Vec<_>>()
            .join(", ");
        let zeros = vec!["0"; count].join(", ");
        out.push_str(&format!(
            "static const size_t MICA_RUNTIME_CAPABILITY_COUNT = {count};\n"
        ));
        out.push_str(&format!(
            "static const char *MICA_RUNTIME_CAPABILITY_NAMES[{count}] = {{ {names} }};\n"
        ));
        out.push_str(&format!(
            "static int MICA_RUNTIME_CAPABILITY_AVAILABLE[{count}] = {{ {zeros} }};\n"
        ));
    }
    out.push_str("static int MICA_RUNTIME_INITIALIZED = 0;\n\n");

    out.push_str("static void mica_runtime_mark_capability(const char *name) {\n");
    out.push_str("  for (size_t i = 0; i < MICA_RUNTIME_CAPABILITY_COUNT; ++i) {\n");
    out.push_str("    const char *candidate = MICA_RUNTIME_CAPABILITY_NAMES[i];\n");
    out.push_str("    if (candidate && strcmp(candidate, name) == 0) {\n");
    out.push_str("      MICA_RUNTIME_CAPABILITY_AVAILABLE[i] = 1;\n");
    out.push_str("      return;\n");
    out.push_str("    }\n");
    out.push_str("  }\n");
    out.push_str("}\n\n");

    out.push_str("static void mica_runtime_register_default_providers(void) {\n");
    if capabilities.is_empty() {
        out.push_str("  (void)MICA_RUNTIME_CAPABILITY_NAMES;\n");
    } else {
        if capabilities.iter().any(|cap| cap == "io") {
            out.push_str(&format!(
                "  mica_runtime_mark_capability(\"{}\");\n",
                escape_string("io")
            ));
        }
        if capabilities.iter().any(|cap| cap == "time") {
            out.push_str(&format!(
                "  mica_runtime_mark_capability(\"{}\");\n",
                escape_string("time")
            ));
        }
    }
    out.push_str("}\n\n");

    out.push_str("static void mica_runtime_initialize(void) {\n");
    out.push_str("  if (!MICA_RUNTIME_INITIALIZED) {\n");
    out.push_str("    MICA_RUNTIME_INITIALIZED = 1;\n");
    out.push_str("    mica_runtime_register_default_providers();\n");
    out.push_str("  }\n");
    out.push_str("}\n\n");

    out.push_str("static void mica_runtime_missing_capability(const char *name) {\n");
    out.push_str(
        "  fprintf(stderr, \"error: capability provider '%s' is not registered\\n\", name);\n",
    );
    out.push_str("  exit(74);\n");
    out.push_str("}\n\n");

    out.push_str("static void mica_runtime_require_capability(const char *name) {\n");
    out.push_str("  for (size_t i = 0; i < MICA_RUNTIME_CAPABILITY_COUNT; ++i) {\n");
    out.push_str("    const char *candidate = MICA_RUNTIME_CAPABILITY_NAMES[i];\n");
    out.push_str("    if (candidate && strcmp(candidate, name) == 0) {\n");
    out.push_str("      if (!MICA_RUNTIME_CAPABILITY_AVAILABLE[i]) {\n");
    out.push_str("        mica_runtime_missing_capability(name);\n");
    out.push_str("      }\n");
    out.push_str("      return;\n");
    out.push_str("    }\n");
    out.push_str("  }\n");
    out.push_str(
        "  fprintf(stderr, \"error: capability '%s' is not declared in this binary\\n\", name);\n",
    );
    out.push_str("  exit(74);\n");
    out.push_str("}\n\n");

    out.push_str(
        "static void mica_runtime_provider_failure(const char *capability, const char *message) {\n",
    );
    out.push_str(
        "  fprintf(stderr, \"error: capability provider '%s' reported an error: %s\\n\", capability, message);\n",
    );
    out.push_str("  exit(74);\n");
    out.push_str("}\n\n");

    out.push_str("static void mica_runtime_io_write_line(const char *message) {\n");
    out.push_str(&format!(
        "  mica_runtime_require_capability(\"{}\");\n",
        escape_string("io")
    ));
    out.push_str("  if (message) {\n");
    out.push_str("    fprintf(stdout, \"%s\\n\", message);\n");
    out.push_str("    fflush(stdout);\n");
    out.push_str("  }\n");
    out.push_str("}\n\n");

    out.push_str("static int64_t mica_runtime_time_now_millis(void) {\n");
    out.push_str(&format!(
        "  mica_runtime_require_capability(\"{}\");\n",
        escape_string("time")
    ));
    out.push_str("  struct timespec ts;\n");
    out.push_str("  timespec_get(&ts, TIME_UTC);\n");
    out.push_str("  return (int64_t)ts.tv_sec * 1000 + (int64_t)(ts.tv_nsec / 1000000);\n");
    out.push_str("}\n\n");

    Ok(())
}

fn collect_capabilities(module: &ir::Module) -> Vec<String> {
    let mut caps = BTreeSet::new();
    for function in &module.functions {
        for effect in &function.effect_row {
            caps.insert(module.effects.name(*effect).to_string());
        }
    }
    caps.into_iter().collect()
}

fn collect_record_names(module: &ir::Module) -> RecordNameMap {
    let mut names = HashMap::new();
    for (id, ty) in module.types.entries() {
        if let Type::Record(record) = ty {
            let base = record
                .name
                .as_ref()
                .map(|name| sanitize_identifier(name))
                .unwrap_or_else(|| format!("anon_{}", id.index()));
            names.insert(id, format!("record_{}", base));
        }
    }
    names
}

fn temp_path(prefix: &str, extension: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    path.push(format!("{prefix}-{nanos}.{extension}"));
    path
}
