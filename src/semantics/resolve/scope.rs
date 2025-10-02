use std::collections::HashMap;

use super::data::{Resolved, SymbolCategory, SymbolInfo, SymbolScope};

#[derive(Clone, Default)]
pub(super) struct ScopeLayer {
    values: HashMap<String, SymbolInfo>,
    types: HashMap<String, SymbolInfo>,
}

impl ScopeLayer {
    pub(super) fn insert_value(&mut self, symbol: SymbolInfo) -> Option<SymbolInfo> {
        self.values.insert(symbol.name.clone(), symbol)
    }

    pub(super) fn insert_type(&mut self, symbol: SymbolInfo) -> Option<SymbolInfo> {
        self.types.insert(symbol.name.clone(), symbol)
    }
}

pub(super) struct ScopeStack {
    stack: Vec<ScopeLayer>,
}

impl ScopeStack {
    pub(super) fn new(root: ScopeLayer) -> Self {
        Self { stack: vec![root] }
    }

    pub(super) fn push_layer(&mut self) {
        self.stack.push(ScopeLayer::default());
    }

    pub(super) fn pop_layer(&mut self) {
        self.stack.pop();
    }

    pub(super) fn insert_value(&mut self, symbol: SymbolInfo) -> Option<SymbolInfo> {
        if let Some(layer) = self.stack.last_mut() {
            layer.insert_value(symbol)
        } else {
            None
        }
    }

    pub(super) fn insert_type(&mut self, symbol: SymbolInfo) -> Option<SymbolInfo> {
        if let Some(layer) = self.stack.last_mut() {
            layer.insert_type(symbol)
        } else {
            None
        }
    }

    pub(super) fn lookup_value(
        &self,
        segments: &[String],
        resolved: &Resolved,
    ) -> Option<SymbolInfo> {
        if segments.is_empty() {
            return None;
        }

        if segments.len() > 1
            && let Some(info) = lookup_variant_from_segments(segments, resolved)
        {
            return Some(info);
        }

        for layer in self.stack.iter().rev() {
            if let Some(symbol) = layer.values.get(&segments[0]) {
                return Some(symbol.clone());
            }
        }

        if segments.len() == 1 {
            lookup_variant_from_segments(segments, resolved)
        } else {
            None
        }
    }

    pub(super) fn lookup_type(
        &self,
        segments: &[String],
        resolved: &Resolved,
    ) -> Option<SymbolInfo> {
        if segments.is_empty() {
            return None;
        }

        for layer in self.stack.iter().rev() {
            if let Some(symbol) = layer.types.get(&segments[0]) {
                return Some(symbol.clone());
            }
        }

        if segments.len() == 2 {
            // Handle Type::Variant references appearing in type positions.
            lookup_variant_from_segments(segments, resolved)
        } else {
            None
        }
    }

    pub(super) fn lookup_variant(
        &self,
        segments: &[String],
        resolved: &Resolved,
    ) -> Option<SymbolInfo> {
        lookup_variant_from_segments(segments, resolved)
    }
}

pub(super) fn lookup_variant_from_segments(
    segments: &[String],
    resolved: &Resolved,
) -> Option<SymbolInfo> {
    if segments.is_empty() {
        return None;
    }

    let (ty_name, variant_name) = if segments.len() == 1 {
        (None, segments[0].clone())
    } else {
        (
            segments.get(segments.len() - 2).cloned(),
            segments.last().cloned().unwrap(),
        )
    };

    let parents = resolved.variant_to_adt.get(&variant_name)?;
    let parent = if let Some(name) = ty_name {
        parents
            .iter()
            .find(|candidate| *candidate == &name)
            .cloned()
            .or_else(|| parents.first().cloned())
    } else {
        parents.first().cloned()
    }?;

    Some(SymbolInfo {
        name: variant_name,
        category: SymbolCategory::Variant { parent },
        scope: SymbolScope::Module(resolved.module_path.clone()),
    })
}
