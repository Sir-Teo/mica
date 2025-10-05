use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use mica::{
    Result, backend, check, error, ir, lexer, lower, parser, pretty,
    resolve::{self, CapabilityScope, PathKind, SymbolCategory, SymbolScope},
    runtime,
    syntax::ast,
    tooling,
};

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {}", err);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli_args = CliArgs::parse(env::args().skip(1))?;
    let source = fs::read_to_string(&cli_args.input_path)
        .map_err(|e| error::Error::lex(None, e.to_string()))?;
    let ctx = CommandContext::new(cli_args.input_path.clone(), source, cli_args.pretty);

    cli_args.command.execute(ctx)
}

struct CliArgs {
    input_path: PathBuf,
    pretty: bool,
    command: CommandKind,
}

impl CliArgs {
    fn parse<I>(mut args: I) -> Result<Self>
    where
        I: Iterator<Item = String>,
    {
        let mut command = CommandKind::Ast;
        let mut pretty = false;
        let mut output_path: Option<PathBuf> = None;
        let mut trace: Option<TraceTarget> = None;
        let mut input_path: Option<PathBuf> = None;

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--tokens" => command = CommandKind::Tokens,
                "--ast" => command = CommandKind::Ast,
                "--check" => command = CommandKind::Check,
                "--pretty" => pretty = true,
                "--resolve" => command = CommandKind::Resolve,
                "--resolve-json" => command = CommandKind::ResolveJson,
                "--lower" => command = CommandKind::Lower,
                "--ir" => command = CommandKind::Ir,
                "--ir-json" => command = CommandKind::IrJson,
                "--pipeline-json" => command = CommandKind::PipelineJson,
                "--llvm" | "--emit-llvm" => command = CommandKind::Llvm,
                "--build" => command = CommandKind::Build { output: None },
                "--run" => {
                    command = CommandKind::Run {
                        output: None,
                        trace: None,
                    }
                }
                "--trace-json" => {
                    let target = args.next().ok_or_else(|| {
                        error::Error::parse(None, "expected output path after --trace-json")
                    })?;
                    trace = Some(match target.as_str() {
                        "-" => TraceTarget::Stdout,
                        other => TraceTarget::File(PathBuf::from(other)),
                    });
                }
                "--out" => {
                    let value = args.next().ok_or_else(|| {
                        error::Error::parse(None, "expected output path after --out")
                    })?;
                    output_path = Some(PathBuf::from(value));
                }
                _ => {
                    input_path = Some(PathBuf::from(arg));
                    for extra in args {
                        if input_path.is_some() {
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

        let input_path =
            input_path.ok_or_else(|| error::Error::parse(None, "missing input file"))?;

        if trace.is_some() && !matches!(command, CommandKind::Run { .. }) {
            return Err(error::Error::parse(
                None,
                "--trace-json is only supported with --run",
            ));
        }

        let command = command
            .with_output_path(output_path)
            .with_trace(trace.clone());

        Ok(Self {
            input_path,
            pretty,
            command,
        })
    }
}

#[derive(Debug, Clone)]
struct CommandContext {
    input_path: PathBuf,
    source: String,
    pretty: bool,
}

impl CommandContext {
    fn new(input_path: PathBuf, source: String, pretty: bool) -> Self {
        Self {
            input_path,
            source,
            pretty,
        }
    }
}

#[derive(Debug, Clone)]
enum CommandKind {
    Tokens,
    Ast,
    Check,
    Resolve,
    ResolveJson,
    Lower,
    Ir,
    IrJson,
    PipelineJson,
    Llvm,
    Build {
        output: Option<PathBuf>,
    },
    Run {
        output: Option<PathBuf>,
        trace: Option<TraceTarget>,
    },
}

impl CommandKind {
    fn with_output_path(self, output: Option<PathBuf>) -> Self {
        match self {
            CommandKind::Build { .. } => CommandKind::Build { output },
            CommandKind::Run { trace, .. } => CommandKind::Run { output, trace },
            other => other,
        }
    }

    fn with_trace(self, trace: Option<TraceTarget>) -> Self {
        match self {
            CommandKind::Run { output, .. } => CommandKind::Run { output, trace },
            other => other,
        }
    }

    fn execute(self, ctx: CommandContext) -> Result<()> {
        match self {
            CommandKind::Tokens => run_tokens(&ctx),
            CommandKind::Ast => run_ast(&ctx),
            CommandKind::Check => run_check(&ctx),
            CommandKind::Resolve => run_resolve(&ctx),
            CommandKind::ResolveJson => run_resolve_json(&ctx),
            CommandKind::Lower => run_lower(&ctx),
            CommandKind::Ir => run_ir(&ctx),
            CommandKind::IrJson => run_ir_json(&ctx),
            CommandKind::PipelineJson => run_pipeline_json(&ctx),
            CommandKind::Llvm => run_llvm(&ctx),
            CommandKind::Build { output } => run_build(&ctx, output),
            CommandKind::Run { output, trace } => run_executable(&ctx, output, trace),
        }
    }
}

#[derive(Debug, Clone)]
enum TraceTarget {
    Stdout,
    File(PathBuf),
}

fn run_tokens(ctx: &CommandContext) -> Result<()> {
    let tokens = lexer::lex(&ctx.source)?;
    for token in tokens {
        println!("{:?}", token);
    }
    Ok(())
}

fn run_ast(ctx: &CommandContext) -> Result<()> {
    let module = parser::parse_module(&ctx.source)?;
    if ctx.pretty {
        println!("{}", pretty::module_to_string(&module));
    } else {
        println!("{:#?}", module);
    }
    Ok(())
}

fn run_check(ctx: &CommandContext) -> Result<()> {
    let module = parser::parse_module(&ctx.source)?;
    let result = check::check_module(&module);
    if result.diagnostics.is_empty() {
        println!("ok");
    } else {
        for d in result.diagnostics {
            println!("warning: {}", d.message);
        }
    }
    Ok(())
}

fn run_resolve(ctx: &CommandContext) -> Result<()> {
    let module = parser::parse_module(&ctx.source)?;
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

    Ok(())
}

fn run_resolve_json(ctx: &CommandContext) -> Result<()> {
    let module = parser::parse_module(&ctx.source)?;
    let resolved = resolve::resolve_module(&module);
    let json = resolved_to_json(&resolved);
    println!("{}", json);
    Ok(())
}

fn run_lower(ctx: &CommandContext) -> Result<()> {
    let module = parser::parse_module(&ctx.source)?;
    let h = lower::lower_module(&module);
    println!("{}", lower::hir_to_string(&h));
    Ok(())
}

fn run_ir(ctx: &CommandContext) -> Result<()> {
    let module = parser::parse_module(&ctx.source)?;
    let hir = lower::lower_module(&module);
    let typed = ir::lower_module(&hir);
    let backend = backend::text::TextBackend;
    let output = backend::run(&backend, &typed, &backend::BackendOptions::default())
        .map_err(|err| error::Error::parse(None, err.to_string()))?;
    println!("{}", output);
    Ok(())
}

fn run_ir_json(ctx: &CommandContext) -> Result<()> {
    let module = parser::parse_module(&ctx.source)?;
    let hir = lower::lower_module(&module);
    let typed = ir::lower_module(&hir);
    let json = ir_module_to_json(&typed);
    println!("{}", json);
    Ok(())
}

fn run_pipeline_json(ctx: &CommandContext) -> Result<()> {
    let snapshot = tooling::PipelineSnapshot::capture(&ctx.source);
    println!("{}", snapshot.to_json_string());
    Ok(())
}

fn run_llvm(ctx: &CommandContext) -> Result<()> {
    let module = parser::parse_module(&ctx.source)?;
    let hir = lower::lower_module(&module);
    let typed = ir::lower_module(&hir);
    let backend = backend::llvm::LlvmBackend::default();
    let output = backend::run(&backend, &typed, &backend::BackendOptions::default())
        .map_err(|err| error::Error::parse(None, err.to_string()))?;
    println!("{}", output.as_str());
    Ok(())
}

fn run_build(ctx: &CommandContext, output: Option<PathBuf>) -> Result<()> {
    let module = parser::parse_module(&ctx.source)?;
    let hir = lower::lower_module(&module);
    let typed = ir::lower_module(&hir);
    let backend = backend::native::NativeBackend;
    let artifact = backend::run(&backend, &typed, &backend::BackendOptions::default())
        .map_err(|err| error::Error::parse(None, err.to_string()))?;
    let mut default_path = ctx.input_path.clone();
    default_path.set_extension("bin");
    let target_path = output.unwrap_or(default_path);
    artifact
        .link_executable(&target_path)
        .map_err(|err| error::Error::parse(None, err.to_string()))?;
    println!("built {}", target_path.display());
    Ok(())
}

fn run_executable(
    ctx: &CommandContext,
    output: Option<PathBuf>,
    trace: Option<TraceTarget>,
) -> Result<()> {
    let module = parser::parse_module(&ctx.source)?;
    let resolved = resolve::resolve_module(&module);
    let entry_spec = entry_task_spec(&module, &resolved);
    if let Some(spec) = &entry_spec {
        let runtime = runtime::Runtime::with_default_shims()
            .map_err(|err| error::Error::parse(None, err.to_string()))?;
        runtime
            .ensure_capabilities(spec)
            .map_err(|err| error::Error::parse(None, err.to_string()))?;
    }
    let hir = lower::lower_module(&module);
    let typed = ir::lower_module(&hir);
    let backend = backend::native::NativeBackend;
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

    let output = Command::new(&exe_path)
        .output()
        .map_err(|err| error::Error::parse(None, err.to_string()))?;
    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let mut message = format!("program exited with status {}", output.status);
        if !stdout.trim().is_empty() {
            message.push_str(&format!("; stdout: {}", stdout.trim()));
        }
        if !stderr.trim().is_empty() {
            message.push_str(&format!("; stderr: {}", stderr.trim()));
        }
        return Err(error::Error::parse(None, message));
    }

    if !output.stdout.is_empty() {
        print!("{}", String::from_utf8_lossy(&output.stdout));
    }
    if !output.stderr.is_empty() {
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
    }

    if let Some(target) = trace {
        let task_name = entry_spec
            .as_ref()
            .map(|spec| spec.name().to_string())
            .unwrap_or_else(|| "main".to_string());
        let stdout_text = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr_text = String::from_utf8_lossy(&output.stderr).to_string();
        let trace_json = build_trace_json(&task_name, &stdout_text, &stderr_text);
        match target {
            TraceTarget::Stdout => {
                println!("{}", trace_json);
            }
            TraceTarget::File(path) => {
                fs::write(&path, trace_json)
                    .map_err(|err| error::Error::parse(None, err.to_string()))?;
                println!("trace written to {}", path.display());
            }
        }
    }

    if let Some(path) = cleanup_path {
        fs::remove_file(&path).ok();
    }
    Ok(())
}

fn build_trace_json(task: &str, stdout: &str, stderr: &str) -> String {
    let mut events = Vec::new();
    events.push(format!(
        "{{\"type\":\"task_started\",\"task\":{}}}",
        json_string(task)
    ));

    let mut capability_invocations = 0usize;
    for line in stdout.lines().filter(|line| !line.is_empty()) {
        events.push(format!(
            "{{\"type\":\"capability_invoked\",\"task\":{task},\"capability\":\"io\",\"operation\":\"write_line\"}}",
            task = json_string(task)
        ));
        events.push(format!(
            "{{\"type\":\"capability_event\",\"task\":{task},\"capability\":\"io\",\"event\":{{\"type\":\"message\",\"value\":{value}}}}}",
            task = json_string(task),
            value = json_string(line)
        ));
        capability_invocations += 1;
    }

    for line in stderr.lines().filter(|line| !line.is_empty()) {
        events.push(format!(
            "{{\"type\":\"capability_event\",\"task\":{task},\"capability\":\"io\",\"event\":{{\"type\":\"message\",\"value\":{value}}}}}",
            task = json_string(task),
            value = json_string(&format!("stderr: {line}"))
        ));
    }

    events.push(format!(
        "{{\"type\":\"task_completed\",\"task\":{}}}",
        json_string(task)
    ));

    let total_events = events.len();
    let mut telemetry = Vec::new();
    for (index, event) in events.iter().enumerate() {
        telemetry.push(format!(
            "{{\"sequence\":{index},\"timestamp_micros\":null,\"event\":{event}}}",
            index = index,
            event = event
        ));
    }

    let capability_counts = if capability_invocations > 0 {
        format!("{{\"io\":{}}}", capability_invocations)
    } else {
        "{}".to_string()
    };
    let operation_counts = if capability_invocations > 0 {
        format!("{{\"io::write_line\":{}}}", capability_invocations)
    } else {
        "{}".to_string()
    };

    let tasks = format!(
        "{{\"task\":{task},\"start_timestamp_micros\":null,\"duration_micros\":0,\"event_count\":{count},\"capability_counts\":{cap_counts},\"operation_counts\":{op_counts},\"capability_durations_micros\":{{}},\"operation_durations_micros\":{{}},\"spawned_tasks\":0}}",
        task = json_string(task),
        count = total_events,
        cap_counts = capability_counts,
        op_counts = operation_counts
    );

    let summary = format!(
        "{{\"total_tasks\":1,\"total_events\":{events},\"spawned_tasks\":0,\"capability_counts\":{cap_counts},\"operation_counts\":{op_counts},\"capability_durations_micros\":{{}},\"operation_durations_micros\":{{}}}}",
        events = total_events,
        cap_counts = capability_counts,
        op_counts = operation_counts
    );

    let events_json = events.join(",");
    let telemetry_json = telemetry.join(",");

    format!(
        "{{\"events\":[{events}],\"telemetry\":[{telemetry}],\"tasks\":[{tasks}],\"summary\":{summary}}}",
        events = events_json,
        telemetry = telemetry_json,
        tasks = tasks,
        summary = summary
    )
}

fn entry_task_spec(
    module: &ast::Module,
    resolved: &resolve::Resolved,
) -> Option<runtime::TaskSpec> {
    let has_main = module.items.iter().any(|item| match item {
        ast::Item::Function(func) => func.name == "main",
        _ => false,
    });
    if !has_main {
        return None;
    }

    let module_name = resolved.module_path.join("::");
    let task_name = if module_name.is_empty() {
        String::from("main")
    } else {
        format!("{}::main", module_name)
    };

    let mut spec = runtime::TaskSpec::new(task_name);
    for binding in &resolved.capabilities {
        if let CapabilityScope::Function {
            module_path,
            function,
        } = &binding.scope
            && module_path == &resolved.module_path
            && function == "main"
        {
            spec.require(binding.name.clone());
        }
    }
    Some(spec)
}

fn resolved_to_json(resolved: &resolve::Resolved) -> String {
    let mut fields = Vec::new();
    fields.push(("module_path", json_string_array(&resolved.module_path)));

    let mut adt_pairs = resolved
        .adts
        .iter()
        .map(|(name, variants)| (name.clone(), json_string_array(variants)))
        .collect::<Vec<_>>();
    adt_pairs.sort_by(|a, b| a.0.cmp(&b.0));
    fields.push(("adts", json_object_pairs(adt_pairs)));

    let mut variant_pairs = resolved
        .variant_to_adt
        .iter()
        .map(|(name, adts)| (name.clone(), json_string_array(adts)))
        .collect::<Vec<_>>();
    variant_pairs.sort_by(|a, b| a.0.cmp(&b.0));
    fields.push(("variant_to_adt", json_object_pairs(variant_pairs)));

    let imports = resolved
        .imports
        .iter()
        .map(|import| {
            json_object(vec![
                ("path", json_string_array(&import.path)),
                (
                    "alias",
                    import
                        .alias
                        .as_ref()
                        .map(|alias| json_string(alias))
                        .unwrap_or_else(|| "null".to_string()),
                ),
            ])
        })
        .collect::<Vec<_>>();
    fields.push(("imports", json_array(imports)));

    let symbols = resolved
        .symbols
        .iter()
        .map(resolved_symbol_json)
        .collect::<Vec<_>>();
    fields.push(("symbols", json_array(symbols)));

    let paths = resolved
        .resolved_paths
        .iter()
        .map(resolved_path_json)
        .collect::<Vec<_>>();
    fields.push(("resolved_paths", json_array(paths)));

    let capabilities = resolved
        .capabilities
        .iter()
        .map(capability_binding_json)
        .collect::<Vec<_>>();
    fields.push(("capabilities", json_array(capabilities)));

    let diagnostics = resolved
        .diagnostics
        .iter()
        .map(resolve_diagnostic_json)
        .collect::<Vec<_>>();
    fields.push(("diagnostics", json_array(diagnostics)));

    json_object(fields)
}

fn ir_module_to_json(module: &ir::Module) -> String {
    let functions = module
        .functions
        .iter()
        .map(|function| ir_function_json(module, function))
        .collect::<Vec<_>>();
    let types = module
        .types
        .entries()
        .map(|(id, ty)| ir_type_entry_json(id, ty))
        .collect::<Vec<_>>();
    let effects = module
        .effects
        .entries()
        .map(|(id, name)| {
            json_object(vec![
                ("id", id.index().to_string()),
                ("name", json_string(name)),
            ])
        })
        .collect::<Vec<_>>();

    json_object(vec![
        ("name", json_string_array(&module.name)),
        ("functions", json_array(functions)),
        ("types", json_array(types)),
        ("effects", json_array(effects)),
    ])
}

fn ir_function_json(module: &ir::Module, function: &ir::Function) -> String {
    let params = function
        .params
        .iter()
        .map(|param| {
            json_object(vec![
                ("name", json_string(&param.name)),
                ("type", param.ty.index().to_string()),
                ("value", param.value.index().to_string()),
            ])
        })
        .collect::<Vec<_>>();

    let blocks = function
        .blocks
        .iter()
        .map(|block| ir_block_json(module, block))
        .collect::<Vec<_>>();

    let effects = function
        .effect_row
        .iter()
        .map(|effect| module.effects.name(*effect).to_string())
        .collect::<Vec<_>>();

    json_object(vec![
        ("name", json_string(&function.name)),
        ("ret_type", function.ret_type.index().to_string()),
        ("effect_row", json_string_array(&effects)),
        ("params", json_array(params)),
        ("blocks", json_array(blocks)),
    ])
}

fn ir_block_json(module: &ir::Module, block: &ir::BasicBlock) -> String {
    let instructions = block
        .instructions
        .iter()
        .map(|inst| ir_instruction_json(module, inst))
        .collect::<Vec<_>>();

    json_object(vec![
        ("id", block.id.index().to_string()),
        ("instructions", json_array(instructions)),
        ("terminator", ir_terminator_json(module, &block.terminator)),
    ])
}

fn ir_instruction_json(module: &ir::Module, inst: &ir::Instruction) -> String {
    let effects = inst
        .effects
        .iter()
        .map(|effect| module.effects.name(*effect).to_string())
        .collect::<Vec<_>>();

    json_object(vec![
        ("id", inst.id.index().to_string()),
        ("type", inst.ty.index().to_string()),
        ("effects", json_string_array(&effects)),
        ("kind", ir_instruction_kind_json(module, &inst.kind)),
    ])
}

fn ir_instruction_kind_json(_module: &ir::Module, kind: &ir::InstKind) -> String {
    match kind {
        ir::InstKind::Literal(lit) => json_object(vec![
            ("kind", json_string("Literal")),
            ("value", literal_json(lit)),
        ]),
        ir::InstKind::Binary { op, lhs, rhs } => json_object(vec![
            ("kind", json_string("Binary")),
            ("op", json_string(&op.to_string())),
            ("lhs", lhs.index().to_string()),
            ("rhs", rhs.index().to_string()),
        ]),
        ir::InstKind::Call { func, args } => {
            let function_json = match func {
                ir::FuncRef::Function(path) => json_object(vec![
                    ("type", json_string("Function")),
                    ("path", json_string_array(&path.segments)),
                ]),
                ir::FuncRef::Method(name) => json_object(vec![
                    ("type", json_string("Method")),
                    ("name", json_string(name)),
                ]),
            };
            let args_json = json_array(
                args.iter()
                    .map(|arg| arg.index().to_string())
                    .collect::<Vec<_>>(),
            );
            json_object(vec![
                ("kind", json_string("Call")),
                ("function", function_json),
                ("args", args_json),
            ])
        }
        ir::InstKind::Record { type_path, fields } => {
            let type_json = type_path
                .as_ref()
                .map(|path| json_string_array(&path.segments))
                .unwrap_or_else(|| "null".to_string());
            let field_json = json_array(
                fields
                    .iter()
                    .map(|(name, value)| {
                        json_object(vec![
                            ("name", json_string(name)),
                            ("value", value.index().to_string()),
                        ])
                    })
                    .collect::<Vec<_>>(),
            );
            json_object(vec![
                ("kind", json_string("Record")),
                ("type_path", type_json),
                ("fields", field_json),
            ])
        }
        ir::InstKind::Path(path) => json_object(vec![
            ("kind", json_string("Path")),
            ("segments", json_string_array(&path.segments)),
        ]),
        ir::InstKind::Phi { incomings } => {
            let incomings_json = json_array(
                incomings
                    .iter()
                    .map(|(block, value)| {
                        json_object(vec![
                            ("block", block.index().to_string()),
                            ("value", value.index().to_string()),
                        ])
                    })
                    .collect::<Vec<_>>(),
            );
            json_object(vec![
                ("kind", json_string("Phi")),
                ("incomings", incomings_json),
            ])
        }
    }
}

fn ir_terminator_json(_module: &ir::Module, terminator: &ir::Terminator) -> String {
    match terminator {
        ir::Terminator::Return(Some(value)) => json_object(vec![
            ("kind", json_string("Return")),
            ("value", value.index().to_string()),
        ]),
        ir::Terminator::Return(None) => json_object(vec![
            ("kind", json_string("Return")),
            ("value", "null".to_string()),
        ]),
        ir::Terminator::Branch {
            condition,
            then_block,
            else_block,
        } => json_object(vec![
            ("kind", json_string("Branch")),
            ("condition", condition.index().to_string()),
            ("then", then_block.index().to_string()),
            ("else", else_block.index().to_string()),
        ]),
        ir::Terminator::Jump(target) => json_object(vec![
            ("kind", json_string("Jump")),
            ("target", target.index().to_string()),
        ]),
    }
}

fn ir_type_entry_json(id: ir::TypeId, ty: &ir::Type) -> String {
    json_object(vec![
        ("id", id.index().to_string()),
        ("type", ir_type_json(ty)),
    ])
}

fn ir_type_json(ty: &ir::Type) -> String {
    match ty {
        ir::Type::Unit => json_object(vec![("kind", json_string("Unit"))]),
        ir::Type::Int => json_object(vec![("kind", json_string("Int"))]),
        ir::Type::Float => json_object(vec![("kind", json_string("Float"))]),
        ir::Type::Bool => json_object(vec![("kind", json_string("Bool"))]),
        ir::Type::String => json_object(vec![("kind", json_string("String"))]),
        ir::Type::Named(name) => json_object(vec![
            ("kind", json_string("Named")),
            ("name", json_string(name)),
        ]),
        ir::Type::Record(record) => {
            let fields = record
                .fields
                .iter()
                .map(|field| {
                    json_object(vec![
                        ("name", json_string(&field.name)),
                        ("type", field.ty.index().to_string()),
                        ("offset", field.offset.to_string()),
                    ])
                })
                .collect::<Vec<_>>();
            json_object(vec![
                ("kind", json_string("Record")),
                (
                    "name",
                    record
                        .name
                        .as_ref()
                        .map(|name| json_string(name))
                        .unwrap_or_else(|| "null".to_string()),
                ),
                ("fields", json_array(fields)),
                ("size", record.size.to_string()),
                ("align", record.align.to_string()),
            ])
        }
        ir::Type::Unknown => json_object(vec![("kind", json_string("Unknown"))]),
    }
}

fn literal_json(literal: &ast::Literal) -> String {
    match literal {
        ast::Literal::Int(value) => json_object(vec![
            ("type", json_string("Int")),
            ("value", value.to_string()),
        ]),
        ast::Literal::Float(value) => json_object(vec![
            ("type", json_string("Float")),
            ("value", value.to_string()),
        ]),
        ast::Literal::Bool(value) => json_object(vec![
            ("type", json_string("Bool")),
            ("value", bool_string(*value)),
        ]),
        ast::Literal::String(value) => json_object(vec![
            ("type", json_string("String")),
            ("value", json_string(value)),
        ]),
        ast::Literal::Unit => json_object(vec![("type", json_string("Unit"))]),
    }
}

fn resolved_symbol_json(symbol: &resolve::SymbolInfo) -> String {
    json_object(vec![
        ("name", json_string(&symbol.name)),
        ("category", symbol_category_json(&symbol.category)),
        ("scope", symbol_scope_json(&symbol.scope)),
    ])
}

fn resolved_path_json(path: &resolve::ResolvedPath) -> String {
    let resolved_json = path
        .resolved
        .as_ref()
        .map(resolved_symbol_json)
        .unwrap_or_else(|| "null".to_string());
    json_object(vec![
        ("segments", json_string_array(&path.segments)),
        ("kind", json_string(path_kind_name(path.kind))),
        ("resolved", resolved_json),
    ])
}

fn capability_binding_json(binding: &resolve::CapabilityBinding) -> String {
    json_object(vec![
        ("name", json_string(&binding.name)),
        ("scope", capability_scope_json(&binding.scope)),
    ])
}

fn resolve_diagnostic_json(diag: &resolve::ResolveDiagnostic) -> String {
    json_object(vec![
        ("path", json_string_array(&diag.path)),
        ("kind", json_string(path_kind_name(diag.kind))),
        ("scope", symbol_scope_json(&diag.scope)),
        ("message", json_string(&diag.message)),
    ])
}

fn symbol_category_json(category: &resolve::SymbolCategory) -> String {
    match category {
        resolve::SymbolCategory::Type { is_public, params } => json_object(vec![
            ("type", json_string("Type")),
            ("is_public", bool_string(*is_public)),
            ("params", json_string_array(params)),
        ]),
        resolve::SymbolCategory::Variant { parent } => json_object(vec![
            ("type", json_string("Variant")),
            ("parent", json_string(parent)),
        ]),
        resolve::SymbolCategory::Function { is_public } => json_object(vec![
            ("type", json_string("Function")),
            ("is_public", bool_string(*is_public)),
        ]),
        resolve::SymbolCategory::TypeParam => json_object(vec![("type", json_string("TypeParam"))]),
        resolve::SymbolCategory::ValueParam => {
            json_object(vec![("type", json_string("ValueParam"))])
        }
        resolve::SymbolCategory::LocalBinding => {
            json_object(vec![("type", json_string("LocalBinding"))])
        }
        resolve::SymbolCategory::ImportAlias { target } => json_object(vec![
            ("type", json_string("ImportAlias")),
            ("target", json_string_array(target)),
        ]),
    }
}

fn symbol_scope_json(scope: &resolve::SymbolScope) -> String {
    match scope {
        resolve::SymbolScope::Module(path) => json_object(vec![
            ("type", json_string("Module")),
            ("path", json_string_array(path)),
        ]),
        resolve::SymbolScope::TypeAlias {
            module_path,
            type_name,
        } => json_object(vec![
            ("type", json_string("TypeAlias")),
            ("module_path", json_string_array(module_path)),
            ("type_name", json_string(type_name)),
        ]),
        resolve::SymbolScope::Function {
            module_path,
            function,
        } => json_object(vec![
            ("type", json_string("Function")),
            ("module_path", json_string_array(module_path)),
            ("function", json_string(function)),
        ]),
    }
}

fn capability_scope_json(scope: &resolve::CapabilityScope) -> String {
    match scope {
        resolve::CapabilityScope::Function {
            module_path,
            function,
        } => json_object(vec![
            ("type", json_string("Function")),
            ("module_path", json_string_array(module_path)),
            ("function", json_string(function)),
        ]),
        resolve::CapabilityScope::TypeAlias {
            module_path,
            type_name,
        } => json_object(vec![
            ("type", json_string("TypeAlias")),
            ("module_path", json_string_array(module_path)),
            ("type_name", json_string(type_name)),
        ]),
    }
}

fn path_kind_name(kind: resolve::PathKind) -> &'static str {
    match kind {
        resolve::PathKind::Type => "Type",
        resolve::PathKind::Value => "Value",
        resolve::PathKind::Variant => "Variant",
    }
}

fn json_string(value: &str) -> String {
    format!("\"{}\"", json_escape(value))
}

fn json_string_array(values: &[String]) -> String {
    json_array(values.iter().map(|value| json_string(value)).collect())
}

fn json_array(values: Vec<String>) -> String {
    let mut out = String::from("[");
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        out.push_str(value);
    }
    out.push(']');
    out
}

fn json_object(fields: Vec<(&str, String)>) -> String {
    let mut out = String::from("{");
    for (index, (key, value)) in fields.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        out.push('"');
        out.push_str(&json_escape(key));
        out.push_str("\":");
        out.push_str(value);
    }
    out.push('}');
    out
}

fn json_object_pairs(pairs: Vec<(String, String)>) -> String {
    let mut out = String::from("{");
    for (index, (key, value)) in pairs.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        out.push('"');
        out.push_str(&json_escape(key));
        out.push_str("\":");
        out.push_str(value);
    }
    out.push('}');
    out
}

fn json_escape(value: &str) -> String {
    let mut escaped = String::new();
    for ch in value.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            ch if ch < ' ' => escaped.push_str(&format!("\\u{:04x}", ch as u32)),
            other => escaped.push(other),
        }
    }
    escaped
}

fn bool_string(value: bool) -> String {
    if value { "true" } else { "false" }.to_string()
}
#[cfg(test)]
mod cli_tests {
    use super::*;

    #[test]
    fn entry_task_spec_collects_main_capabilities() {
        let module = ast::Module {
            name: vec!["demo".into()],
            items: vec![ast::Item::Function(ast::Function {
                is_public: false,
                name: "main".into(),
                generics: Vec::new(),
                params: Vec::new(),
                return_type: None,
                effect_row: vec!["io".into()],
                body: ast::Block {
                    statements: Vec::new(),
                },
            })],
        };

        let mut resolved = resolve::Resolved {
            module_path: vec!["demo".into()],
            ..Default::default()
        };
        resolved.capabilities.push(resolve::CapabilityBinding {
            name: "io".into(),
            scope: CapabilityScope::Function {
                module_path: vec!["demo".into()],
                function: "main".into(),
            },
        });

        let spec = entry_task_spec(&module, &resolved).expect("entry spec");
        assert_eq!(spec.name(), "demo::main");
        assert_eq!(spec.capabilities(), &[String::from("io")]);
    }

    #[test]
    fn entry_task_spec_ignores_modules_without_main() {
        let module = ast::Module {
            name: vec!["demo".into()],
            items: Vec::new(),
        };
        let resolved = resolve::Resolved::default();
        assert!(entry_task_spec(&module, &resolved).is_none());
    }

    #[test]
    fn build_trace_json_captures_stdout_lines() {
        let json = build_trace_json("demo::main", "first\nsecond\n", "");

        assert!(json.contains("\"task\":\"demo::main\""));
        assert!(json.contains("\"value\":\"first\""));
        assert!(json.contains("\"value\":\"second\""));
        assert!(json.contains("\"capability_counts\":{\"io\":2}"));
        assert!(json.contains("\"operation_counts\":{\"io::write_line\":2}"));
        assert!(json.contains("\"event_count\":6"));
    }

    #[test]
    fn build_trace_json_annotations_include_stderr_messages() {
        let json = build_trace_json("demo::main", "", "failure\n");

        assert!(json.contains("stderr: failure"));
        assert!(json.contains("\"capability_counts\":{}"));
        assert!(json.contains("\"operation_counts\":{}"));
        assert!(json.contains("\"event_count\":3"));
    }
}
