use std::fmt::Write;

use crate::ir::{self, InstKind, Terminator, Type};

use super::{Backend, BackendOptions, BackendResult};

#[derive(Debug, Default)]
pub struct TextBackend;

impl Backend for TextBackend {
    type Output = String;

    fn compile(
        &self,
        module: &ir::Module,
        _options: &BackendOptions,
    ) -> BackendResult<Self::Output> {
        Ok(render_module(module))
    }
}

pub fn render_module(module: &ir::Module) -> String {
    let mut out = String::new();
    writeln!(out, "module {}", module.name.join(".")).unwrap();
    for function in &module.functions {
        writeln!(out).unwrap();
        render_function(&mut out, module, function);
    }
    out
}

fn render_function(out: &mut String, module: &ir::Module, function: &ir::Function) {
    write!(out, "fn {}(", function.name).unwrap();
    for (index, param) in function.params.iter().enumerate() {
        if index > 0 {
            write!(out, ", ").unwrap();
        }
        let ty = module.type_of(param.ty);
        write!(out, "{}: {}", param.name, format_type(ty)).unwrap();
    }
    write!(out, ")").unwrap();
    let ret_type = module.type_of(function.ret_type);
    if !matches!(ret_type, Type::Unit) {
        write!(out, " -> {}", format_type(ret_type)).unwrap();
    }
    if !function.effect_row.is_empty() {
        let names: Vec<_> = function
            .effect_row
            .iter()
            .map(|id| module.effect_name(*id))
            .collect();
        write!(out, " !{{{}}}", names.join(", ")).unwrap();
    }
    writeln!(out).unwrap();
    for block in &function.blocks {
        writeln!(out, "  block {}:", block.id.index()).unwrap();
        for inst in &block.instructions {
            let ty = module.type_of(inst.ty);
            writeln!(
                out,
                "    %{} = {} : {}",
                inst.id.index(),
                format_inst(inst),
                format_type(ty)
            )
            .unwrap();
        }
        writeln!(out, "    {}", format_terminator(block)).unwrap();
    }
}

fn format_inst(inst: &ir::Instruction) -> String {
    match &inst.kind {
        InstKind::Literal(lit) => format_literal(lit),
        InstKind::Binary { op, lhs, rhs } => format!("{} %{}, %{}", op, lhs.index(), rhs.index()),
        InstKind::Call { func, args } => {
            let name = match func {
                ir::FuncRef::Function(path) => path.segments.join("::"),
                ir::FuncRef::Method(name) => name.clone(),
            };
            let mut parts = Vec::with_capacity(args.len());
            for arg in args {
                parts.push(format!("%{}", arg.index()));
            }
            format!("call {}({})", name, parts.join(", "))
        }
        InstKind::Record { type_path, fields } => {
            let mut parts = Vec::new();
            for (name, value) in fields {
                parts.push(format!("{}: %{}", name, value.index()));
            }
            match type_path {
                Some(path) => format!(
                    "record {} {{ {} }}",
                    path.segments.join("::"),
                    parts.join(", ")
                ),
                None => format!("record {{ {} }}", parts.join(", ")),
            }
        }
        InstKind::Path(path) => format!("path {}", path.segments.join("::")),
        InstKind::Phi { incomings } => {
            let mut parts = Vec::new();
            for (block, value) in incomings {
                parts.push(format!("bb{}: %{}", block.index(), value.index()));
            }
            format!("phi {{ {} }}", parts.join(", "))
        }
    }
}

fn format_terminator(block: &ir::BasicBlock) -> String {
    match &block.terminator {
        Terminator::Return(Some(value)) => {
            format!("return %{}", value.index())
        }
        Terminator::Return(None) => "return".to_string(),
        Terminator::Branch {
            condition,
            then_block,
            else_block,
        } => format!(
            "branch %{} -> bb{}, bb{}",
            condition.index(),
            then_block.index(),
            else_block.index()
        ),
        Terminator::Jump(target) => format!("jump bb{}", target.index()),
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

fn format_type(ty: &Type) -> String {
    match ty {
        Type::Unit => "Unit".to_string(),
        Type::Int => "Int".to_string(),
        Type::Float => "Float".to_string(),
        Type::Bool => "Bool".to_string(),
        Type::String => "String".to_string(),
        Type::Named(name) => name.clone(),
        Type::Unknown => "_".to_string(),
    }
}
