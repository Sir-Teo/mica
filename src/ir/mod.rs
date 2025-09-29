use std::collections::HashMap;

use crate::lower::{HBlock, HExpr, HFuncRef, HFunction, HItem, HModule, HParam, HStmt};
use crate::syntax::ast::{BinaryOp, Literal, Path, TypeExpr};

#[derive(Debug, Clone)]
pub struct Module {
    pub name: Vec<String>,
    pub functions: Vec<Function>,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<Param>,
    pub ret_type: Type,
    pub blocks: Vec<BasicBlock>,
    pub effect_row: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub ty: Type,
    pub value: ValueId,
}

#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub id: BlockId,
    pub instructions: Vec<Instruction>,
    pub terminator: Terminator,
}

#[derive(Debug, Clone)]
pub struct Instruction {
    pub id: ValueId,
    pub ty: Type,
    pub kind: InstKind,
}

#[derive(Debug, Clone)]
pub enum InstKind {
    Literal(Literal),
    Binary {
        op: BinaryOp,
        lhs: ValueId,
        rhs: ValueId,
    },
    Call {
        func: FuncRef,
        args: Vec<ValueId>,
    },
    Record {
        type_path: Option<Path>,
        fields: Vec<(String, ValueId)>,
    },
    Path(Path),
}

#[derive(Debug, Clone)]
pub enum Terminator {
    Return(Option<ValueId>),
}

#[derive(Debug, Clone)]
pub enum FuncRef {
    Function(Path),
    Method(String),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ValueId(u32);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct BlockId(u32);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Unit,
    Int,
    Float,
    Bool,
    String,
    Named(String),
    Unknown,
}

pub fn lower_module(module: &HModule) -> Module {
    let functions = module
        .items
        .iter()
        .filter_map(|item| match item {
            HItem::Function(func) => Some(lower_function(func)),
        })
        .collect();
    Module {
        name: module.name.clone(),
        functions,
    }
}

fn lower_function(func: &HFunction) -> Function {
    let mut lowerer = FunctionLower::new(
        func.name.clone(),
        func.return_type.as_ref().map(Type::from_type_expr),
        func.effect_row.clone(),
    );
    for param in &func.params {
        lowerer.push_param(param);
    }
    lowerer.lower_block(&func.body);
    lowerer.finish()
}

struct FunctionLower {
    name: String,
    params: Vec<Param>,
    next_value: u32,
    current_block: BlockBuilder,
    blocks: Vec<BasicBlock>,
    scopes: Vec<HashMap<String, ValueId>>,
    value_types: HashMap<ValueId, Type>,
    ret_type: Type,
    effect_row: Vec<String>,
}

impl FunctionLower {
    fn new(name: String, ret_type: Option<Type>, effect_row: Vec<String>) -> Self {
        let entry = BlockBuilder::new(BlockId(0));
        FunctionLower {
            name,
            params: Vec::new(),
            next_value: 0,
            current_block: entry,
            blocks: Vec::new(),
            scopes: vec![HashMap::new()],
            value_types: HashMap::new(),
            ret_type: ret_type.unwrap_or(Type::Unknown),
            effect_row,
        }
    }

    fn finish(mut self) -> Function {
        if self.current_block.terminator.is_none() {
            let (unit, _) = self.emit_literal(Literal::Unit);
            self.current_block
                .set_terminator(Terminator::Return(Some(unit)));
            self.merge_return_type(Type::Unit);
        }
        self.blocks.push(self.current_block.finish());
        Function {
            name: self.name,
            params: self.params,
            ret_type: self.ret_type,
            blocks: self.blocks,
            effect_row: self.effect_row,
        }
    }

    fn push_param(&mut self, param: &HParam) {
        let id = self.alloc_value();
        let ty = Type::from_type_expr(&param.ty);
        self.value_types.insert(id, ty.clone());
        self.scopes
            .last_mut()
            .expect("scope stack")
            .insert(param.name.clone(), id);
        self.params.push(Param {
            name: param.name.clone(),
            ty,
            value: id,
        });
    }

    fn lower_block(&mut self, block: &HBlock) {
        self.with_scope(|this| {
            for (index, stmt) in block.stmts.iter().enumerate() {
                if this.current_block.terminator.is_some() {
                    break;
                }

                let is_last = index + 1 == block.stmts.len();
                if is_last {
                    if let HStmt::Expr(expr) = stmt {
                        this.lower_return(Some(expr));
                        break;
                    }
                }

                this.lower_stmt(stmt);
            }
        });
    }

    fn lower_stmt(&mut self, stmt: &HStmt) {
        match stmt {
            HStmt::Let { name, value } => {
                let (val, ty) = self.lower_expr(value);
                self.define(name.clone(), val, ty);
            }
            HStmt::Expr(expr) => {
                let _ = self.lower_expr(expr);
            }
            HStmt::Return(expr) => {
                self.lower_return(expr.as_ref());
            }
        }
    }

    fn lower_return(&mut self, expr: Option<&HExpr>) {
        if self.current_block.terminator.is_some() {
            return;
        }
        let value = expr.map(|e| self.lower_expr(e));
        let value_id = value.as_ref().map(|(id, _)| *id);
        if let Some((_, ty)) = &value {
            self.merge_return_type(ty.clone());
        } else {
            self.merge_return_type(Type::Unit);
        }
        self.current_block
            .set_terminator(Terminator::Return(value_id));
    }

    fn lower_expr(&mut self, expr: &HExpr) -> (ValueId, Type) {
        match expr {
            HExpr::Literal(lit) => self.emit_literal(lit.clone()),
            HExpr::Var(name) => {
                let id = self
                    .lookup(name)
                    .unwrap_or_else(|| panic!("unknown variable {name}"));
                let ty = self.value_types.get(&id).cloned().unwrap_or(Type::Unknown);
                (id, ty)
            }
            HExpr::Path(path) => {
                if path.segments.len() == 1 {
                    if let Some(id) = self.lookup(&path.segments[0]) {
                        let ty = self.value_types.get(&id).cloned().unwrap_or(Type::Unknown);
                        return (id, ty);
                    }
                }
                self.emit_instruction(InstKind::Path(path.clone()), Type::Unknown)
            }
            HExpr::Call { func, args } => {
                let mut lowered_args = Vec::with_capacity(args.len());
                for arg in args {
                    let (id, _) = self.lower_expr(arg);
                    lowered_args.push(id);
                }
                let func_ref = match func {
                    HFuncRef::Function(path) => FuncRef::Function(path.clone()),
                    HFuncRef::Method(name) => FuncRef::Method(name.clone()),
                };
                self.emit_instruction(
                    InstKind::Call {
                        func: func_ref,
                        args: lowered_args,
                    },
                    Type::Unknown,
                )
            }
            HExpr::Binary { lhs, op, rhs } => {
                let (lhs_id, lhs_ty) = self.lower_expr(lhs);
                let (rhs_id, rhs_ty) = self.lower_expr(rhs);
                let ty = if lhs_ty == rhs_ty && lhs_ty != Type::Unknown {
                    lhs_ty
                } else {
                    Type::Unknown
                };
                self.emit_instruction(
                    InstKind::Binary {
                        op: *op,
                        lhs: lhs_id,
                        rhs: rhs_id,
                    },
                    ty,
                )
            }
            HExpr::Block(block) => self.lower_block_expr(block),
            HExpr::Record { type_path, fields } => {
                let mut lowered_fields = Vec::with_capacity(fields.len());
                for (name, value) in fields {
                    let (id, _) = self.lower_expr(value);
                    lowered_fields.push((name.clone(), id));
                }
                self.emit_instruction(
                    InstKind::Record {
                        type_path: type_path.clone(),
                        fields: lowered_fields,
                    },
                    Type::Unknown,
                )
            }
        }
    }

    fn lower_block_expr(&mut self, block: &HBlock) -> (ValueId, Type) {
        let mut result: Option<(ValueId, Type)> = None;
        self.with_scope(|this| {
            for stmt in &block.stmts {
                match stmt {
                    HStmt::Let { name, value } => {
                        let (val, ty) = this.lower_expr(value);
                        this.define(name.clone(), val, ty);
                    }
                    HStmt::Expr(expr) => {
                        result = Some(this.lower_expr(expr));
                    }
                    HStmt::Return(expr) => {
                        this.lower_return(expr.as_ref());
                    }
                }
                if this.current_block.terminator.is_some() {
                    break;
                }
            }
        });
        if let Some(res) = result {
            res
        } else {
            self.emit_literal(Literal::Unit)
        }
    }

    fn emit_literal(&mut self, literal: Literal) -> (ValueId, Type) {
        let ty = Type::from_literal(&literal);
        self.emit_instruction(InstKind::Literal(literal), ty)
    }

    fn emit_instruction(&mut self, kind: InstKind, ty: Type) -> (ValueId, Type) {
        if self.current_block.terminator.is_some() {
            panic!("attempted to emit instruction after block was terminated");
        }
        let id = self.alloc_value();
        let stored_ty = ty.clone();
        let instr = Instruction {
            id,
            ty: stored_ty.clone(),
            kind,
        };
        self.value_types.insert(id, stored_ty);
        self.current_block.push_instruction(instr);
        (id, ty)
    }

    fn define(&mut self, name: String, value: ValueId, ty: Type) {
        self.value_types.entry(value).or_insert(ty);
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, value);
        }
    }

    fn lookup(&self, name: &str) -> Option<ValueId> {
        for scope in self.scopes.iter().rev() {
            if let Some(id) = scope.get(name) {
                return Some(*id);
            }
        }
        None
    }

    fn with_scope<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Self),
    {
        self.scopes.push(HashMap::new());
        f(self);
        self.scopes.pop();
    }

    fn merge_return_type(&mut self, ty: Type) {
        match (&self.ret_type, &ty) {
            (Type::Unknown, _) => self.ret_type = ty,
            (_, Type::Unknown) => {}
            (existing, new) if existing == new => {}
            _ => self.ret_type = Type::Unknown,
        }
    }

    fn alloc_value(&mut self) -> ValueId {
        let id = ValueId(self.next_value);
        self.next_value += 1;
        id
    }
}

#[derive(Debug, Clone)]
struct BlockBuilder {
    id: BlockId,
    instructions: Vec<Instruction>,
    terminator: Option<Terminator>,
}

impl BlockBuilder {
    fn new(id: BlockId) -> Self {
        BlockBuilder {
            id,
            instructions: Vec::new(),
            terminator: None,
        }
    }

    fn push_instruction(&mut self, inst: Instruction) {
        self.instructions.push(inst);
    }

    fn set_terminator(&mut self, terminator: Terminator) {
        self.terminator = Some(terminator);
    }

    fn finish(self) -> BasicBlock {
        BasicBlock {
            id: self.id,
            instructions: self.instructions,
            terminator: self.terminator.unwrap_or(Terminator::Return(None)),
        }
    }
}

impl ValueId {
    pub fn index(self) -> u32 {
        self.0
    }
}

impl Default for ValueId {
    fn default() -> Self {
        ValueId(u32::MAX)
    }
}

impl BlockId {
    pub fn index(self) -> u32 {
        self.0
    }
}

impl Type {
    fn from_literal(literal: &Literal) -> Type {
        match literal {
            Literal::Unit => Type::Unit,
            Literal::Int(_) => Type::Int,
            Literal::Float(_) => Type::Float,
            Literal::Bool(_) => Type::Bool,
            Literal::String(_) => Type::String,
        }
    }

    fn from_type_expr(expr: &TypeExpr) -> Type {
        match expr {
            TypeExpr::Unit => Type::Unit,
            TypeExpr::Name(name) => Type::from_builtin_name(name),
            TypeExpr::Generic(name, _) => Type::from_builtin_name(name),
            TypeExpr::Tuple(items) => {
                if items.is_empty() {
                    Type::Unit
                } else {
                    Type::Unknown
                }
            }
            TypeExpr::Function { return_type, .. } => Type::from_type_expr(return_type),
            _ => Type::Unknown,
        }
    }

    fn from_builtin_name(name: &str) -> Type {
        match name {
            "Unit" => Type::Unit,
            "Int" => Type::Int,
            "Float" => Type::Float,
            "Bool" => Type::Bool,
            "String" => Type::String,
            other => Type::Named(other.to_string()),
        }
    }
}
