use std::collections::HashMap;
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
        let mut renderer = ModuleRenderer::new(module, triple.clone());
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
}

impl<'m> ModuleRenderer<'m> {
    fn new(module: &'m ir::Module, target_triple: Option<String>) -> Self {
        ModuleRenderer {
            module,
            target_triple,
        }
    }

    fn render(&mut self) -> String {
        let mut out = String::new();
        let module_name = self.module.name.join(".");
        writeln!(out, "; ModuleID = '{}'", module_name).unwrap();
        if let Some(triple) = &self.target_triple {
            writeln!(out, "target triple = \"{}\"", triple).unwrap();
        }
        writeln!(out).unwrap();

        for function in &self.module.functions {
            render_function(&mut out, self.module, function);
            writeln!(out).unwrap();
        }

        out
    }
}

fn render_function(out: &mut String, module: &ir::Module, function: &ir::Function) {
    let ret_ty = format_type(module, function.ret_type);
    let mut params = Vec::with_capacity(function.params.len());
    let mut value_types: HashMap<ValueId, TypeId> = HashMap::new();

    for param in &function.params {
        value_types.insert(param.value, param.ty);
        params.push(format!("{} %{}", format_type(module, param.ty), param.name));
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
            .map(|effect| module.effect_name(*effect).to_string())
            .collect();
        writeln!(out, "  ; effects: {}", names.join(", ")).unwrap();
    }

    for block in &function.blocks {
        render_block(out, module, block, &mut value_types);
    }

    writeln!(out, "}}").unwrap();
}

fn render_block(
    out: &mut String,
    module: &ir::Module,
    block: &ir::BasicBlock,
    value_types: &mut HashMap<ValueId, TypeId>,
) {
    writeln!(out, "bb{}:", block.id.index()).unwrap();
    for inst in &block.instructions {
        value_types.insert(inst.id, inst.ty);
        let ty = format_type(module, inst.ty);
        writeln!(
            out,
            "  ; %{} : {} = {}",
            inst.id.index(),
            ty,
            format_inst(inst)
        )
        .unwrap();
    }
    render_terminator(out, module, &block.terminator, value_types);
}

fn render_terminator(
    out: &mut String,
    module: &ir::Module,
    terminator: &Terminator,
    value_types: &HashMap<ValueId, TypeId>,
) {
    match terminator {
        Terminator::Return(Some(value)) => {
            let ty = value_types
                .get(value)
                .copied()
                .unwrap_or_else(|| module.unknown_type());
            writeln!(out, "  ret {} %{}", format_type(module, ty), value.index()).unwrap();
        }
        Terminator::Return(None) => {
            writeln!(out, "  ret void").unwrap();
        }
        Terminator::Branch {
            condition,
            then_block,
            else_block,
        } => {
            let ty = value_types
                .get(condition)
                .copied()
                .unwrap_or_else(|| module.unknown_type());
            let cond_ty = format_type(module, ty);
            writeln!(
                out,
                "  br {} %{}, label %bb{}, label %bb{}",
                cond_ty,
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

fn format_inst(inst: &ir::Instruction) -> String {
    match &inst.kind {
        InstKind::Literal(literal) => format!("literal {}", format_literal(literal)),
        InstKind::Binary { op, lhs, rhs } => format!("{} %{}, %{}", op, lhs.index(), rhs.index()),
        InstKind::Call { func, args } => {
            let mut rendered_args = Vec::with_capacity(args.len());
            for arg in args {
                rendered_args.push(format!("%{}", arg.index()));
            }
            match func {
                ir::FuncRef::Function(path) => format!(
                    "call @{}({})",
                    path.segments.join("::"),
                    rendered_args.join(", ")
                ),
                ir::FuncRef::Method(name) => {
                    format!("call %{}({})", name, rendered_args.join(", "))
                }
            }
        }
        InstKind::Record { type_path, fields } => {
            let mut formatted = Vec::new();
            for (name, value) in fields {
                formatted.push(format!("{}: %{}", name, value.index()));
            }
            match type_path {
                Some(path) => format!(
                    "record {} {{{}}}",
                    path.segments.join("::"),
                    formatted.join(", ")
                ),
                None => format!("record {{{}}}", formatted.join(", ")),
            }
        }
        InstKind::Path(path) => format!("path {}", path.segments.join("::")),
        InstKind::Phi { incomings } => {
            let mut parts = Vec::new();
            for (block, value) in incomings {
                parts.push(format!("[ %{}, %bb{} ]", value.index(), block.index()));
            }
            format!("phi {{{}}}", parts.join(", "))
        }
    }
}

fn format_literal(literal: &crate::syntax::ast::Literal) -> String {
    match literal {
        crate::syntax::ast::Literal::Int(value) => value.to_string(),
        crate::syntax::ast::Literal::Float(value) => value.to_string(),
        crate::syntax::ast::Literal::Bool(value) => value.to_string(),
        crate::syntax::ast::Literal::String(value) => format!("\"{}\"", value),
        crate::syntax::ast::Literal::Unit => "()".to_string(),
    }
}

fn format_type(module: &ir::Module, ty: TypeId) -> String {
    match module.type_of(ty) {
        Type::Unit => "void".to_string(),
        Type::Int => "i64".to_string(),
        Type::Float => "double".to_string(),
        Type::Bool => "i1".to_string(),
        Type::String => "ptr".to_string(),
        Type::Named(name) => format!("%{}", sanitize_symbol(name)),
        Type::Unknown => "ptr".to_string(),
    }
}

fn sanitize_symbol(name: &str) -> String {
    name.replace(['.', ':'], "_")
}
