use super::*;
use super::helpers::*;

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
fn lower_covers_all_expression_kinds() {
    let using_body = Block {
        statements: vec![Stmt::Expr(literal_bool(true))],
    };

    let complex_block = Block {
        statements: vec![
            Stmt::Let(LetStmt {
                mutable: false,
                name: "value".into(),
                value: literal_int(1),
            }),
            Stmt::Expr(Expr::Binary {
                lhs: Box::new(literal_int(1)),
                op: BinaryOp::Add,
                rhs: Box::new(literal_int(2)),
            }),
            Stmt::Expr(Expr::Unary {
                op: UnaryOp::Neg,
                expr: Box::new(literal_int(3)),
            }),
            Stmt::Expr(Expr::Unary {
                op: UnaryOp::Not,
                expr: Box::new(literal_bool(true)),
            }),
            Stmt::Expr(Expr::Unary {
                op: UnaryOp::Ref,
                expr: Box::new(literal_int(4)),
            }),
            Stmt::Expr(Expr::Unary {
                op: UnaryOp::RefMut,
                expr: Box::new(literal_int(5)),
            }),
            Stmt::Expr(Expr::Call {
                callee: Box::new(Expr::Field {
                    expr: Box::new(Expr::Path(path(["value"]))),
                    name: "method".into(),
                }),
                args: vec![literal_int(6)],
            }),
            Stmt::Expr(Expr::Call {
                callee: Box::new(Expr::Path(path(["fun"]))),
                args: vec![literal_int(7)],
            }),
            Stmt::Expr(Expr::Call {
                callee: Box::new(Expr::Binary {
                    lhs: Box::new(literal_int(1)),
                    op: BinaryOp::Add,
                    rhs: Box::new(literal_int(2)),
                }),
                args: vec![literal_int(8)],
            }),
            Stmt::Expr(Expr::Field {
                expr: Box::new(Expr::Binary {
                    lhs: Box::new(literal_int(9)),
                    op: BinaryOp::Add,
                    rhs: Box::new(literal_int(10)),
                }),
                name: "field".into(),
            }),
            Stmt::Expr(Expr::Index {
                expr: Box::new(Expr::Path(path(["array"]))),
                index: Box::new(literal_int(0)),
            }),
            Stmt::Expr(Expr::If {
                condition: Box::new(literal_bool(true)),
                then_branch: Box::new(literal_int(1)),
                else_branch: Some(Box::new(literal_int(2))),
            }),
            Stmt::Expr(Expr::Assignment {
                target: Box::new(Expr::Path(path(["value"]))),
                value: Box::new(literal_int(11)),
            }),
            Stmt::Expr(Expr::Await(Box::new(Expr::Path(path(["task"]))))),
            Stmt::Expr(Expr::Spawn(Box::new(Expr::Path(path(["task"]))))),
            Stmt::Expr(Expr::Chan {
                ty: Box::new(TypeExpr::Name("Int".into())),
                capacity: Some(Box::new(literal_int(1))),
            }),
            Stmt::Expr(Expr::Chan {
                ty: Box::new(TypeExpr::Name("Int".into())),
                capacity: None,
            }),
            Stmt::Expr(Expr::Using {
                binding: Some("f".into()),
                expr: Box::new(Expr::Try(Box::new(Expr::Path(path(["File", "open"]))))),
                body: using_body.clone(),
            }),
            Stmt::Expr(Expr::Ctor {
                path: path(["Option", "Some"]),
                args: vec![literal_int(12)],
            }),
            Stmt::Expr(Expr::Record {
                type_path: Some(path(["Row"])),
                fields: vec![("value".into(), literal_int(19))],
            }),
            Stmt::Expr(Expr::Match {
                scrutinee: Box::new(Expr::Path(path(["value"]))),
                arms: vec![MatchArm {
                    pattern: Pattern::EnumVariant {
                        path: path(["Option", "Some"]),
                        fields: vec![Pattern::Binding("x".into())],
                    },
                    guard: None,
                    body: literal_int(13),
                }],
            }),
            Stmt::Expr(Expr::For {
                binding: "item".into(),
                iterable: Box::new(Expr::Path(path(["items"]))),
                body: Box::new(literal_int(14)),
            }),
            Stmt::Expr(Expr::While {
                condition: Box::new(literal_bool(true)),
                body: Box::new(literal_int(15)),
            }),
            Stmt::Expr(Expr::Loop {
                body: Box::new(literal_int(16)),
            }),
            Stmt::Expr(Expr::Cast {
                expr: Box::new(literal_int(17)),
                ty: TypeExpr::Name("Int".into()),
            }),
            Stmt::Return(Some(literal_int(18))),
            Stmt::Return(None),
        ],
    };

    let function = Function {
        is_public: false,
        name: "complex".into(),
        generics: vec![],
        params: vec![Param {
            name: "value".into(),
            mutable: false,
            ty: TypeExpr::Name("Int".into()),
        }],
        return_type: Some(TypeExpr::Name("Int".into())),
        effect_row: vec![],
        body: complex_block,
    };

    let module = Module {
        name: vec!["demo".into()],
        items: vec![Item::Function(function)],
    };
    let lowered = lower::lower_module(&module);
    let dump = lower::hir_to_string(&lowered);

    assert!(dump.contains("fn complex(value)"));
    assert!(dump.contains("method(value, 6)"));
    assert!(dump.contains("fun(7)"));
    assert!(dump.contains("<expr>(8)"));
    assert!(dump.contains("index(array, 0)"));
    assert!(dump.contains("if(true, 1, 2)"));
    assert!(dump.contains("assign(value, 11)"));
    assert!(dump.contains("await(task)"));
    assert!(dump.contains("spawn(task)"));
    assert!(dump.contains("chan(1)"));
    assert!(dump.contains("chan()"));
    assert!(dump.contains("using(try(File::open), { true; })"));
    assert!(dump.contains("Option::Some(12)"));
    assert!(dump.contains("Row { value: 19 }"));
    assert!(dump.contains("match()"));
    assert!(dump.contains("for()"));
    assert!(dump.contains("while()"));
    assert!(dump.contains("loop()"));
    assert!(dump.contains("neg(3)"));
    assert!(dump.contains("not(true)"));
    assert!(dump.contains("ref(4)"));
    assert!(dump.contains("ref_mut(5)"));
    assert!(dump.contains("return 18"));
    assert!(dump.contains("return"));
}
