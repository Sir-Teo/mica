use std::collections::HashMap;

use crate::syntax::ast::Module;

use super::collector::{ModuleSymbols, collect_module};
use super::data::{ModuleExports, PathKind, SymbolInfo};

#[derive(Default)]
pub(super) struct ModuleGraph {
    exports: HashMap<Vec<String>, ModuleExports>,
}

impl ModuleGraph {
    pub(super) fn new() -> Self {
        Self {
            exports: HashMap::new(),
        }
    }

    pub(super) fn add_module(&mut self, module: &Module) -> ModuleSymbols {
        let symbols = collect_module(module);
        self.exports
            .insert(module.name.clone(), symbols.exports.clone());
        symbols
    }

    pub(super) fn lookup(&self, segments: &[String], kind: PathKind) -> Option<SymbolInfo> {
        if segments.is_empty() {
            return None;
        }

        for module_len in (1..segments.len()).rev() {
            let module_path: Vec<String> = segments[..module_len].to_vec();
            let Some(exports) = self.exports.get(&module_path) else {
                continue;
            };
            let remainder = &segments[module_len..];
            if remainder.is_empty() {
                continue;
            }

            match kind {
                PathKind::Type => {
                    if remainder.len() == 1 {
                        if let Some(symbol) = exports.types.get(&remainder[0]) {
                            return Some(symbol.clone());
                        }
                    } else if remainder.len() == 2 {
                        if let Some(symbol) = exports.variants.get(&remainder[1]) {
                            return Some(symbol.clone());
                        }
                    }
                }
                PathKind::Value => {
                    if remainder.len() == 1 {
                        if let Some(symbol) = exports.values.get(&remainder[0]) {
                            return Some(symbol.clone());
                        }
                        if let Some(symbol) = exports.variants.get(&remainder[0]) {
                            return Some(symbol.clone());
                        }
                    } else if let Some(symbol) = exports.variants.get(remainder.last().unwrap()) {
                        return Some(symbol.clone());
                    }
                }
                PathKind::Variant => {
                    if let Some(symbol) = exports.variants.get(remainder.last().unwrap()) {
                        return Some(symbol.clone());
                    }
                }
            }
        }

        None
    }
}
