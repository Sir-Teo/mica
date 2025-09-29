use crate::syntax::ast::*;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct Resolved {
    pub module_path: Vec<String>,
    pub adts: HashMap<String, Vec<String>>, // ADT name -> variant names
    pub variant_to_adt: HashMap<String, Vec<String>>, // Variant name -> candidate ADT names
}

pub fn resolve_module(m: &Module) -> Resolved {
    let mut r = Resolved {
        module_path: m.name.clone(),
        adts: HashMap::new(),
        variant_to_adt: HashMap::new(),
    };

    for item in &m.items {
        if let Item::TypeAlias(ta) = item {
            if let TypeExpr::Sum(vars) = &ta.value {
                let vnames: Vec<String> = vars.iter().map(|v| v.name.clone()).collect();
                r.adts.insert(ta.name.clone(), vnames.clone());
                for vn in vnames {
                    r.variant_to_adt
                        .entry(vn)
                        .or_default()
                        .push(ta.name.clone());
                }
            }
        }
    }
    r
}
