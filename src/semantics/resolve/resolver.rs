use crate::syntax::ast::*;

use super::data::{
    CapabilityBinding, CapabilityScope, PathKind, Resolved, ResolvedPath, SymbolCategory,
    SymbolInfo, SymbolScope,
};
use super::scope::{ScopeLayer, ScopeStack};

pub(super) struct Resolver<'a> {
    module: &'a Module,
    scope: ScopeStack,
    resolved: Resolved,
    current_scope: SymbolScope,
}

impl<'a> Resolver<'a> {
    pub(super) fn new(module: &'a Module, module_scope: ScopeLayer, resolved: Resolved) -> Self {
        let scope = ScopeStack::new(module_scope);
        Self {
            module,
            scope,
            resolved,
            current_scope: SymbolScope::Module(module.name.clone()),
        }
    }

    pub(super) fn into_resolved(self) -> Resolved {
        self.resolved
    }

    pub(super) fn resolve(&mut self) {
        for item in &self.module.items {
            match item {
                Item::TypeAlias(ta) => self.resolve_type_alias(ta),
                Item::Function(func) => self.resolve_function(func),
                Item::Use(_) => {}
                Item::Impl(impl_block) => self.resolve_impl(impl_block),
            }
        }
    }

    fn resolve_type_alias(&mut self, ta: &TypeAlias) {
        let prev_scope = self.current_scope.clone();
        self.current_scope = SymbolScope::TypeAlias {
            module_path: self.module.name.clone(),
            type_name: ta.name.clone(),
        };

        self.scope.push_layer();
        for param in &ta.params {
            let symbol = SymbolInfo {
                name: param.clone(),
                category: SymbolCategory::TypeParam,
                scope: self.current_scope.clone(),
            };
            self.scope.insert_type(symbol.clone());
            self.resolved.symbols.push(symbol);
        }

        self.resolve_type_expr(&ta.value);
        self.scope.pop_layer();
        self.current_scope = prev_scope;
    }

    fn resolve_function(&mut self, func: &Function) {
        let prev_scope = self.current_scope.clone();
        self.current_scope = SymbolScope::Function {
            module_path: self.module.name.clone(),
            function: func.name.clone(),
        };

        self.scope.push_layer();
        for generic in &func.generics {
            let symbol = SymbolInfo {
                name: generic.name.clone(),
                category: SymbolCategory::TypeParam,
                scope: self.current_scope.clone(),
            };
            self.scope.insert_type(symbol.clone());
            self.resolved.symbols.push(symbol);
            for bound in &generic.bounds {
                self.resolve_path(&bound.segments, PathKind::Type);
            }
        }

        for param in &func.params {
            self.resolve_type_expr(&param.ty);
            let symbol = SymbolInfo {
                name: param.name.clone(),
                category: SymbolCategory::ValueParam,
                scope: self.current_scope.clone(),
            };
            self.scope.insert_value(symbol.clone());
            self.resolved.symbols.push(symbol);
        }

        if let Some(ret) = &func.return_type {
            self.resolve_type_expr(ret);
        }

        if !func.effect_row.is_empty() {
            for cap in &func.effect_row {
                self.resolved.capabilities.push(CapabilityBinding {
                    name: cap.clone(),
                    scope: CapabilityScope::Function {
                        module_path: self.module.name.clone(),
                        function: func.name.clone(),
                    },
                });
            }
        }

        self.resolve_block(&func.body);
        self.scope.pop_layer();
        self.current_scope = prev_scope;
    }

    fn resolve_impl(&mut self, impl_block: &ImplBlock) {
        self.resolve_path(&impl_block.trait_path.segments, PathKind::Type);
        self.resolve_type_expr(&impl_block.for_type);
        for item in &impl_block.items {
            match item {
                ImplItem::Function(func) => self.resolve_function(func),
            }
        }
    }

    fn resolve_block(&mut self, block: &Block) {
        self.scope.push_layer();
        for stmt in &block.statements {
            self.resolve_stmt(stmt);
        }
        self.scope.pop_layer();
    }

    fn resolve_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Let(let_stmt) => {
                self.resolve_expr(&let_stmt.value);
                let symbol = SymbolInfo {
                    name: let_stmt.name.clone(),
                    category: SymbolCategory::LocalBinding,
                    scope: self.current_scope.clone(),
                };
                self.scope.insert_value(symbol.clone());
                self.resolved.symbols.push(symbol);
            }
            Stmt::Expr(expr) => self.resolve_expr(expr),
            Stmt::Return(Some(expr)) => self.resolve_expr(expr),
            Stmt::Return(None) | Stmt::Break | Stmt::Continue => {}
        }
    }

    fn resolve_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Block(block) => self.resolve_block(block),
            Expr::Literal(_) => {}
            Expr::Path(path) => {
                self.resolve_path(&path.segments, PathKind::Value);
            }
            Expr::Binary { lhs, rhs, .. } => {
                self.resolve_expr(lhs);
                self.resolve_expr(rhs);
            }
            Expr::Unary { expr, .. } => self.resolve_expr(expr),
            Expr::Call { callee, args } => {
                self.resolve_expr(callee);
                for arg in args {
                    self.resolve_expr(arg);
                }
            }
            Expr::Ctor { path, args } => {
                self.resolve_path(&path.segments, PathKind::Variant);
                for arg in args {
                    self.resolve_expr(arg);
                }
            }
            Expr::Record { type_path, fields } => {
                if let Some(path) = type_path {
                    self.resolve_path(&path.segments, PathKind::Type);
                }
                for (_, expr) in fields {
                    self.resolve_expr(expr);
                }
            }
            Expr::Field { expr, .. } => self.resolve_expr(expr),
            Expr::Index { expr, index } => {
                self.resolve_expr(expr);
                self.resolve_expr(index);
            }
            Expr::Cast { expr, ty } => {
                self.resolve_expr(expr);
                self.resolve_type_expr(ty);
            }
            Expr::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.resolve_expr(condition);
                self.resolve_expr(then_branch);
                if let Some(expr) = else_branch {
                    self.resolve_expr(expr);
                }
            }
            Expr::Match { scrutinee, arms } => {
                self.resolve_expr(scrutinee);
                for arm in arms {
                    self.scope.push_layer();
                    self.resolve_pattern(&arm.pattern);
                    if let Some(guard) = &arm.guard {
                        self.resolve_expr(guard);
                    }
                    self.resolve_expr(&arm.body);
                    self.scope.pop_layer();
                }
            }
            Expr::For {
                binding,
                iterable,
                body,
            } => {
                self.resolve_expr(iterable);
                self.scope.push_layer();
                let symbol = SymbolInfo {
                    name: binding.clone(),
                    category: SymbolCategory::LocalBinding,
                    scope: self.current_scope.clone(),
                };
                self.scope.insert_value(symbol.clone());
                self.resolved.symbols.push(symbol);
                self.resolve_expr(body);
                self.scope.pop_layer();
            }
            Expr::While { condition, body } => {
                self.resolve_expr(condition);
                self.resolve_expr(body);
            }
            Expr::Loop { body } => self.resolve_expr(body),
            Expr::Assignment { target, value } => {
                self.resolve_expr(target);
                self.resolve_expr(value);
            }
            Expr::Spawn(expr) | Expr::Await(expr) | Expr::Try(expr) => self.resolve_expr(expr),
            Expr::Chan { ty, capacity } => {
                self.resolve_type_expr(ty);
                if let Some(cap) = capacity {
                    self.resolve_expr(cap);
                }
            }
            Expr::Using {
                binding,
                expr,
                body,
            } => {
                self.resolve_expr(expr);
                self.scope.push_layer();
                if let Some(name) = binding {
                    let symbol = SymbolInfo {
                        name: name.clone(),
                        category: SymbolCategory::LocalBinding,
                        scope: self.current_scope.clone(),
                    };
                    self.scope.insert_value(symbol.clone());
                    self.resolved.symbols.push(symbol);
                }
                self.resolve_block(body);
                self.scope.pop_layer();
            }
        }
    }

    fn resolve_pattern(&mut self, pattern: &Pattern) {
        match pattern {
            Pattern::Wildcard | Pattern::Literal(_) => {}
            Pattern::Binding(name) => {
                let symbol = SymbolInfo {
                    name: name.clone(),
                    category: SymbolCategory::LocalBinding,
                    scope: self.current_scope.clone(),
                };
                self.scope.insert_value(symbol.clone());
                self.resolved.symbols.push(symbol);
            }
            Pattern::Tuple(patterns) => {
                for pat in patterns {
                    self.resolve_pattern(pat);
                }
            }
            Pattern::Record(fields) => {
                for (_, pat) in fields {
                    self.resolve_pattern(pat);
                }
            }
            Pattern::EnumVariant { path, fields } => {
                self.resolve_path(&path.segments, PathKind::Variant);
                for pat in fields {
                    self.resolve_pattern(pat);
                }
            }
        }
    }

    fn resolve_type_expr(&mut self, ty: &TypeExpr) {
        match ty {
            TypeExpr::Name(name) => self.resolve_path(&[name.clone()], PathKind::Type),
            TypeExpr::Generic(name, args) => {
                self.resolve_path(&[name.clone()], PathKind::Type);
                for arg in args {
                    self.resolve_type_expr(arg);
                }
            }
            TypeExpr::Record(fields) => {
                for (_, field_ty) in fields {
                    self.resolve_type_expr(field_ty);
                }
            }
            TypeExpr::Sum(variants) => {
                for variant in variants {
                    for field in &variant.fields {
                        self.resolve_type_expr(field);
                    }
                }
            }
            TypeExpr::List(inner) => self.resolve_type_expr(inner),
            TypeExpr::Tuple(items) => {
                for item in items {
                    self.resolve_type_expr(item);
                }
            }
            TypeExpr::Reference { inner, .. } => self.resolve_type_expr(inner),
            TypeExpr::Function {
                params,
                return_type,
                effect_row,
            } => {
                for param in params {
                    self.resolve_type_expr(param);
                }
                self.resolve_type_expr(return_type);
                if !effect_row.is_empty() {
                    if let Some(scope) = self.current_capability_scope() {
                        for cap in effect_row {
                            self.resolved.capabilities.push(CapabilityBinding {
                                name: cap.clone(),
                                scope: scope.clone(),
                            });
                        }
                    }
                }
            }
            TypeExpr::SelfType | TypeExpr::Unit => {}
        }
    }

    fn current_capability_scope(&self) -> Option<CapabilityScope> {
        match &self.current_scope {
            SymbolScope::Function {
                module_path,
                function,
            } => Some(CapabilityScope::Function {
                module_path: module_path.clone(),
                function: function.clone(),
            }),
            SymbolScope::TypeAlias {
                module_path,
                type_name,
            } => Some(CapabilityScope::TypeAlias {
                module_path: module_path.clone(),
                type_name: type_name.clone(),
            }),
            SymbolScope::Module(_) => None,
        }
    }

    fn resolve_path(&mut self, segments: &[String], kind: PathKind) {
        let resolved = match kind {
            PathKind::Type => self.scope.lookup_type(segments, &self.resolved),
            PathKind::Value => self.scope.lookup_value(segments, &self.resolved),
            PathKind::Variant => self.scope.lookup_variant(segments, &self.resolved),
        };
        self.resolved.resolved_paths.push(ResolvedPath {
            segments: segments.to_vec(),
            kind,
            resolved,
        });
    }
}
