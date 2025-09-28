use super::*;
use super::helpers::*;

#[test]
fn parse_adt_sum() {
    let m = parse(
        "module m\npub type Option[T] = Some(T) | None\npub type Result[T,E] = Ok(T) | Err(E)",
    );
    assert_eq!(m.items.len(), 2);
    match &m.items[0] {
        Item::TypeAlias(ta) => {
            assert_eq!(ta.name, "Option");
            match &ta.value {
                TypeExpr::Sum(vars) => {
                    let names: Vec<_> = vars.iter().map(|v| v.name.as_str()).collect();
                    assert_eq!(names, vec!["Some", "None"]);
                }
                _ => panic!("expected sum"),
            }
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
    let f = match &m.items[0] {
        Item::Function(f) => f,
        _ => panic!(),
    };
    let stmts = &f.body.statements;
    if let Stmt::Expr(Expr::Using {
        binding,
        expr,
        body,
    }) = &stmts[0]
    {
        assert!(binding.is_none());
        match &**expr {
            Expr::Try(inner) => match &**inner {
                Expr::Call { callee, .. } => match &**callee {
                    Expr::Path(p) => assert_eq!(p.segments, vec!["File", "open"]),
                    _ => panic!(),
                },
                _ => panic!(),
            },
            _ => panic!(),
        }
        // inside body, first stmt let q = chan[Int](3)
        if let Stmt::Let(l) = &body.statements[0] {
            match &l.value {
                Expr::Chan { ty, capacity } => {
                    match &**ty {
                        TypeExpr::Name(n) => assert_eq!(n, "Int"),
                        _ => panic!(),
                    }
                    assert!(matches!(capacity, Some(_)));
                }
                _ => panic!(),
            }
        } else {
            panic!()
        }
    } else {
        panic!()
    }
}

#[test]
fn parse_cast_and_patterns() {
    let src = r#"
      module demo
      type Pair = { a: Int, b: Int }
      fn work(p: Pair) -> Int { match p { { a, b: bb } => (a + bb) as Int } }
    "#;
    let m = parse(src);
    let f = match &m.items[1] {
        Item::Function(f) => f,
        _ => panic!(),
    };
    if let Stmt::Expr(Expr::Match { arms, .. }) = &f.body.statements[0] {
        match &arms[0].pattern {
            Pattern::Record(fields) => {
                assert_eq!(fields.len(), 2);
                assert!(matches!(fields[0].1, Pattern::Binding(_)));
            }
            _ => panic!(),
        }
        match &arms[0].body {
            Expr::Cast { ty, .. } => match ty {
                TypeExpr::Name(n) => assert_eq!(n, "Int"),
                _ => panic!(),
            },
            _ => panic!(),
        }
    } else {
        panic!()
    }
}

#[test]
fn parse_impl_and_self_receiver() {
    let src = r#"
      module demo
      type Vec2 = { x: Int, y: Int }
      impl Addable for Vec2 { fn add(self, other: Vec2) -> Vec2 { other } }
    "#;
    let m = parse(src);
    let ib = match &m.items[1] {
        Item::Impl(ib) => ib,
        _ => panic!(),
    };
    let f = match &ib.items[0] {
        ImplItem::Function(f) => f,
    };
    assert!(matches!(f.params[0].ty, TypeExpr::SelfType));
}

#[test]
fn list_type_parses() {
    let m = parse("module m\nfn g(x: [Int]) { x }");
    let f = match &m.items[0] {
        Item::Function(f) => f,
        _ => panic!(),
    };
    match &f.params[0].ty {
        TypeExpr::List(inner) => match &**inner {
            TypeExpr::Name(n) => assert_eq!(n, "Int"),
            _ => panic!(),
        },
        _ => panic!(),
    }
}
