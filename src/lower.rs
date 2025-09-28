use crate::ast::*;

#[derive(Debug, Clone)]
pub struct HModule {
    pub name: Vec<String>,
    pub items: Vec<HItem>,
}

#[derive(Debug, Clone)]
pub enum HItem {
    Function(HFunction),
}

#[derive(Debug, Clone)]
pub struct HFunction {
    pub name: String,
    pub params: Vec<String>,
    pub body: HBlock,
}

#[derive(Debug, Clone)]
pub struct HBlock {
    pub stmts: Vec<HStmt>,
}

#[derive(Debug, Clone)]
pub enum HStmt {
    Let { name: String, value: HExpr },
    Expr(HExpr),
    Return(Option<HExpr>),
}

#[derive(Debug, Clone)]
pub enum HExpr {
    Literal(Literal),
    Var(String),
    Path(Path),
    Call { func: HFuncRef, args: Vec<HExpr> },
    Binary { lhs: Box<HExpr>, op: BinaryOp, rhs: Box<HExpr> },
    Block(HBlock),
}

#[derive(Debug, Clone)]
pub enum HFuncRef {
    Function(Path),
    Method(String), // desugared method name, receiver passed as first arg
}

pub fn lower_module(m: &Module) -> HModule {
    let mut items = Vec::new();
    for it in &m.items {
        if let Item::Function(f) = it {
            items.push(HItem::Function(lower_function(f)));
        }
    }
    HModule { name: m.name.clone(), items }
}

fn lower_function(f: &Function) -> HFunction {
    let params = f.params.iter().map(|p| p.name.clone()).collect();
    HFunction { name: f.name.clone(), params, body: lower_block(&f.body) }
}

fn lower_block(b: &Block) -> HBlock {
    let mut stmts = Vec::new();
    for s in &b.statements {
        match s {
            Stmt::Let(l) => stmts.push(HStmt::Let { name: l.name.clone(), value: lower_expr(&l.value) }),
            Stmt::Expr(e) => stmts.push(HStmt::Expr(lower_expr(e))),
            Stmt::Return(e) => stmts.push(HStmt::Return(e.as_ref().map(|e| lower_expr(e)))),
            Stmt::Break | Stmt::Continue => {}
        }
    }
    HBlock { stmts }
}

fn lower_expr(e: &Expr) -> HExpr {
    match e {
        Expr::Literal(l) => HExpr::Literal(l.clone()),
        Expr::Path(p) => HExpr::Path(p.clone()),
        Expr::Block(b) => HExpr::Block(lower_block(b)),
        Expr::Binary { lhs, op, rhs } => HExpr::Binary { lhs: Box::new(lower_expr(lhs)), op: *op, rhs: Box::new(lower_expr(rhs)) },
        Expr::Call { callee, args } => {
            // Detect method call: (Field { expr: recv, name })
            if let Expr::Field { expr: recv, name } = &**callee {
                let mut largs = Vec::with_capacity(args.len() + 1);
                largs.push(lower_expr(recv));
                for a in args { largs.push(lower_expr(a)); }
                HExpr::Call { func: HFuncRef::Method(name.clone()), args: largs }
            } else if let Expr::Path(p) = &**callee {
                HExpr::Call { func: HFuncRef::Function(p.clone()), args: args.iter().map(lower_expr).collect() }
            } else {
                // Fallback: try to stringify callee by lowering then discarding
                HExpr::Call { func: HFuncRef::Method("<expr>".into()), args: args.iter().map(lower_expr).collect() }
            }
        }
        Expr::Field { expr, name } => {
            // As value: keep as Path if simple, else ignore
            let base = match &**expr { Expr::Path(p) => HExpr::Path(p.clone()), _ => lower_expr(expr) };
            // Encode as a call-ready method reference if needed; for now just return the base
            base
        }
        Expr::Index { expr, index } => {
            // Desugar index as method call: index(expr, idx)
            HExpr::Call { func: HFuncRef::Method("index".into()), args: vec![lower_expr(expr), lower_expr(index)] }
        }
        Expr::If { condition, then_branch, else_branch } => {
            // Desugar to ternary-like call for demo
            let mut args = vec![lower_expr(condition), lower_expr(then_branch)];
            if let Some(e) = else_branch { args.push(lower_expr(e)); }
            HExpr::Call { func: HFuncRef::Method("if".into()), args }
        }
        Expr::Assignment { target, value } => {
            HExpr::Call { func: HFuncRef::Method("assign".into()), args: vec![lower_expr(target), lower_expr(value)] }
        }
        Expr::Await(e) => HExpr::Call { func: HFuncRef::Method("await".into()), args: vec![lower_expr(e)] },
        Expr::Spawn(e) => HExpr::Call { func: HFuncRef::Method("spawn".into()), args: vec![lower_expr(e)] },
        Expr::Chan { ty: _, capacity } => {
            let mut args = Vec::new();
            if let Some(c) = capacity { args.push(lower_expr(c)); }
            HExpr::Call { func: HFuncRef::Method("chan".into()), args }
        }
        Expr::Using { expr, body, .. } => {
            HExpr::Call { func: HFuncRef::Method("using".into()), args: vec![lower_expr(expr), HExpr::Block(lower_block(body))] }
        }
        Expr::Try(e) => HExpr::Call { func: HFuncRef::Method("try".into()), args: vec![lower_expr(e)] },
        Expr::Ctor { path, args } => HExpr::Call { func: HFuncRef::Function(path.clone()), args: args.iter().map(lower_expr).collect() },
        // Unhandled: For/While/Loop/Match/Cast; reduce to placeholders for this demo
        Expr::For { .. } => HExpr::Call { func: HFuncRef::Method("for".into()), args: vec![] },
        Expr::While { .. } => HExpr::Call { func: HFuncRef::Method("while".into()), args: vec![] },
        Expr::Loop { .. } => HExpr::Call { func: HFuncRef::Method("loop".into()), args: vec![] },
        Expr::Match { .. } => HExpr::Call { func: HFuncRef::Method("match".into()), args: vec![] },
        Expr::Cast { expr, .. } => lower_expr(expr),
        Expr::Unary { op, expr } => {
            // Desugar unary as call for demo
            let name = match op {
                UnaryOp::Neg => "neg",
                UnaryOp::Not => "not",
                UnaryOp::Ref => "ref",
                UnaryOp::RefMut => "ref_mut",
            };
            HExpr::Call { func: HFuncRef::Method(name.into()), args: vec![lower_expr(expr)] }
        }
    }
}

pub fn hir_to_string(m: &HModule) -> String {
    let mut out = String::new();
    out.push_str(&format!("hir module {}\n", m.name.join(".")));
    for it in &m.items {
        match it {
            HItem::Function(f) => {
                out.push_str(&format!("fn {}({})\n", f.name, f.params.join(", ")));
                for s in &f.body.stmts {
                    match s {
                        HStmt::Let { name, value } => out.push_str(&format!("  let {} = {}\n", name, fmt_expr(value))),
                        HStmt::Expr(e) => out.push_str(&format!("  {}\n", fmt_expr(e))),
                        HStmt::Return(Some(e)) => out.push_str(&format!("  return {}\n", fmt_expr(e))),
                        HStmt::Return(None) => out.push_str("  return\n"),
                    }
                }
            }
        }
    }
    out
}

fn fmt_expr(e: &HExpr) -> String {
    match e {
        HExpr::Literal(Literal::Int(i)) => format!("{}", i),
        HExpr::Literal(Literal::Float(x)) => format!("{}", x),
        HExpr::Literal(Literal::Bool(b)) => format!("{}", b),
        HExpr::Literal(Literal::String(s)) => format!("\"{}\"", s),
        HExpr::Literal(Literal::Unit) => "()".into(),
        HExpr::Var(v) => v.clone(),
        HExpr::Path(p) => p.segments.join("::"),
        HExpr::Binary { lhs, op, rhs } => format!("({} {} {})", fmt_expr(lhs), op, fmt_expr(rhs)),
        HExpr::Block(b) => {
            let mut s = String::from("{");
            for st in &b.stmts { s.push_str(&format!(" {};",&fmt_stmt(st))); }
            s.push_str(" }");
            s
        }
        HExpr::Call { func, args } => {
            let fname = match func { HFuncRef::Function(p) => p.segments.join("::"), HFuncRef::Method(n) => n.clone() };
            let astr = args.iter().map(fmt_expr).collect::<Vec<_>>().join(", ");
            format!("{}({})", fname, astr)
        }
    }
}

fn fmt_stmt(s: &HStmt) -> String {
    match s {
        HStmt::Let { name, value } => format!("let {} = {}", name, fmt_expr(value)),
        HStmt::Expr(e) => fmt_expr(e),
        HStmt::Return(Some(e)) => format!("return {}", fmt_expr(e)),
        HStmt::Return(None) => "return".into(),
    }
}
