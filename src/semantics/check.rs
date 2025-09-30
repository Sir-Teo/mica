use std::collections::{HashMap, HashSet};

use crate::syntax::ast::*;

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub message: String,
}

#[derive(Debug, Default)]
pub struct CheckResult {
    pub diagnostics: Vec<Diagnostic>,
}

pub fn check_module(module: &Module) -> CheckResult {
    let mut checker = Checker::new(module);
    checker.run();
    CheckResult {
        diagnostics: checker.diagnostics,
    }
}

fn check_exhaustiveness(module: &Module) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    let mut adts: Vec<(String, Vec<String>)> = Vec::new();

    for item in &module.items {
        if let Item::TypeAlias(ta) = item {
            if let TypeExpr::Sum(vars) = &ta.value {
                let variants: Vec<String> = vars.iter().map(|v| v.name.clone()).collect();
                adts.push((ta.name.clone(), variants));
            }
        }
    }

    for item in &module.items {
        if let Item::Function(f) = item {
            visit_block(&f.body, &adts, &mut diags);
        }
    }

    diags
}

fn visit_block(block: &Block, adts: &[(String, Vec<String>)], diags: &mut Vec<Diagnostic>) {
    for stmt in &block.statements {
        match stmt {
            Stmt::Expr(expr) => visit_expr(expr, adts, diags),
            Stmt::Let(binding) => visit_expr(&binding.value, adts, diags),
            Stmt::Return(Some(expr)) => visit_expr(expr, adts, diags),
            Stmt::Return(None) | Stmt::Break | Stmt::Continue => {}
        }
    }
}

fn visit_expr(expr: &Expr, adts: &[(String, Vec<String>)], diags: &mut Vec<Diagnostic>) {
    match expr {
        Expr::Match { arms, .. } => {
            let mut seen: Vec<String> = Vec::new();
            let mut saw_wild_or_bind = false;
            for arm in arms {
                match &arm.pattern {
                    Pattern::EnumVariant { path, .. } => {
                        if let Some(last) = path.segments.last() {
                            seen.push(last.clone());
                        }
                    }
                    Pattern::Wildcard | Pattern::Binding(_) => saw_wild_or_bind = true,
                    _ => {}
                }
                visit_expr(&arm.body, adts, diags);
            }
            if !saw_wild_or_bind && !seen.is_empty() {
                seen.sort();
                seen.dedup();
                if let Some((name, all)) = adts
                    .iter()
                    .find(|(_, all)| seen.iter().all(|v| all.contains(v)))
                {
                    let missing: Vec<&str> = all
                        .iter()
                        .map(|s| s.as_str())
                        .filter(|v| !seen.iter().any(|x| x == *v))
                        .collect();
                    if !missing.is_empty() {
                        diags.push(Diagnostic {
                            message: format!(
                                "non-exhaustive match for {}: missing variants {}",
                                name,
                                missing.join(", ")
                            ),
                        });
                    }
                }
            }
        }
        Expr::Block(b) => visit_block(b, adts, diags),
        Expr::Binary { lhs, rhs, .. } => {
            visit_expr(lhs, adts, diags);
            visit_expr(rhs, adts, diags)
        }
        Expr::Unary { expr, .. } => visit_expr(expr, adts, diags),
        Expr::Call { callee, args } => {
            visit_expr(callee, adts, diags);
            for a in args {
                visit_expr(a, adts, diags)
            }
        }
        Expr::Ctor { args, .. } => {
            for a in args {
                visit_expr(a, adts, diags)
            }
        }
        Expr::Field { expr, .. } => visit_expr(expr, adts, diags),
        Expr::Record { fields, .. } => {
            for (_, value) in fields {
                visit_expr(value, adts, diags);
            }
        }
        Expr::Index { expr, index } => {
            visit_expr(expr, adts, diags);
            visit_expr(index, adts, diags)
        }
        Expr::If {
            condition,
            then_branch,
            else_branch,
        } => {
            visit_expr(condition, adts, diags);
            visit_expr(then_branch, adts, diags);
            if let Some(e) = else_branch {
                visit_expr(e, adts, diags)
            }
        }
        Expr::For { iterable, body, .. } => {
            visit_expr(iterable, adts, diags);
            visit_expr(body, adts, diags)
        }
        Expr::While { condition, body } => {
            visit_expr(condition, adts, diags);
            visit_expr(body, adts, diags)
        }
        Expr::Loop { body } => visit_expr(body, adts, diags),
        Expr::Assignment { target, value } => {
            visit_expr(target, adts, diags);
            visit_expr(value, adts, diags)
        }
        Expr::Spawn(e) | Expr::Await(e) | Expr::Try(e) => visit_expr(e, adts, diags),
        Expr::Chan { capacity, .. } => {
            if let Some(c) = capacity {
                visit_expr(c, adts, diags)
            }
        }
        Expr::Using { expr, body, .. } => {
            visit_expr(expr, adts, diags);
            for s in &body.statements {
                if let Stmt::Expr(e) = s {
                    visit_expr(e, adts, diags)
                }
            }
        }
        _ => {}
    }
}

struct Checker<'a> {
    module: &'a Module,
    diagnostics: Vec<Diagnostic>,
    functions: HashMap<String, FunctionSig>,
    variants: HashMap<Vec<String>, VariantInfo>,
}

impl<'a> Checker<'a> {
    fn new(module: &'a Module) -> Self {
        let mut checker = Self {
            module,
            diagnostics: Vec::new(),
            functions: HashMap::new(),
            variants: HashMap::new(),
        };
        checker.collect_signatures();
        checker.collect_variants();
        checker
    }

    fn run(&mut self) {
        for item in &self.module.items {
            if let Item::Function(func) = item {
                if let Some(sig) = self.functions.get(&func.name).cloned() {
                    self.check_function(func, sig);
                }
            }
        }

        self.diagnostics.extend(check_exhaustiveness(self.module));
    }

    fn collect_signatures(&mut self) {
        for item in &self.module.items {
            if let Item::Function(func) = item {
                let sig = FunctionSig::from_function(func);
                self.functions.insert(func.name.clone(), sig);
            }
        }
    }

    fn collect_variants(&mut self) {
        for item in &self.module.items {
            if let Item::TypeAlias(alias) = item {
                if let TypeExpr::Sum(variants) = &alias.value {
                    self.register_variants(alias, variants);
                }
            }
        }
    }

    fn register_variants(&mut self, alias: &TypeAlias, variants: &[TypeVariant]) {
        let generics: HashSet<String> = alias.params.iter().cloned().collect();
        let mut module_parent = self.module.name.clone();
        module_parent.push(alias.name.clone());
        let generic_args: Vec<TypeRepr> = alias
            .params
            .iter()
            .map(|param| TypeRepr::Generic(param.clone()))
            .collect();
        let parent_type = TypeRepr::Named(vec![alias.name.clone()], generic_args);

        for variant in variants {
            let fields = variant
                .fields
                .iter()
                .map(|ty| parse_type_expr(ty, &generics))
                .collect();
            let info = VariantInfo {
                fields,
                ty: parent_type.clone(),
            };

            let mut fully_qualified = module_parent.clone();
            fully_qualified.push(variant.name.clone());
            self.variants.insert(fully_qualified, info.clone());
            self.variants
                .insert(vec![alias.name.clone(), variant.name.clone()], info.clone());
            self.variants.insert(vec![variant.name.clone()], info);
        }
    }

    fn check_function(&mut self, func: &Function, sig: FunctionSig) {
        let checker = FunctionChecker::new(
            &func.name,
            sig,
            &self.module.name,
            &self.functions,
            &self.variants,
            &mut self.diagnostics,
        );
        checker.check(func);
    }
}

#[derive(Clone)]
struct FunctionSig {
    params: Vec<(String, TypeRepr)>,
    return_type: Option<TypeRepr>,
    function_type: TypeRepr,
    effects: Vec<String>,
}

impl FunctionSig {
    fn from_function(func: &Function) -> Self {
        let generics: HashSet<String> = func.generics.iter().map(|g| g.name.clone()).collect();
        let params: Vec<(String, TypeRepr)> = func
            .params
            .iter()
            .map(|param| (param.name.clone(), parse_type_expr(&param.ty, &generics)))
            .collect();

        let return_type = func
            .return_type
            .as_ref()
            .map(|ty| parse_type_expr(ty, &generics));

        let function_type = TypeRepr::Function {
            params: params.iter().map(|(_, ty)| ty.clone()).collect(),
            return_type: Box::new(return_type.clone().unwrap_or(TypeRepr::Unit)),
            effects: func.effect_row.clone(),
        };

        Self {
            params,
            return_type,
            function_type,
            effects: func.effect_row.clone(),
        }
    }
}

#[derive(Clone)]
struct VariantInfo {
    fields: Vec<TypeRepr>,
    ty: TypeRepr,
}

struct FunctionChecker<'a, 'm> {
    name: &'a str,
    params: Vec<(String, TypeRepr)>,
    return_type: Option<TypeRepr>,
    effects: Vec<String>,
    module_path: &'m [String],
    functions: &'a HashMap<String, FunctionSig>,
    variants: &'a HashMap<Vec<String>, VariantInfo>,
    diagnostics: &'a mut Vec<Diagnostic>,
    scopes: Vec<HashMap<String, TypeRepr>>,
}

impl<'a, 'm> FunctionChecker<'a, 'm> {
    fn new(
        name: &'a str,
        sig: FunctionSig,
        module_path: &'m [String],
        functions: &'a HashMap<String, FunctionSig>,
        variants: &'a HashMap<Vec<String>, VariantInfo>,
        diagnostics: &'a mut Vec<Diagnostic>,
    ) -> Self {
        Self {
            name,
            params: sig.params,
            return_type: sig.return_type,
            effects: sig.effects,
            module_path,
            functions,
            variants,
            diagnostics,
            scopes: vec![HashMap::new()],
        }
    }

    fn check(mut self, func: &Function) {
        let mut seen_caps = HashSet::new();
        for cap in &self.effects {
            if !seen_caps.insert(cap.clone()) {
                self.diagnostics.push(Diagnostic {
                    message: format!(
                        "function '{}' has duplicate capability '{}' in effect row",
                        self.name, cap
                    ),
                });
            }
        }

        let params = self.params.clone();
        for (param, ty) in params {
            self.bind(param, ty);
        }

        for capability in &self.effects {
            if !self.has_capability(capability) {
                self.diagnostics.push(Diagnostic {
                    message: format!(
                        "function '{}' declares capability '{}' but has no parameter with that name",
                        self.name, capability
                    ),
                });
            }
        }

        let block_ty = self.check_block(&func.body);
        if let Some(expected) = &self.return_type {
            if let Some(actual) = block_ty {
                if !types_compatible(expected, &actual) {
                    self.diagnostics.push(Diagnostic {
                        message: format!(
                            "function '{}' returns '{}' but expected '{}'",
                            self.name,
                            actual.describe(),
                            expected.describe()
                        ),
                    });
                }
            }
        } else if let Some(actual) = block_ty {
            if !matches!(actual, TypeRepr::Unit | TypeRepr::Unknown) {
                self.diagnostics.push(Diagnostic {
                    message: format!(
                        "function '{}' returns value of type '{}' but is declared without return type",
                        self.name,
                        actual.describe()
                    ),
                });
            }
        }
    }

    fn bind(&mut self, name: String, ty: TypeRepr) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, ty);
        }
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn lookup_value(&self, name: &str) -> Option<TypeRepr> {
        for scope in self.scopes.iter().rev() {
            if let Some(ty) = scope.get(name) {
                return Some(ty.clone());
            }
        }
        None
    }

    fn check_block(&mut self, block: &Block) -> Option<TypeRepr> {
        self.push_scope();
        let mut last_type = TypeRepr::Unit;
        let expected_return = self.return_type.clone();
        for stmt in &block.statements {
            match stmt {
                Stmt::Let(let_stmt) => {
                    if let Some(value_ty) = self.check_expr(&let_stmt.value) {
                        self.bind(let_stmt.name.clone(), value_ty);
                    } else {
                        self.bind(let_stmt.name.clone(), TypeRepr::Unknown);
                    }
                    last_type = TypeRepr::Unit;
                }
                Stmt::Expr(expr) => {
                    last_type = self.check_expr(expr).unwrap_or(TypeRepr::Unknown);
                }
                Stmt::Return(Some(expr)) => {
                    if let Some(expected) = &expected_return {
                        if let Some(actual) = self.check_expr(expr) {
                            if !types_compatible(expected, &actual) {
                                self.diagnostics.push(Diagnostic {
                                    message: format!(
                                        "returning '{}' but function '{}' expects '{}'",
                                        actual.describe(),
                                        self.name,
                                        expected.describe()
                                    ),
                                });
                            }
                        }
                    } else if let Some(actual) = self.check_expr(expr) {
                        self.diagnostics.push(Diagnostic {
                            message: format!(
                                "function '{}' does not declare a return type but returns '{}'",
                                self.name,
                                actual.describe()
                            ),
                        });
                    }
                    last_type = TypeRepr::Unit;
                }
                Stmt::Return(None) | Stmt::Break | Stmt::Continue => {
                    last_type = TypeRepr::Unit;
                }
            }
        }
        self.pop_scope();
        Some(last_type)
    }

    fn check_expr(&mut self, expr: &Expr) -> Option<TypeRepr> {
        match expr {
            Expr::Block(block) => self.check_block(block),
            Expr::Literal(lit) => Some(match lit {
                Literal::Int(_) => TypeRepr::Primitive(PrimitiveType::Int),
                Literal::Float(_) => TypeRepr::Primitive(PrimitiveType::Float),
                Literal::Bool(_) => TypeRepr::Primitive(PrimitiveType::Bool),
                Literal::String(_) => TypeRepr::Primitive(PrimitiveType::String),
                Literal::Unit => TypeRepr::Unit,
            }),
            Expr::Path(path) => self.check_path(path),
            Expr::Binary { lhs, rhs, op } => {
                let lhs_ty = self.check_expr(lhs);
                let rhs_ty = self.check_expr(rhs);
                self.check_binary(op, lhs_ty, rhs_ty)
            }
            Expr::Unary { op, expr } => {
                let ty = self.check_expr(expr);
                self.check_unary(*op, ty)
            }
            Expr::Call { callee, args } => self.check_call(callee, args),
            Expr::Ctor { path, args } => self.check_ctor(path, args),
            Expr::Record { fields, .. } => {
                let mut record_fields = Vec::new();
                for (name, expr) in fields {
                    let ty = self.check_expr(expr).unwrap_or(TypeRepr::Unknown);
                    record_fields.push((name.clone(), ty));
                }
                Some(TypeRepr::Record(record_fields))
            }
            Expr::Field { expr, .. } => self.check_expr(expr),
            Expr::Index { expr, .. } => self.check_expr(expr),
            Expr::Cast { expr, ty } => {
                self.check_expr(expr);
                let parsed = parse_type_expr(ty, &HashSet::new());
                Some(parsed)
            }
            Expr::If {
                condition,
                then_branch,
                else_branch,
            } => {
                if let Some(cond_ty) = self.check_expr(condition) {
                    if !types_compatible(&TypeRepr::Primitive(PrimitiveType::Bool), &cond_ty) {
                        self.diagnostics.push(Diagnostic {
                            message: format!(
                                "if condition in '{}' is '{}' but must be Bool",
                                self.name,
                                cond_ty.describe()
                            ),
                        });
                    }
                }
                let then_ty = self.check_expr(then_branch).unwrap_or(TypeRepr::Unknown);
                if let Some(else_expr) = else_branch {
                    let else_ty = self.check_expr(else_expr).unwrap_or(TypeRepr::Unknown);
                    if !types_compatible(&then_ty, &else_ty) {
                        self.diagnostics.push(Diagnostic {
                            message: format!(
                                "if branches in '{}' return '{}' and '{}'",
                                self.name,
                                then_ty.describe(),
                                else_ty.describe()
                            ),
                        });
                    }
                }
                Some(then_ty)
            }
            Expr::Match { scrutinee, arms } => {
                let scrutinee_ty = self.check_expr(scrutinee);
                let mut arm_type: Option<TypeRepr> = None;
                for arm in arms {
                    self.push_scope();
                    self.bind_pattern(&arm.pattern, scrutinee_ty.as_ref());
                    if let Some(guard) = &arm.guard {
                        if let Some(guard_ty) = self.check_expr(guard) {
                            if !types_compatible(
                                &TypeRepr::Primitive(PrimitiveType::Bool),
                                &guard_ty,
                            ) {
                                self.diagnostics.push(Diagnostic {
                                    message: format!(
                                        "match guard in '{}' has type '{}' but must be Bool",
                                        self.name,
                                        guard_ty.describe()
                                    ),
                                });
                            }
                        }
                    }
                    let body_ty = self.check_expr(&arm.body).unwrap_or(TypeRepr::Unknown);
                    if let Some(existing) = &arm_type {
                        if !types_compatible(existing, &body_ty) {
                            self.diagnostics.push(Diagnostic {
                                message: format!(
                                    "match arms in '{}' have incompatible types '{}' and '{}'",
                                    self.name,
                                    existing.describe(),
                                    body_ty.describe()
                                ),
                            });
                        }
                    } else {
                        arm_type = Some(body_ty);
                    }
                    self.pop_scope();
                }
                arm_type
            }
            Expr::For { body, .. } | Expr::While { body, .. } | Expr::Loop { body } => {
                self.check_expr(body);
                Some(TypeRepr::Unit)
            }
            Expr::Assignment { target, value } => {
                let target_ty = self.check_expr(target).unwrap_or(TypeRepr::Unknown);
                if let Some(value_ty) = self.check_expr(value) {
                    if !types_compatible(&target_ty, &value_ty) {
                        self.diagnostics.push(Diagnostic {
                            message: format!(
                                "cannot assign '{}' to '{}' in '{}'",
                                value_ty.describe(),
                                target_ty.describe(),
                                self.name
                            ),
                        });
                    }
                }
                Some(target_ty)
            }
            Expr::Spawn(expr) | Expr::Await(expr) | Expr::Try(expr) => {
                self.check_expr(expr);
                Some(TypeRepr::Unknown)
            }
            Expr::Chan { ty, capacity } => {
                if let Some(cap) = capacity {
                    self.check_expr(cap);
                }
                let inner = parse_type_expr(ty, &HashSet::new());
                Some(TypeRepr::Named(vec!["Chan".into()], vec![inner]))
            }
            Expr::Using {
                expr,
                body,
                binding,
            } => {
                self.check_expr(expr);
                self.push_scope();
                if let Some(name) = binding {
                    self.bind(name.clone(), TypeRepr::Unknown);
                }
                let body_ty = self.check_block(body).unwrap_or(TypeRepr::Unknown);
                self.pop_scope();
                if matches!(body_ty, TypeRepr::Unit) {
                    Some(TypeRepr::Unknown)
                } else {
                    Some(body_ty)
                }
            }
        }
    }

    fn check_path(&mut self, path: &Path) -> Option<TypeRepr> {
        if path.segments.len() == 1 {
            let name = &path.segments[0];
            if let Some(value) = self.lookup_value(name) {
                return Some(value);
            }
            if let Some(sig) = self.functions.get(name) {
                return Some(sig.function_type.clone());
            }
        }
        None
    }

    fn check_binary(
        &mut self,
        op: &BinaryOp,
        lhs: Option<TypeRepr>,
        rhs: Option<TypeRepr>,
    ) -> Option<TypeRepr> {
        let lhs = lhs.unwrap_or(TypeRepr::Unknown);
        let rhs = rhs.unwrap_or(TypeRepr::Unknown);
        match op {
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
                if !types_compatible(&lhs, &rhs) {
                    self.diagnostics.push(Diagnostic {
                        message: format!(
                            "binary operator '{}' in '{}' used with '{}' and '{}'",
                            op,
                            self.name,
                            lhs.describe(),
                            rhs.describe()
                        ),
                    });
                }
                Some(lhs)
            }
            BinaryOp::Eq | BinaryOp::Ne => Some(TypeRepr::Primitive(PrimitiveType::Bool)),
            BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => {
                if !types_compatible(&lhs, &rhs) {
                    self.diagnostics.push(Diagnostic {
                        message: format!(
                            "comparison operator '{}' in '{}' used with '{}' and '{}'",
                            op,
                            self.name,
                            lhs.describe(),
                            rhs.describe()
                        ),
                    });
                }
                Some(TypeRepr::Primitive(PrimitiveType::Bool))
            }
            BinaryOp::And | BinaryOp::Or => {
                let bool_ty = TypeRepr::Primitive(PrimitiveType::Bool);
                if !types_compatible(&bool_ty, &lhs) {
                    self.diagnostics.push(Diagnostic {
                        message: format!(
                            "operator '{}' in '{}' expects Bool but found '{}'",
                            op,
                            self.name,
                            lhs.describe()
                        ),
                    });
                }
                if !types_compatible(&bool_ty, &rhs) {
                    self.diagnostics.push(Diagnostic {
                        message: format!(
                            "operator '{}' in '{}' expects Bool but found '{}'",
                            op,
                            self.name,
                            rhs.describe()
                        ),
                    });
                }
                Some(bool_ty)
            }
        }
    }

    fn check_unary(&mut self, op: UnaryOp, ty: Option<TypeRepr>) -> Option<TypeRepr> {
        let ty = ty.unwrap_or(TypeRepr::Unknown);
        match op {
            UnaryOp::Neg => Some(ty),
            UnaryOp::Not => {
                let bool_ty = TypeRepr::Primitive(PrimitiveType::Bool);
                if !types_compatible(&bool_ty, &ty) {
                    self.diagnostics.push(Diagnostic {
                        message: format!(
                            "operator '!' in '{}' expects Bool but found '{}'",
                            self.name,
                            ty.describe()
                        ),
                    });
                }
                Some(bool_ty)
            }
            UnaryOp::Ref | UnaryOp::RefMut => Some(TypeRepr::Unknown),
        }
    }

    fn check_call(&mut self, callee: &Expr, args: &[Expr]) -> Option<TypeRepr> {
        let callee_ty = self.check_expr(callee);
        let Some(TypeRepr::Function {
            params,
            return_type,
            effects,
        }) = callee_ty
        else {
            if let Some(actual) = callee_ty {
                self.diagnostics.push(Diagnostic {
                    message: format!(
                        "attempted to call expression of type '{}' in '{}'",
                        actual.describe(),
                        self.name
                    ),
                });
            }
            return Some(TypeRepr::Unknown);
        };

        if params.len() != args.len() {
            self.diagnostics.push(Diagnostic {
                message: format!(
                    "call in '{}' expected {} arguments but found {}",
                    self.name,
                    params.len(),
                    args.len()
                ),
            });
        }

        for (expected, arg) in params.iter().zip(args.iter()) {
            if let Some(actual) = self.check_expr(arg) {
                if !types_compatible(expected, &actual) {
                    self.diagnostics.push(Diagnostic {
                        message: format!(
                            "argument to call in '{}' has type '{}' but parameter expects '{}'",
                            self.name,
                            actual.describe(),
                            expected.describe()
                        ),
                    });
                }
            }
        }

        for capability in &effects {
            if !self.effects.iter().any(|declared| declared == capability) {
                self.diagnostics.push(Diagnostic {
                    message: format!(
                        "call in '{}' uses capability '{}' but the function does not declare it in its effect row",
                        self.name, capability
                    ),
                });
            }
            if !self.has_capability(capability) {
                self.diagnostics.push(Diagnostic {
                    message: format!(
                        "call in '{}' requires capability '{}' which is not in scope",
                        self.name, capability
                    ),
                });
            }
        }

        Some(*return_type)
    }

    fn check_ctor(&mut self, path: &Path, args: &[Expr]) -> Option<TypeRepr> {
        if let Some(info) = self.lookup_variant(&path.segments).cloned() {
            if info.fields.len() != args.len() {
                self.diagnostics.push(Diagnostic {
                    message: format!(
                        "constructor '{}' in '{}' expects {} fields but found {}",
                        path.segments.join("::"),
                        self.name,
                        info.fields.len(),
                        args.len()
                    ),
                });
            }

            for (expected, expr) in info.fields.iter().zip(args.iter()) {
                if let Some(actual) = self.check_expr(expr) {
                    if !types_compatible(expected, &actual) {
                        self.diagnostics.push(Diagnostic {
                            message: format!(
                                "constructor '{}' field expected '{}' but found '{}'",
                                path.segments.join("::"),
                                expected.describe(),
                                actual.describe()
                            ),
                        });
                    }
                }
            }

            Some(info.ty)
        } else {
            self.diagnostics.push(Diagnostic {
                message: format!(
                    "unknown constructor '{}' in '{}'",
                    path.segments.join("::"),
                    self.name
                ),
            });
            Some(TypeRepr::Unknown)
        }
    }

    fn lookup_variant(&self, path: &[String]) -> Option<&VariantInfo> {
        if let Some(info) = self.variants.get(path) {
            return Some(info);
        }

        if path.len() > 1 {
            let mut qualified = self.module_path.to_vec();
            qualified.extend_from_slice(path);
            if let Some(info) = self.variants.get(&qualified) {
                return Some(info);
            }
        }

        if let Some(last) = path.last() {
            self.variants.get(&vec![last.clone()])
        } else {
            None
        }
    }

    fn bind_pattern(&mut self, pattern: &Pattern, ty: Option<&TypeRepr>) {
        match pattern {
            Pattern::Wildcard | Pattern::Literal(_) => {}
            Pattern::Binding(name) => {
                self.bind(name.clone(), ty.cloned().unwrap_or(TypeRepr::Unknown));
            }
            Pattern::Tuple(patterns) => {
                if let Some(TypeRepr::Tuple(items)) = ty {
                    for (pat, item_ty) in patterns.iter().zip(items.iter()) {
                        self.bind_pattern(pat, Some(item_ty));
                    }
                } else {
                    for pat in patterns {
                        self.bind_pattern(pat, None);
                    }
                }
            }
            Pattern::Record(fields) => {
                let mut field_map = HashMap::new();
                if let Some(TypeRepr::Record(items)) = ty {
                    for (name, ty) in items {
                        field_map.insert(name.clone(), ty.clone());
                    }
                }
                for (name, pat) in fields {
                    let field_ty = field_map.get(name);
                    self.bind_pattern(pat, field_ty);
                }
            }
            Pattern::EnumVariant { path, fields } => {
                if let Some(info) = self.lookup_variant(&path.segments).cloned() {
                    for (pat, field_ty) in fields.iter().zip(info.fields.iter()) {
                        self.bind_pattern(pat, Some(field_ty));
                    }
                } else {
                    self.diagnostics.push(Diagnostic {
                        message: format!(
                            "unknown variant '{}' in pattern within '{}'",
                            path.segments.join("::"),
                            self.name
                        ),
                    });
                }
            }
        }
    }

    fn has_capability(&self, name: &str) -> bool {
        if self.params.iter().any(|(param, _)| param == name) {
            return true;
        }
        self.lookup_value(name).is_some()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum PrimitiveType {
    Int,
    Float,
    Bool,
    String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum TypeRepr {
    Unit,
    Primitive(PrimitiveType),
    Named(Vec<String>, Vec<TypeRepr>),
    Tuple(Vec<TypeRepr>),
    List(Box<TypeRepr>),
    Record(Vec<(String, TypeRepr)>),
    Function {
        params: Vec<TypeRepr>,
        return_type: Box<TypeRepr>,
        effects: Vec<String>,
    },
    Generic(String),
    Unknown,
}

impl TypeRepr {
    fn describe(&self) -> String {
        match self {
            TypeRepr::Unit => "Unit".into(),
            TypeRepr::Primitive(p) => match p {
                PrimitiveType::Int => "Int".into(),
                PrimitiveType::Float => "Float".into(),
                PrimitiveType::Bool => "Bool".into(),
                PrimitiveType::String => "String".into(),
            },
            TypeRepr::Named(path, args) => {
                let mut name = path.join("::");
                if !args.is_empty() {
                    let args = args.iter().map(|arg| arg.describe()).collect::<Vec<_>>();
                    name.push('[');
                    name.push_str(&args.join(", "));
                    name.push(']');
                }
                name
            }
            TypeRepr::Tuple(items) => format!(
                "({})",
                items
                    .iter()
                    .map(|item| item.describe())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            TypeRepr::List(inner) => format!("[{}]", inner.describe()),
            TypeRepr::Record(fields) => {
                let entries = fields
                    .iter()
                    .map(|(name, ty)| format!("{}: {}", name, ty.describe()))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{{{}}}", entries)
            }
            TypeRepr::Function {
                params,
                return_type,
                effects,
            } => {
                let params = params
                    .iter()
                    .map(|param| param.describe())
                    .collect::<Vec<_>>()
                    .join(", ");
                let mut repr = format!("fn({}) -> {}", params, return_type.describe());
                if !effects.is_empty() {
                    repr.push_str(" !{");
                    repr.push_str(&effects.join(", "));
                    repr.push('}');
                }
                repr
            }
            TypeRepr::Generic(name) => name.clone(),
            TypeRepr::Unknown => "<unknown>".into(),
        }
    }
}

fn types_compatible(expected: &TypeRepr, actual: &TypeRepr) -> bool {
    match (expected, actual) {
        (_, TypeRepr::Unknown) | (TypeRepr::Unknown, _) => true,
        (TypeRepr::Generic(_), _) => true,
        (TypeRepr::Unit, TypeRepr::Unit) => true,
        (TypeRepr::Primitive(a), TypeRepr::Primitive(b)) => a == b,
        (TypeRepr::Named(a_path, a_args), TypeRepr::Named(b_path, b_args)) => {
            a_path == b_path
                && a_args.len() == b_args.len()
                && a_args
                    .iter()
                    .zip(b_args.iter())
                    .all(|(a, b)| types_compatible(a, b))
        }
        (TypeRepr::Tuple(a_items), TypeRepr::Tuple(b_items)) => {
            a_items.len() == b_items.len()
                && a_items
                    .iter()
                    .zip(b_items.iter())
                    .all(|(a, b)| types_compatible(a, b))
        }
        (TypeRepr::List(a_inner), TypeRepr::List(b_inner)) => types_compatible(a_inner, b_inner),
        (TypeRepr::Record(a_fields), TypeRepr::Record(b_fields)) => {
            if a_fields.len() != b_fields.len() {
                return false;
            }
            a_fields
                .iter()
                .zip(b_fields.iter())
                .all(|((a_name, a_ty), (b_name, b_ty))| {
                    a_name == b_name && types_compatible(a_ty, b_ty)
                })
        }
        (
            TypeRepr::Function {
                params: a_params,
                return_type: a_ret,
                effects: a_eff,
            },
            TypeRepr::Function {
                params: b_params,
                return_type: b_ret,
                effects: b_eff,
            },
        ) => {
            a_params.len() == b_params.len()
                && a_params
                    .iter()
                    .zip(b_params.iter())
                    .all(|(a, b)| types_compatible(a, b))
                && types_compatible(a_ret, b_ret)
                && a_eff == b_eff
        }
        _ => false,
    }
}

fn parse_type_expr(expr: &TypeExpr, generics: &HashSet<String>) -> TypeRepr {
    match expr {
        TypeExpr::Name(name) => {
            if let Some(primitive) = primitive_from_name(name) {
                TypeRepr::Primitive(primitive)
            } else if generics.contains(name) {
                TypeRepr::Generic(name.clone())
            } else {
                TypeRepr::Named(vec![name.clone()], Vec::new())
            }
        }
        TypeExpr::Generic(name, args) => {
            let args = args
                .iter()
                .map(|arg| parse_type_expr(arg, generics))
                .collect();
            TypeRepr::Named(vec![name.clone()], args)
        }
        TypeExpr::Record(fields) => TypeRepr::Record(
            fields
                .iter()
                .map(|(name, ty)| (name.clone(), parse_type_expr(ty, generics)))
                .collect(),
        ),
        TypeExpr::Sum(_) => TypeRepr::Named(vec!["<sum>".into()], Vec::new()),
        TypeExpr::List(inner) => TypeRepr::List(Box::new(parse_type_expr(inner, generics))),
        TypeExpr::Tuple(items) => TypeRepr::Tuple(
            items
                .iter()
                .map(|item| parse_type_expr(item, generics))
                .collect(),
        ),
        TypeExpr::Reference { inner, .. } => {
            TypeRepr::Named(vec!["Ref".into()], vec![parse_type_expr(inner, generics)])
        }
        TypeExpr::Function {
            params,
            return_type,
            effect_row,
        } => TypeRepr::Function {
            params: params
                .iter()
                .map(|param| parse_type_expr(param, generics))
                .collect(),
            return_type: Box::new(parse_type_expr(return_type, generics)),
            effects: effect_row.clone(),
        },
        TypeExpr::SelfType => TypeRepr::Named(vec!["Self".into()], Vec::new()),
        TypeExpr::Unit => TypeRepr::Unit,
    }
}

fn primitive_from_name(name: &str) -> Option<PrimitiveType> {
    match name {
        "Int" => Some(PrimitiveType::Int),
        "F64" | "Float" => Some(PrimitiveType::Float),
        "Bool" => Some(PrimitiveType::Bool),
        "String" => Some(PrimitiveType::String),
        _ => None,
    }
}
