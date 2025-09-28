#![cfg(test)]

use crate::{parser, lexer, pretty, resolve, check, lower};
use crate::ast::*;

fn parse(src: &str) -> Module { parser::parse_module(src).expect("parse ok") }

#[test]
fn lex_double_colon_and_question() {
    let src = "File::open(\"/tmp/x\")?";
    let toks = lexer::lex(src).expect("lex ok");
    use crate::token::TokenKind::*;
    assert!(toks.iter().any(|t| matches!(t.kind, DoubleColon)));
    assert!(toks.iter().any(|t| matches!(t.kind, Question)));
}

#[test]
fn parse_adt_sum() {
    let m = parse("module m
pub type Option[T] = Some(T) | None
pub type Result[T,E] = Ok(T) | Err(E)");
    assert_eq!(m.items.len(), 2);
    match &m.items[0] {
        Item::TypeAlias(ta) => {
            assert_eq!(ta.name, "Option");
            match &ta.value { TypeExpr::Sum(vars) => {
                let names: Vec<_> = vars.iter().map(|v| v.name.as_str()).collect();
                assert_eq!(names, vec!["Some","None"]);
            }, _ => panic!("expected sum") }
        }
        _ => panic!("expected type alias"),
    }
}

#[test]
fn parse_using_try_and_chan() {
    let src = r#"
      module demo
      fn main(io: IO) !{io} {
        using File::open("/tmp/x", io)? {
          let q = chan[Int](3)
          q
        }
      }
    "#;
    let m = parse(src);
    let f = match &m.items[0] { Item::Function(f) => f, _ => panic!() };
    let stmts = &f.body.statements;
    if let Stmt::Expr(Expr::Using { binding, expr, body }) = &stmts[0] {
        assert!(binding.is_none());
        match &**expr { Expr::Try(inner) => match &**inner { Expr::Call{ callee, .. } => match &**callee { Expr::Path(p) => assert_eq!(p.segments, vec!["File","open"]), _ => panic!() }, _ => panic!() }, _ => panic!() }
        // inside body, first stmt let q = chan[Int](3)
        if let Stmt::Let(l) = &body.statements[0] {
            match &l.value { Expr::Chan { ty, capacity } => {
                match &**ty { TypeExpr::Name(n) => assert_eq!(n, "Int"), _ => panic!() }
                assert!(matches!(capacity, Some(_)));
            }, _ => panic!() }
        } else { panic!() }
    } else { panic!() }
}

#[test]
fn parse_cast_and_patterns() {
    let src = r#"
      module demo
      type Pair = { a: Int, b: Int }
      fn work(p: Pair) -> Int { match p { { a, b: bb } => (a + bb) as Int } }
    "#;
    let m = parse(src);
    let f = match &m.items[1] { Item::Function(f) => f, _ => panic!() };
    if let Stmt::Expr(Expr::Match { arms, .. }) = &f.body.statements[0] {
        match &arms[0].pattern {
            Pattern::Record(fields) => {
                assert_eq!(fields.len(), 2);
                assert!(matches!(fields[0].1, Pattern::Binding(_)));
            }
            _ => panic!(),
        }
        match &arms[0].body {
            Expr::Cast { ty, .. } => match ty { TypeExpr::Name(n) => assert_eq!(n, "Int"), _ => panic!() },
            _ => panic!(),
        }
    } else { panic!() }
}

#[test]
fn parse_impl_and_self_receiver() {
    let src = r#"
      module demo
      type Vec2 = { x: Int, y: Int }
      impl Addable for Vec2 { fn add(self, other: Vec2) -> Vec2 { other } }
    "#;
    let m = parse(src);
    let ib = match &m.items[1] { Item::Impl(ib) => ib, _ => panic!() };
    let f = match &ib.items[0] { ImplItem::Function(f) => f, _ => panic!() };
    assert!(matches!(f.params[0].ty, TypeExpr::SelfType));
}

#[test]
fn lower_method_call() {
    let src = r#"
      module demo
      type V = { x: Int }
      fn f(a: V, b: V) -> V { a.add(b) }
    "#;
    let m = parse(src);
    let h = lower::lower_module(&m);
    let s = lower::hir_to_string(&h);
    assert!(s.contains("add(a, b)"));
}

#[test]
fn resolve_adts() {
    let m = parse("module m
type A = X | Y");
    let r = resolve::resolve_module(&m);
    assert!(r.adts.contains_key("A"));
    assert_eq!(r.adts["A"], vec!["X".to_string(), "Y".to_string()]);
}

#[test]
fn exhaustiveness_checker() {
    let m1 = parse("module m
type S = A | B
fn f(x: S) -> Int { match x { A => 1 } }");
    let diags = check::check_exhaustiveness(&m1);
    assert!(!diags.is_empty());

    let m2 = parse("module m
type S = A | B
fn f(x: S) -> Int { match x { A => 1, B => 2 } }");
    let diags2 = check::check_exhaustiveness(&m2);
    assert!(diags2.is_empty());
}

#[test]
fn pretty_print_snapshot() {
    let m = parse("module m
pub type Option[T] = Some(T) | None");
    let s = pretty::module_to_string(&m);
    assert!(s.contains("pub type Option[T] = Some(T) | None"));
}

#[test]
fn list_type_parses() {
    let m = parse("module m
fn g(x: [Int]) { x }");
    let f = match &m.items[0] { Item::Function(f) => f, _ => panic!() };
    match &f.params[0].ty { TypeExpr::List(inner) => match &**inner { TypeExpr::Name(n) => assert_eq!(n, "Int"), _ => panic!() }, _ => panic!() }
}
