use crate::syntax::ast::{Function, Item, Module, TypeAlias, TypeExpr, UseDecl};

use super::data::{
    ModuleExports, PathKind, Resolved, ResolvedImport, SymbolCategory, SymbolInfo, SymbolScope,
};
use super::scope::ScopeLayer;

pub(super) struct ModuleSymbols {
    pub(super) module_scope: ScopeLayer,
    pub(super) resolved: Resolved,
    pub(super) exports: ModuleExports,
}

pub(super) fn collect_module(module: &Module) -> ModuleSymbols {
    let mut collector = Collector::new(module);
    collector.collect();
    ModuleSymbols {
        module_scope: collector.module_scope,
        resolved: collector.resolved,
        exports: collector.exports,
    }
}

struct Collector<'a> {
    module: &'a Module,
    module_scope: ScopeLayer,
    resolved: Resolved,
    exports: ModuleExports,
}

impl<'a> Collector<'a> {
    fn new(module: &'a Module) -> Self {
        Self {
            module,
            module_scope: ScopeLayer::default(),
            resolved: Resolved {
                module_path: module.name.clone(),
                ..Resolved::default()
            },
            exports: ModuleExports::default(),
        }
    }

    fn collect(&mut self) {
        for item in &self.module.items {
            match item {
                Item::TypeAlias(ta) => self.collect_type_alias(ta),
                Item::Function(func) => self.collect_function(func),
                Item::Use(import) => self.collect_use(import),
                Item::Impl(_) => {}
            }
        }
    }

    fn collect_type_alias(&mut self, ta: &TypeAlias) {
        let symbol = SymbolInfo {
            name: ta.name.clone(),
            category: SymbolCategory::Type {
                is_public: ta.is_public,
                params: ta.params.clone(),
            },
            scope: SymbolScope::Module(self.module.name.clone()),
        };
        if let Some(existing) = self.module_scope.insert_type(symbol.clone()) {
            self.resolved
                .diagnostics
                .push(super::data::ResolveDiagnostic {
                    path: vec![ta.name.clone()],
                    kind: PathKind::Type,
                    scope: SymbolScope::Module(self.module.name.clone()),
                    message: format!("duplicate type definition for '{}'", existing.name),
                });
        }
        self.resolved.symbols.push(symbol.clone());
        if ta.is_public {
            self.exports
                .types
                .insert(symbol.name.clone(), symbol.clone());
        }

        if let TypeExpr::Sum(variants) = &ta.value {
            let mut names = Vec::new();
            for variant in variants {
                names.push(variant.name.clone());
                self.resolved
                    .variant_to_adt
                    .entry(variant.name.clone())
                    .or_default()
                    .push(ta.name.clone());
                let variant_symbol = SymbolInfo {
                    name: variant.name.clone(),
                    category: SymbolCategory::Variant {
                        parent: ta.name.clone(),
                    },
                    scope: SymbolScope::Module(self.module.name.clone()),
                };
                self.resolved.symbols.push(variant_symbol.clone());
                if ta.is_public {
                    self.exports
                        .variants
                        .insert(variant_symbol.name.clone(), variant_symbol);
                }
            }
            self.resolved.adts.insert(ta.name.clone(), names);
        }
    }

    fn collect_function(&mut self, func: &Function) {
        let symbol = SymbolInfo {
            name: func.name.clone(),
            category: SymbolCategory::Function {
                is_public: func.is_public,
            },
            scope: SymbolScope::Module(self.module.name.clone()),
        };
        if let Some(existing) = self.module_scope.insert_value(symbol.clone()) {
            self.resolved
                .diagnostics
                .push(super::data::ResolveDiagnostic {
                    path: vec![func.name.clone()],
                    kind: PathKind::Value,
                    scope: SymbolScope::Module(self.module.name.clone()),
                    message: format!("duplicate function definition for '{}'", existing.name),
                });
        }
        self.resolved.symbols.push(symbol.clone());
        if func.is_public {
            self.exports.values.insert(symbol.name.clone(), symbol);
        }
    }

    fn collect_use(&mut self, import: &UseDecl) {
        let alias = import.alias.clone().or_else(|| import.path.last().cloned());
        self.resolved.imports.push(ResolvedImport {
            path: import.path.clone(),
            alias: alias.clone(),
        });

        if let Some(name) = alias {
            let symbol = SymbolInfo {
                name: name.clone(),
                category: SymbolCategory::ImportAlias {
                    target: import.path.clone(),
                },
                scope: SymbolScope::Module(self.module.name.clone()),
            };
            if let Some(existing) = self.module_scope.insert_value(symbol.clone()) {
                self.resolved
                    .diagnostics
                    .push(super::data::ResolveDiagnostic {
                        path: vec![name.clone()],
                        kind: PathKind::Value,
                        scope: SymbolScope::Module(self.module.name.clone()),
                        message: format!("duplicate import alias '{}'", existing.name),
                    });
            }
            if let Some(existing) = self.module_scope.insert_type(symbol.clone()) {
                self.resolved
                    .diagnostics
                    .push(super::data::ResolveDiagnostic {
                        path: vec![name.clone()],
                        kind: PathKind::Type,
                        scope: SymbolScope::Module(self.module.name.clone()),
                        message: format!("duplicate import alias '{}'", existing.name),
                    });
            }
            self.resolved.symbols.push(symbol);
        }
    }
}
