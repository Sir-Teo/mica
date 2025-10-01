use std::collections::HashMap;
use std::sync::Arc;

pub mod analysis;

use crate::lower::{HBlock, HExpr, HFuncRef, HFunction, HItem, HModule, HParam, HStmt, HTypeAlias};
use crate::syntax::ast::{BinaryOp, Literal, Path, TypeExpr};

#[derive(Debug, Clone)]
pub struct Module {
    pub name: Vec<String>,
    pub functions: Vec<Function>,
    pub types: TypeTable,
    pub effects: EffectTable,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<Param>,
    pub ret_type: TypeId,
    pub blocks: Vec<BasicBlock>,
    pub effect_row: Vec<EffectId>,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub ty: TypeId,
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
    pub ty: TypeId,
    pub effects: Vec<EffectId>,
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
    Phi {
        incomings: Vec<(BlockId, ValueId)>,
    },
}

#[derive(Debug, Clone)]
pub enum Terminator {
    Return(Option<ValueId>),
    Branch {
        condition: ValueId,
        then_block: BlockId,
        else_block: BlockId,
    },
    Jump(BlockId),
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct TypeId(u32);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct EffectId(u32);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RecordField {
    pub name: String,
    pub ty: TypeId,
    pub offset: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RecordType {
    pub name: Option<String>,
    pub fields: Vec<RecordField>,
    pub size: u32,
    pub align: u32,
}

impl RecordType {
    pub fn field(&self, name: &str) -> Option<&RecordField> {
        self.fields.iter().find(|field| field.name == name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Unit,
    Int,
    Float,
    Bool,
    String,
    Named(String),
    Record(RecordType),
    Unknown,
}

#[derive(Debug, Clone)]
pub struct TypeTable {
    inner: Arc<TypeTableInner>,
}

#[derive(Debug, Clone)]
struct TypeTableInner {
    entries: Vec<Type>,
    index: HashMap<Type, TypeId>,
    named: HashMap<String, TypeId>,
    unknown: TypeId,
}

#[derive(Debug, Clone, Default)]
pub struct EffectTable {
    inner: Arc<EffectTableInner>,
}

#[derive(Debug, Clone, Default)]
struct EffectTableInner {
    entries: Vec<String>,
    index: HashMap<String, EffectId>,
}

pub fn lower_module(module: &HModule) -> Module {
    let mut lowerer = ModuleLower::new(module.name.clone());
    for item in &module.items {
        if let HItem::TypeAlias(alias) = item {
            lowerer.push_type_alias(alias);
        }
    }
    for func in module.items.iter().filter_map(|item| match item {
        HItem::Function(func) => Some(func),
        HItem::TypeAlias(_) => None,
    }) {
        lowerer.push_function(func);
    }
    lowerer.finish()
}

#[derive(Debug, Clone)]
struct FunctionSignature {
    ret_type: TypeId,
    effects: Vec<EffectId>,
}

struct ModuleLower {
    name: Vec<String>,
    functions: Vec<Function>,
    types: TypeTable,
    effects: EffectTable,
    function_signatures: HashMap<String, FunctionSignature>,
}

impl ModuleLower {
    fn new(name: Vec<String>) -> Self {
        ModuleLower {
            name,
            functions: Vec::new(),
            types: TypeTable::new(),
            effects: EffectTable::default(),
            function_signatures: HashMap::new(),
        }
    }

    fn push_type_alias(&mut self, alias: &HTypeAlias) {
        if !alias.params.is_empty() {
            return;
        }
        match &alias.value {
            TypeExpr::Record(fields) => {
                self.types.intern_record(Some(&alias.name), fields);
            }
            other => {
                let ty = self.types.intern_type_expr(other);
                self.types.define_alias(&alias.name, ty);
            }
        }
    }

    fn push_function(&mut self, func: &HFunction) {
        let declared_ret_type = func
            .return_type
            .as_ref()
            .map(|ty| self.types.intern_type_expr(ty));
        let ret_type = declared_ret_type.unwrap_or_else(|| self.types.unknown());
        let effect_row: Vec<_> = func
            .effect_row
            .iter()
            .map(|name| self.effects.intern(name.clone()))
            .collect();
        self.function_signatures.insert(
            func.name.clone(),
            FunctionSignature {
                ret_type,
                effects: effect_row.clone(),
            },
        );
        let mut lowerer = FunctionLower::new(
            func.name.clone(),
            declared_ret_type,
            effect_row,
            &mut self.types,
            self.function_signatures.clone(),
        );
        for param in &func.params {
            lowerer.push_param(param);
        }
        lowerer.lower_block(&func.body);
        let lowered = lowerer.finish();
        if let Some(signature) = self.function_signatures.get_mut(&func.name) {
            signature.ret_type = lowered.ret_type;
            signature.effects = lowered.effect_row.clone();
        }
        self.functions.push(lowered);
    }

    fn finish(self) -> Module {
        Module {
            name: self.name,
            functions: self.functions,
            types: self.types,
            effects: self.effects,
        }
    }
}

struct FunctionLower<'a> {
    name: String,
    params: Vec<Param>,
    next_value: u32,
    next_block: u32,
    current_block: BlockBuilder,
    blocks: Vec<BasicBlock>,
    scopes: Vec<HashMap<String, ValueId>>,
    value_types: HashMap<ValueId, TypeId>,
    ret_type: TypeId,
    effect_row: Vec<EffectId>,
    types: &'a mut TypeTable,
    unknown: TypeId,
    functions: HashMap<String, FunctionSignature>,
}

impl<'a> FunctionLower<'a> {
    fn new(
        name: String,
        ret_type: Option<TypeId>,
        effect_row: Vec<EffectId>,
        types: &'a mut TypeTable,
        functions: HashMap<String, FunctionSignature>,
    ) -> Self {
        let entry = BlockBuilder::new(BlockId(0));
        let unknown = types.unknown();
        FunctionLower {
            name,
            params: Vec::new(),
            next_value: 0,
            next_block: 1,
            current_block: entry,
            blocks: Vec::new(),
            scopes: vec![HashMap::new()],
            value_types: HashMap::new(),
            ret_type: ret_type.unwrap_or(unknown),
            effect_row,
            types,
            unknown,
            functions,
        }
    }

    fn finish(mut self) -> Function {
        if self.current_block.terminator.is_none() {
            let (unit, unit_ty) = self.emit_literal(Literal::Unit);
            self.current_block
                .set_terminator(Terminator::Return(Some(unit)));
            self.merge_return_type(unit_ty);
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
        let ty = self.types.intern_type_expr(&param.ty);
        self.value_types.insert(id, ty);
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
            self.merge_return_type(*ty);
        } else {
            let unit = self.types.intern(Type::Unit);
            self.merge_return_type(unit);
        }
        self.current_block
            .set_terminator(Terminator::Return(value_id));
    }

    fn lower_expr(&mut self, expr: &HExpr) -> (ValueId, TypeId) {
        match expr {
            HExpr::Literal(lit) => self.emit_literal(lit.clone()),
            HExpr::Var(name) => {
                let id = self
                    .lookup(name)
                    .unwrap_or_else(|| panic!("unknown variable {name}"));
                let ty = self.value_types.get(&id).copied().unwrap_or(self.unknown);
                (id, ty)
            }
            HExpr::Path(path) => {
                if path.segments.len() == 1 {
                    if let Some(id) = self.lookup(&path.segments[0]) {
                        let ty = self.value_types.get(&id).copied().unwrap_or(self.unknown);
                        return (id, ty);
                    }
                }
                self.emit_instruction(InstKind::Path(path.clone()), self.unknown, Vec::new())
            }
            HExpr::Call { func, args } => {
                if let HFuncRef::Method(name) = func {
                    if name == "if" {
                        return self.lower_if_call(args);
                    }
                }
                let mut lowered_args = Vec::with_capacity(args.len());
                for arg in args {
                    let (id, _) = self.lower_expr(arg);
                    lowered_args.push(id);
                }
                let func_ref = match func {
                    HFuncRef::Function(path) => FuncRef::Function(path.clone()),
                    HFuncRef::Method(name) => FuncRef::Method(name.clone()),
                };
                let effects = self.lookup_effects(&func_ref);
                let ret_ty = self.lookup_return_type(&func_ref);
                self.emit_instruction(
                    InstKind::Call {
                        func: func_ref,
                        args: lowered_args,
                    },
                    ret_ty,
                    effects,
                )
            }
            HExpr::Binary { lhs, op, rhs } => {
                let (lhs_id, lhs_ty) = self.lower_expr(lhs);
                let (rhs_id, rhs_ty) = self.lower_expr(rhs);
                let ty = if lhs_ty != self.unknown && lhs_ty == rhs_ty {
                    lhs_ty
                } else {
                    self.unknown
                };
                self.emit_instruction(
                    InstKind::Binary {
                        op: *op,
                        lhs: lhs_id,
                        rhs: rhs_id,
                    },
                    ty,
                    Vec::new(),
                )
            }
            HExpr::Block(block) => self.lower_block_expr(block),
            HExpr::Record { type_path, fields } => {
                let mut lowered_fields = Vec::with_capacity(fields.len());
                for (name, value) in fields {
                    let (id, _) = self.lower_expr(value);
                    lowered_fields.push((name.clone(), id));
                }
                let ty = type_path
                    .as_ref()
                    .and_then(|path| self.lookup_type(path))
                    .unwrap_or(self.unknown);
                let reordered = self.reorder_record_fields(ty, lowered_fields);
                self.emit_instruction(
                    InstKind::Record {
                        type_path: type_path.clone(),
                        fields: reordered,
                    },
                    ty,
                    Vec::new(),
                )
            }
        }
    }

    fn lower_block_expr(&mut self, block: &HBlock) -> (ValueId, TypeId) {
        let mut result: Option<(ValueId, TypeId)> = None;
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

    fn emit_literal(&mut self, literal: Literal) -> (ValueId, TypeId) {
        let ty = self.types.intern(Type::from_literal(&literal));
        self.emit_instruction(InstKind::Literal(literal), ty, Vec::new())
    }

    fn emit_instruction(
        &mut self,
        kind: InstKind,
        ty: TypeId,
        effects: Vec<EffectId>,
    ) -> (ValueId, TypeId) {
        if self.current_block.terminator.is_some() {
            panic!("attempted to emit instruction after block was terminated");
        }
        let id = self.alloc_value();
        self.value_types.insert(id, ty);
        let instr = Instruction {
            id,
            ty,
            effects,
            kind,
        };
        self.current_block.push_instruction(instr);
        (id, ty)
    }

    fn lower_if_call(&mut self, args: &[HExpr]) -> (ValueId, TypeId) {
        if args.len() < 2 {
            panic!("if call expected at least condition and then branch");
        }
        let condition = &args[0];
        let then_branch = &args[1];
        let else_branch = args.get(2);

        let (cond_value, _) = self.lower_expr(condition);

        let then_block = self.alloc_block();
        let then_block_id = then_block.id();
        let else_block = self.alloc_block();
        let else_block_id = else_block.id();
        let merge_block = self.alloc_block();
        let merge_block_id = merge_block.id();

        self.current_block.set_terminator(Terminator::Branch {
            condition: cond_value,
            then_block: then_block_id,
            else_block: else_block_id,
        });

        let previous = self.switch_block(then_block);
        self.blocks.push(previous.finish());

        let (then_value, then_ty) = self.with_scope(|this| this.lower_expr(then_branch));
        if !self.current_block.has_terminator() {
            self.current_block
                .set_terminator(Terminator::Jump(merge_block_id));
        }
        let previous_then = self.switch_block(else_block);
        self.blocks.push(previous_then.finish());

        let (else_value, else_ty) = if let Some(expr) = else_branch {
            self.with_scope(|this| this.lower_expr(expr))
        } else {
            self.emit_literal(Literal::Unit)
        };
        if !self.current_block.has_terminator() {
            self.current_block
                .set_terminator(Terminator::Jump(merge_block_id));
        }
        let previous_else = self.switch_block(merge_block);
        self.blocks.push(previous_else.finish());

        let ty = self.join_types(then_ty, else_ty);
        let (phi_value, phi_ty) = self.emit_instruction(
            InstKind::Phi {
                incomings: vec![(then_block_id, then_value), (else_block_id, else_value)],
            },
            ty,
            Vec::new(),
        );
        (phi_value, phi_ty)
    }

    fn define(&mut self, name: String, value: ValueId, ty: TypeId) {
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

    fn lookup_effects(&self, func: &FuncRef) -> Vec<EffectId> {
        match func {
            FuncRef::Function(path) if path.segments.len() == 1 => self
                .functions
                .get(&path.segments[0])
                .map(|sig| sig.effects.clone())
                .unwrap_or_default(),
            _ => Vec::new(),
        }
    }

    fn lookup_return_type(&self, func: &FuncRef) -> TypeId {
        match func {
            FuncRef::Function(path) if path.segments.len() == 1 => self
                .functions
                .get(&path.segments[0])
                .map(|sig| sig.ret_type)
                .unwrap_or(self.unknown),
            _ => self.unknown,
        }
    }

    fn lookup_type(&mut self, path: &Path) -> Option<TypeId> {
        if path.segments.len() == 1 {
            self.types.lookup_named(&path.segments[0])
        } else {
            None
        }
    }

    fn reorder_record_fields(
        &mut self,
        ty: TypeId,
        values: Vec<(String, ValueId)>,
    ) -> Vec<(String, ValueId)> {
        match self.types.get(ty) {
            Type::Record(record) => record
                .fields
                .iter()
                .filter_map(|field| {
                    values
                        .iter()
                        .find(|(name, _)| name == &field.name)
                        .map(|(_, value)| (field.name.clone(), *value))
                })
                .collect(),
            _ => values,
        }
    }

    fn with_scope<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        self.scopes.push(HashMap::new());
        let result = f(self);
        self.scopes.pop();
        result
    }

    fn merge_return_type(&mut self, ty: TypeId) {
        if self.ret_type == self.unknown {
            self.ret_type = ty;
            return;
        }
        if ty == self.unknown {
            return;
        }
        let existing = self.types.get(self.ret_type).clone();
        let new = self.types.get(ty).clone();
        if existing != new {
            self.ret_type = self.unknown;
        }
    }

    fn alloc_value(&mut self) -> ValueId {
        let id = ValueId(self.next_value);
        self.next_value += 1;
        id
    }

    fn alloc_block(&mut self) -> BlockBuilder {
        let id = BlockId(self.next_block);
        self.next_block += 1;
        BlockBuilder::new(id)
    }

    fn switch_block(&mut self, mut next: BlockBuilder) -> BlockBuilder {
        std::mem::swap(&mut self.current_block, &mut next);
        next
    }

    fn join_types(&mut self, lhs: TypeId, rhs: TypeId) -> TypeId {
        if lhs == rhs {
            return lhs;
        }
        if lhs == self.unknown {
            return rhs;
        }
        if rhs == self.unknown {
            return lhs;
        }
        let lty = self.types.get(lhs).clone();
        let rty = self.types.get(rhs).clone();
        if lty == rty { lhs } else { self.unknown }
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

    fn id(&self) -> BlockId {
        self.id
    }

    fn push_instruction(&mut self, inst: Instruction) {
        self.instructions.push(inst);
    }

    fn set_terminator(&mut self, terminator: Terminator) {
        self.terminator = Some(terminator);
    }

    fn has_terminator(&self) -> bool {
        self.terminator.is_some()
    }

    fn finish(self) -> BasicBlock {
        BasicBlock {
            id: self.id,
            instructions: self.instructions,
            terminator: self.terminator.unwrap_or(Terminator::Return(None)),
        }
    }
}

impl Module {
    pub fn effect_name(&self, id: EffectId) -> &str {
        self.effects.name(id)
    }

    pub fn type_of(&self, id: TypeId) -> &Type {
        self.types.get(id)
    }

    pub fn unknown_type(&self) -> TypeId {
        self.types.unknown()
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

impl TypeId {
    pub fn index(self) -> usize {
        self.0 as usize
    }
}

impl EffectId {
    pub fn index(self) -> usize {
        self.0 as usize
    }
}

impl TypeTable {
    pub fn new() -> Self {
        let mut inner = TypeTableInner {
            entries: Vec::new(),
            index: HashMap::new(),
            named: HashMap::new(),
            unknown: TypeId(0),
        };
        let unknown = inner.insert_raw(Type::Unknown);
        inner.unknown = unknown;
        for (name, builtin) in [
            ("Unit", Type::Unit),
            ("Int", Type::Int),
            ("Float", Type::Float),
            ("Bool", Type::Bool),
            ("String", Type::String),
        ] {
            let id = inner.insert_raw(builtin);
            inner.named.insert(name.to_string(), id);
        }
        TypeTable {
            inner: Arc::new(inner),
        }
    }

    pub fn intern(&mut self, ty: Type) -> TypeId {
        if let Some(id) = self.inner.index.get(&ty) {
            *id
        } else {
            Arc::make_mut(&mut self.inner).insert_raw(ty)
        }
    }

    pub fn intern_type_expr(&mut self, expr: &TypeExpr) -> TypeId {
        match expr {
            TypeExpr::Unit => self.intern(Type::Unit),
            TypeExpr::Name(name) | TypeExpr::Generic(name, _) => {
                if let Some(id) = self.lookup_named(name) {
                    id
                } else {
                    self.intern(Type::from_builtin_name(name))
                }
            }
            TypeExpr::Record(fields) => self.intern_record(None, fields),
            TypeExpr::Tuple(items) => {
                if items.is_empty() {
                    self.intern(Type::Unit)
                } else {
                    self.intern(Type::Unknown)
                }
            }
            TypeExpr::Function { return_type, .. } => self.intern_type_expr(return_type),
            TypeExpr::List(inner) => self.intern_type_expr(inner),
            TypeExpr::Reference { .. } | TypeExpr::Sum(_) | TypeExpr::SelfType => {
                self.intern(Type::Unknown)
            }
        }
    }

    pub fn intern_record(&mut self, name: Option<&str>, fields: &[(String, TypeExpr)]) -> TypeId {
        let mut layout = Vec::with_capacity(fields.len());
        let mut offset = 0u32;
        let mut align = 1u32;
        for (field_name, field_ty) in fields {
            let field_ty = self.intern_type_expr(field_ty);
            let field_align = self.align_of(field_ty);
            let field_size = self.size_of(field_ty);
            offset = align_to(offset, field_align);
            layout.push(RecordField {
                name: field_name.clone(),
                ty: field_ty,
                offset,
            });
            offset = offset.saturating_add(field_size);
            align = align.max(field_align);
        }
        align = align.max(1);
        let size = align_to(offset, align);
        let record = RecordType {
            name: name.map(|n| n.to_string()),
            fields: layout,
            size,
            align,
        };
        self.intern(Type::Record(record))
    }

    pub fn define_alias(&mut self, name: &str, ty: TypeId) {
        Arc::make_mut(&mut self.inner)
            .named
            .insert(name.to_string(), ty);
    }

    pub fn get(&self, id: TypeId) -> &Type {
        &self.inner.entries[id.index()]
    }

    pub fn entries(&self) -> impl Iterator<Item = (TypeId, &Type)> {
        self.inner
            .entries
            .iter()
            .enumerate()
            .map(|(index, ty)| (TypeId(index as u32), ty))
    }

    pub fn lookup_named(&self, name: &str) -> Option<TypeId> {
        self.inner.named.get(name).copied()
    }

    pub fn unknown(&self) -> TypeId {
        self.inner.unknown
    }

    pub fn size_of(&self, ty: TypeId) -> u32 {
        match self.get(ty) {
            Type::Unit => 0,
            Type::Bool => 1,
            Type::Int | Type::Float => 8,
            Type::String | Type::Named(_) | Type::Unknown => 8,
            Type::Record(record) => record.size,
        }
    }

    pub fn align_of(&self, ty: TypeId) -> u32 {
        match self.get(ty) {
            Type::Unit => 1,
            Type::Bool => 1,
            Type::Int | Type::Float | Type::String | Type::Named(_) | Type::Unknown => 8,
            Type::Record(record) => record.align,
        }
    }
}

impl TypeTableInner {
    fn insert_raw(&mut self, ty: Type) -> TypeId {
        if let Some(id) = self.index.get(&ty) {
            return *id;
        }
        let id = TypeId(self.entries.len() as u32);
        match &ty {
            Type::Named(name) => {
                self.named.insert(name.clone(), id);
            }
            Type::Record(record) => {
                if let Some(name) = &record.name {
                    self.named.insert(name.clone(), id);
                }
            }
            _ => {}
        }
        self.index.insert(ty.clone(), id);
        self.entries.push(ty);
        id
    }
}

impl EffectTable {
    pub fn intern(&mut self, name: String) -> EffectId {
        if let Some(id) = self.inner.index.get(&name) {
            *id
        } else {
            Arc::make_mut(&mut self.inner).insert(name)
        }
    }

    pub fn entries(&self) -> impl Iterator<Item = (EffectId, &String)> {
        self.inner
            .entries
            .iter()
            .enumerate()
            .map(|(index, name)| (EffectId(index as u32), name))
    }

    pub fn name(&self, id: EffectId) -> &str {
        &self.inner.entries[id.index()]
    }
}

impl EffectTableInner {
    fn insert(&mut self, name: String) -> EffectId {
        if let Some(id) = self.index.get(&name) {
            return *id;
        }
        let id = EffectId(self.entries.len() as u32);
        self.entries.push(name.clone());
        self.index.insert(name, id);
        id
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

fn align_to(value: u32, align: u32) -> u32 {
    if align <= 1 {
        value
    } else {
        ((value + align - 1) / align) * align
    }
}
