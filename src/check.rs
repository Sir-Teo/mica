use crate::ast::*;

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub message: String,
}

pub fn check_exhaustiveness(module: &Module) -> Vec<Diagnostic> {
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

    // Walk and check on the fly (no borrowing issues)
    for item in &module.items {
        if let Item::Function(f) = item {
            visit_expr(&Expr::Block(f.body.clone()), &adts, &mut diags);
        }
    }

    diags
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
                seen.sort(); seen.dedup();
                if let Some((name, all)) = adts.iter().find(|(_, all)| seen.iter().all(|v| all.contains(v))) {
                    let missing: Vec<&str> = all.iter().map(|s| s.as_str()).filter(|v| !seen.iter().any(|x| x == *v)).collect();
                    if !missing.is_empty() { diags.push(Diagnostic { message: format!("non-exhaustive match for {}: missing variants {}", name, missing.join(", ")) }); }
                }
            }
        }
        Expr::Block(b) => for s in &b.statements { match s { Stmt::Expr(e) => visit_expr(e, adts, diags), Stmt::Let(l) => visit_expr(&l.value, adts, diags), Stmt::Return(Some(e)) => visit_expr(e, adts, diags), _ => {} }},
        Expr::Binary { lhs, rhs, .. } => { visit_expr(lhs, adts, diags); visit_expr(rhs, adts, diags) }
        Expr::Unary { expr, .. } => visit_expr(expr, adts, diags),
        Expr::Call { callee, args } => { visit_expr(callee, adts, diags); for a in args { visit_expr(a, adts, diags) } }
        Expr::Ctor { args, .. } => { for a in args { visit_expr(a, adts, diags) } }
        Expr::Field { expr, .. } => visit_expr(expr, adts, diags),
        Expr::Record { fields, .. } => {
            for (_, value) in fields {
                visit_expr(value, adts, diags);
            }
        }
        Expr::Index { expr, index } => { visit_expr(expr, adts, diags); visit_expr(index, adts, diags) }
        Expr::If { condition, then_branch, else_branch } => { visit_expr(condition, adts, diags); visit_expr(then_branch, adts, diags); if let Some(e) = else_branch { visit_expr(e, adts, diags) } }
        Expr::For { iterable, body, .. } => { visit_expr(iterable, adts, diags); visit_expr(body, adts, diags) }
        Expr::While { condition, body } => { visit_expr(condition, adts, diags); visit_expr(body, adts, diags) }
        Expr::Loop { body } => visit_expr(body, adts, diags),
        Expr::Assignment { target, value } => { visit_expr(target, adts, diags); visit_expr(value, adts, diags) }
        Expr::Spawn(e) | Expr::Await(e) | Expr::Try(e) => visit_expr(e, adts, diags),
        Expr::Chan { capacity, .. } => if let Some(c) = capacity { visit_expr(c, adts, diags) },
        Expr::Using { expr, body, .. } => { visit_expr(expr, adts, diags); for s in &body.statements { if let Stmt::Expr(e) = s { visit_expr(e, adts, diags) } } },
        _ => {}
    }
}

