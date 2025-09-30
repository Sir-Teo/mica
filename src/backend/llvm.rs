use std::collections::{HashMap, HashSet};
use std::fmt::Write;

use crate::ir::{self, InstKind, Terminator, Type, TypeId, ValueId};

use super::{Backend, BackendOptions, BackendResult};

/// Scaffolding backend that translates the typed SSA module into a
/// lightweight LLVM-flavoured IR string. The intent is to expose a stable
/// contract for future native code generation work without pulling in LLVM
/// as a dependency yet.
#[derive(Debug, Default, Clone)]
pub struct LlvmBackend {
    /// Preferred target triple for the emitted module. When `None`, the
    /// backend falls back to `BackendOptions::target_triple` or leaves the
    /// triple unspecified so later stages can decide.
    pub target_triple: Option<String>,
}

/// Result of the LLVM backend scaffolding. The IR is stored verbatim so
/// downstream tooling can persist it to disk or feed it into the real LLVM
/// toolchain when it lands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LlvmModule {
    pub ir: String,
    pub target_triple: Option<String>,
}

impl LlvmModule {
    pub fn as_str(&self) -> &str {
        &self.ir
    }
}

impl Backend for LlvmBackend {
    type Output = LlvmModule;

    fn compile(
        &self,
        module: &ir::Module,
        options: &BackendOptions,
    ) -> BackendResult<Self::Output> {
        let triple = self
            .target_triple
            .clone()
            .or_else(|| options.target_triple.clone());
        let renderer = ModuleRenderer::new(module, triple.clone());
        let ir = renderer.render();
        Ok(LlvmModule {
            ir,
            target_triple: triple,
        })
    }
}

struct ModuleRenderer<'m> {
    module: &'m ir::Module,
    target_triple: Option<String>,
    string_literals: Vec<String>,
    string_map: HashMap<String, usize>,
}

impl<'m> ModuleRenderer<'m> {
    fn new(module: &'m ir::Module, target_triple: Option<String>) -> Self {
        ModuleRenderer {
            module,
            target_triple,
            string_literals: Vec::new(),
            string_map: HashMap::new(),
        }
    }

    fn render(mut self) -> String {
        let mut functions = String::new();
        for function in &self.module.functions {
            self.render_function(&mut functions, function);
            writeln!(functions).unwrap();
        }

        let mut out = String::new();
        let module_name = self.module.name.join(".");
        writeln!(out, "; ModuleID = '{}'", module_name).unwrap();
        if let Some(triple) = &self.target_triple {
            writeln!(out, "target triple = \"{}\"", triple).unwrap();
        }
        writeln!(out).unwrap();

        for (index, value) in self.string_literals.iter().enumerate() {
            let symbol = format!(".str{}", index);
            let escaped = escape_string(value);
            let len = value.as_bytes().len() + 1;
            writeln!(
                out,
                "@{} = private constant [{} x i8] c\"{}\\00\"",
                symbol, len, escaped
            )
            .unwrap();
        }

        if !self.string_literals.is_empty() {
            writeln!(out).unwrap();
        }

        out.push_str(&functions);
        out
    }

    fn render_function(&mut self, out: &mut String, function: &ir::Function) {
        let ret_ty = format_type(self.module, function.ret_type);
        let mut params = Vec::with_capacity(function.params.len());
        let mut context = RenderContext::new(self.module);

        for param in &function.params {
            context.value_types.insert(param.value, param.ty);
            params.push(format!(
                "{} %{}",
                format_type(self.module, param.ty),
                param.name
            ));
        }

        writeln!(
            out,
            "define {} @{}({}) {{",
            ret_ty,
            function.name,
            params.join(", ")
        )
        .unwrap();

        if !function.effect_row.is_empty() {
            let names: Vec<_> = function
                .effect_row
                .iter()
                .map(|effect| self.module.effect_name(*effect).to_string())
                .collect();
            writeln!(out, "  ; effects: {}", names.join(", ")).unwrap();
        }

        for block in &function.blocks {
            self.render_block(out, block, &mut context);
        }

        writeln!(out, "}}").unwrap();
    }

    fn render_block(
        &mut self,
        out: &mut String,
        block: &ir::BasicBlock,
        context: &mut RenderContext<'_>,
    ) {
        writeln!(out, "bb{}:", block.id.index()).unwrap();
        for inst in &block.instructions {
            context.value_types.insert(inst.id, inst.ty);
            if let Some(line) = self.render_instruction(inst, context) {
                writeln!(out, "{}", line).unwrap();
            }
        }
        self.render_terminator(out, &block.terminator, context);
    }

    fn render_instruction(
        &mut self,
        inst: &ir::Instruction,
        context: &mut RenderContext<'_>,
    ) -> Option<String> {
        match &inst.kind {
            InstKind::Literal(literal) => self.render_literal(inst, literal, context),
            InstKind::Binary { op, lhs, rhs } => {
                Some(render_binary(inst, *op, *lhs, *rhs, context))
            }
            InstKind::Call { func, args } => Some(self.render_call(inst, func, args, context)),
            InstKind::Record { type_path, fields } => {
                Some(render_record_stub(inst, type_path, fields, context))
            }
            InstKind::Path(path) => Some(render_path(inst, path)),
            InstKind::Phi { incomings } => Some(render_phi(inst, incomings, context)),
        }
    }

    fn render_literal(
        &mut self,
        inst: &ir::Instruction,
        literal: &crate::syntax::ast::Literal,
        context: &mut RenderContext<'_>,
    ) -> Option<String> {
        let id = inst.id.index();
        match literal {
            crate::syntax::ast::Literal::Int(value) => Some(format!(
                "  %{} = add {} 0, {}",
                id,
                format_type(context.module, inst.ty),
                value
            )),
            crate::syntax::ast::Literal::Float(value) => Some(format!(
                "  %{} = fadd {} 0.0, {:.6e}",
                id,
                format_type(context.module, inst.ty),
                value
            )),
            crate::syntax::ast::Literal::Bool(value) => Some(format!(
                "  %{} = or {} false, {}",
                id,
                format_type(context.module, inst.ty),
                if *value { "true" } else { "false" }
            )),
            crate::syntax::ast::Literal::String(value) => {
                let symbol = self.intern_string(value);
                Some(format!(
                    "  %{} = getelementptr inbounds ([{} x i8], ptr @{}, i32 0, i32 0)",
                    id,
                    value.as_bytes().len() + 1,
                    symbol
                ))
            }
            crate::syntax::ast::Literal::Unit => {
                context.unit_values.insert(inst.id);
                None
            }
        }
    }

    fn render_call(
        &mut self,
        inst: &ir::Instruction,
        func: &ir::FuncRef,
        args: &[ValueId],
        context: &RenderContext<'_>,
    ) -> String {
        let ret_ty = format_type(context.module, inst.ty);
        let callee = match func {
            ir::FuncRef::Function(path) => {
                format!("@{}", sanitize_symbol(&path.segments.join("::")))
            }
            ir::FuncRef::Method(name) => format!("@{}", sanitize_symbol(name)),
        };
        let formatted_args: Vec<String> = args
            .iter()
            .map(|arg| {
                let ty = context
                    .value_types
                    .get(arg)
                    .copied()
                    .unwrap_or_else(|| context.module.unknown_type());
                format!("{} %{}", format_type(context.module, ty), arg.index())
            })
            .collect();
        if ret_ty == "void" {
            format!(
                "  call {} {}({})",
                ret_ty,
                callee,
                formatted_args.join(", ")
            )
        } else {
            format!(
                "  %{} = call {} {}({})",
                inst.id.index(),
                ret_ty,
                callee,
                formatted_args.join(", ")
            )
        }
    }

    fn render_terminator(
        &self,
        out: &mut String,
        terminator: &Terminator,
        context: &RenderContext<'_>,
    ) {
        match terminator {
            Terminator::Return(Some(value)) => {
                if context.unit_values.contains(value) {
                    writeln!(out, "  ret void").unwrap();
                    return;
                }
                let ty = context
                    .value_types
                    .get(value)
                    .copied()
                    .unwrap_or_else(|| context.module.unknown_type());
                writeln!(
                    out,
                    "  ret {} %{}",
                    format_type(context.module, ty),
                    value.index()
                )
                .unwrap();
            }
            Terminator::Return(None) => {
                writeln!(out, "  ret void").unwrap();
            }
            Terminator::Branch {
                condition,
                then_block,
                else_block,
            } => {
                let ty = context
                    .value_types
                    .get(condition)
                    .copied()
                    .unwrap_or_else(|| context.module.unknown_type());
                writeln!(
                    out,
                    "  br {} %{}, label %bb{}, label %bb{}",
                    format_type(context.module, ty),
                    condition.index(),
                    then_block.index(),
                    else_block.index()
                )
                .unwrap();
            }
            Terminator::Jump(target) => {
                writeln!(out, "  br label %bb{}", target.index()).unwrap();
            }
        }
    }

    fn intern_string(&mut self, value: &str) -> String {
        if let Some(index) = self.string_map.get(value) {
            return format!(".str{}", index);
        }
        let index = self.string_literals.len();
        self.string_literals.push(value.to_string());
        self.string_map.insert(value.to_string(), index);
        format!(".str{}", index)
    }
}

struct RenderContext<'m> {
    module: &'m ir::Module,
    value_types: HashMap<ValueId, TypeId>,
    unit_values: HashSet<ValueId>,
}

impl<'m> RenderContext<'m> {
    fn new(module: &'m ir::Module) -> Self {
        RenderContext {
            module,
            value_types: HashMap::new(),
            unit_values: HashSet::new(),
        }
    }
}

fn render_binary(
    inst: &ir::Instruction,
    op: crate::syntax::ast::BinaryOp,
    lhs: ValueId,
    rhs: ValueId,
    context: &RenderContext<'_>,
) -> String {
    let result_id = inst.id.index();
    let lhs_ty = context
        .value_types
        .get(&lhs)
        .copied()
        .unwrap_or_else(|| context.module.unknown_type());
    let ty_name = format_type(context.module, lhs_ty);
    match op {
        crate::syntax::ast::BinaryOp::Add => {
            binary_arith("add", "fadd", ty_name, lhs, rhs, result_id)
        }
        crate::syntax::ast::BinaryOp::Sub => {
            binary_arith("sub", "fsub", ty_name, lhs, rhs, result_id)
        }
        crate::syntax::ast::BinaryOp::Mul => {
            binary_arith("mul", "fmul", ty_name, lhs, rhs, result_id)
        }
        crate::syntax::ast::BinaryOp::Div => {
            binary_arith("sdiv", "fdiv", ty_name, lhs, rhs, result_id)
        }
        crate::syntax::ast::BinaryOp::Mod => {
            binary_arith("srem", "frem", ty_name, lhs, rhs, result_id)
        }
        crate::syntax::ast::BinaryOp::And => format!(
            "  %{} = and {} %{}, %{}",
            result_id,
            ty_name,
            lhs.index(),
            rhs.index()
        ),
        crate::syntax::ast::BinaryOp::Or => format!(
            "  %{} = or {} %{}, %{}",
            result_id,
            ty_name,
            lhs.index(),
            rhs.index()
        ),
        crate::syntax::ast::BinaryOp::Eq => render_cmp("eq", "oeq", ty_name, lhs, rhs, result_id),
        crate::syntax::ast::BinaryOp::Ne => render_cmp("ne", "one", ty_name, lhs, rhs, result_id),
        crate::syntax::ast::BinaryOp::Lt => render_cmp("slt", "olt", ty_name, lhs, rhs, result_id),
        crate::syntax::ast::BinaryOp::Le => render_cmp("sle", "ole", ty_name, lhs, rhs, result_id),
        crate::syntax::ast::BinaryOp::Gt => render_cmp("sgt", "ogt", ty_name, lhs, rhs, result_id),
        crate::syntax::ast::BinaryOp::Ge => render_cmp("sge", "oge", ty_name, lhs, rhs, result_id),
    }
}

fn binary_arith(
    int_op: &str,
    float_op: &str,
    ty_name: String,
    lhs: ValueId,
    rhs: ValueId,
    result_id: u32,
) -> String {
    if ty_name == "double" {
        format!(
            "  %{} = {} double %{}, %{}",
            result_id,
            float_op,
            lhs.index(),
            rhs.index()
        )
    } else {
        format!(
            "  %{} = {} {} %{}, %{}",
            result_id,
            int_op,
            ty_name,
            lhs.index(),
            rhs.index()
        )
    }
}

fn render_cmp(
    int_pred: &str,
    float_pred: &str,
    ty_name: String,
    lhs: ValueId,
    rhs: ValueId,
    result_id: u32,
) -> String {
    if ty_name == "double" {
        format!(
            "  %{} = fcmp {} double %{}, %{}",
            result_id,
            float_pred,
            lhs.index(),
            rhs.index()
        )
    } else {
        let cmp_ty = if ty_name == "void" {
            "i1".to_string()
        } else {
            ty_name
        };
        format!(
            "  %{} = icmp {} {} %{}, %{}",
            result_id,
            int_pred,
            cmp_ty,
            lhs.index(),
            rhs.index()
        )
    }
}

fn render_path(inst: &ir::Instruction, path: &crate::syntax::ast::Path) -> String {
    let symbol = sanitize_symbol(&path.segments.join("::"));
    format!("  %{} = bitcast ptr @{} to ptr", inst.id.index(), symbol)
}

fn render_phi(
    inst: &ir::Instruction,
    incomings: &[(crate::ir::BlockId, ValueId)],
    context: &RenderContext<'_>,
) -> String {
    let ty = format_type(context.module, inst.ty);
    let mut parts = Vec::new();
    for (block, value) in incomings {
        parts.push(format!("[ %{}, %bb{} ]", value.index(), block.index()));
    }
    format!("  %{} = phi {} {}", inst.id.index(), ty, parts.join(", "))
}

fn render_record_stub(
    inst: &ir::Instruction,
    _type_path: &Option<crate::syntax::ast::Path>,
    fields: &[(String, ValueId)],
    context: &RenderContext<'_>,
) -> String {
    let ret_ty = format_type(context.module, inst.ty);
    let field_args: Vec<String> = fields
        .iter()
        .map(|(_, value)| {
            let ty = context
                .value_types
                .get(value)
                .copied()
                .unwrap_or_else(|| context.module.unknown_type());
            format!("{} %{}", format_type(context.module, ty), value.index())
        })
        .collect();
    format!(
        "  %{} = call {} @__mica_record_stub({})",
        inst.id.index(),
        ret_ty,
        field_args.join(", ")
    )
}

fn escape_string(value: &str) -> String {
    value
        .chars()
        .flat_map(|ch| match ch {
            '\\' => Some("\\5C".to_string()),
            '"' => Some("\\22".to_string()),
            '\n' => Some("\\0A".to_string()),
            '\r' => Some("\\0D".to_string()),
            '\t' => Some("\\09".to_string()),
            c if c.is_ascii_graphic() || c == ' ' => Some(c.to_string()),
            other => Some(format!("\\{:02X}", other as u32)),
        })
        .collect()
}

fn format_type(module: &ir::Module, ty: TypeId) -> String {
    match module.type_of(ty) {
        Type::Unit => "void".to_string(),
        Type::Int => "i64".to_string(),
        Type::Float => "double".to_string(),
        Type::Bool => "i1".to_string(),
        Type::String => "ptr".to_string(),
        Type::Named(_name) => "ptr".to_string(),
        Type::Unknown => "ptr".to_string(),
    }
}

fn sanitize_symbol(name: &str) -> String {
    name.replace(['.', ':'], "_")
}
