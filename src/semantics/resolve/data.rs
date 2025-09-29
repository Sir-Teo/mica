use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct Resolved {
    pub module_path: Vec<String>,
    pub adts: HashMap<String, Vec<String>>, // ADT name -> variant names
    pub variant_to_adt: HashMap<String, Vec<String>>, // Variant name -> candidate ADT names
    pub imports: Vec<ResolvedImport>,
    pub symbols: Vec<SymbolInfo>,
    pub resolved_paths: Vec<ResolvedPath>,
    pub capabilities: Vec<CapabilityBinding>,
}

#[derive(Debug, Clone)]
pub struct ResolvedImport {
    pub path: Vec<String>,
    pub alias: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SymbolInfo {
    pub name: String,
    pub category: SymbolCategory,
    pub scope: SymbolScope,
}

#[derive(Debug, Clone)]
pub enum SymbolCategory {
    Type {
        is_public: bool,
        params: Vec<String>,
    },
    Variant {
        parent: String,
    },
    Function {
        is_public: bool,
    },
    TypeParam,
    ValueParam,
    LocalBinding,
    ImportAlias {
        target: Vec<String>,
    },
}

#[derive(Debug, Clone)]
pub enum SymbolScope {
    Module(Vec<String>),
    TypeAlias {
        module_path: Vec<String>,
        type_name: String,
    },
    Function {
        module_path: Vec<String>,
        function: String,
    },
}

#[derive(Debug, Clone)]
pub struct ResolvedPath {
    pub segments: Vec<String>,
    pub kind: PathKind,
    pub resolved: Option<SymbolInfo>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathKind {
    Type,
    Value,
    Variant,
}

#[derive(Debug, Clone)]
pub struct CapabilityBinding {
    pub name: String,
    pub scope: CapabilityScope,
}

#[derive(Debug, Clone)]
pub enum CapabilityScope {
    Function {
        module_path: Vec<String>,
        function: String,
    },
    TypeAlias {
        module_path: Vec<String>,
        type_name: String,
    },
}

#[derive(Debug, Clone, Default)]
pub struct ModuleExports {
    pub values: HashMap<String, SymbolInfo>,
    pub types: HashMap<String, SymbolInfo>,
    pub variants: HashMap<String, SymbolInfo>,
}
