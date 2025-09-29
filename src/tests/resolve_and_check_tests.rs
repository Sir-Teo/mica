use super::helpers::*;
use super::*;
use crate::semantics::resolve::{CapabilityScope, PathKind, SymbolCategory, SymbolScope};

fn exhaustive_module() -> Module {
    let match_expr = Expr::Match {
        scrutinee: Box::new(Expr::Path(path(["state"]))),
        arms: vec![
            MatchArm {
                pattern: Pattern::EnumVariant {
                    path: path(["State", "Idle"]),
                    fields: vec![],
                },
                guard: None,
                body: literal_int(1),
            },
            MatchArm {
                pattern: Pattern::Binding("other".into()),
                guard: None,
                body: literal_int(2),
            },
        ],
    };

    let body = Block {
        statements: vec![
            Stmt::Expr(match_expr),
            Stmt::Expr(Expr::Block(Block {
                statements: vec![Stmt::Expr(literal_int(3))],
            })),
            Stmt::Expr(Expr::Ctor {
                path: path(["State", "Busy"]),
                args: vec![literal_int(4)],
            }),
            Stmt::Expr(Expr::Field {
                expr: Box::new(Expr::Path(path(["value"]))),
                name: "len".into(),
            }),
            Stmt::Expr(Expr::Index {
                expr: Box::new(Expr::Path(path(["items"]))),
                index: Box::new(literal_int(0)),
            }),
            Stmt::Expr(Expr::If {
                condition: Box::new(literal_bool(true)),
                then_branch: Box::new(literal_int(5)),
                else_branch: Some(Box::new(literal_int(6))),
            }),
            Stmt::Expr(Expr::For {
                binding: "x".into(),
                iterable: Box::new(Expr::Path(path(["items"]))),
                body: Box::new(literal_int(7)),
            }),
            Stmt::Expr(Expr::While {
                condition: Box::new(literal_bool(true)),
                body: Box::new(literal_int(8)),
            }),
            Stmt::Expr(Expr::Loop {
                body: Box::new(literal_int(9)),
            }),
            Stmt::Expr(Expr::Assignment {
                target: Box::new(Expr::Path(path(["value"]))),
                value: Box::new(literal_int(10)),
            }),
            Stmt::Expr(Expr::Spawn(Box::new(Expr::Path(path(["task"]))))),
            Stmt::Expr(Expr::Await(Box::new(Expr::Path(path(["task"]))))),
            Stmt::Expr(Expr::Try(Box::new(Expr::Path(path(["task"]))))),
            Stmt::Expr(Expr::Chan {
                ty: Box::new(TypeExpr::Name("Int".into())),
                capacity: Some(Box::new(literal_int(1))),
            }),
            Stmt::Expr(Expr::Using {
                binding: None,
                expr: Box::new(Expr::Path(path(["guard"]))),
                body: Block {
                    statements: vec![Stmt::Let(LetStmt {
                        mutable: false,
                        name: "v".into(),
                        value: literal_int(11),
                    })],
                },
            }),
        ],
    };

    Module {
        name: vec!["demo".into()],
        items: vec![
            Item::TypeAlias(TypeAlias {
                is_public: false,
                name: "State".into(),
                params: vec![],
                value: TypeExpr::Sum(vec![
                    TypeVariant {
                        name: "Idle".into(),
                        fields: vec![],
                    },
                    TypeVariant {
                        name: "Busy".into(),
                        fields: vec![TypeExpr::Name("Int".into())],
                    },
                ]),
            }),
            Item::Function(Function {
                is_public: false,
                name: "drive".into(),
                generics: vec![],
                params: vec![Param {
                    name: "state".into(),
                    mutable: false,
                    ty: TypeExpr::Name("State".into()),
                }],
                return_type: Some(TypeExpr::Name("Int".into())),
                effect_row: vec![],
                body,
            }),
        ],
    }
}

#[test]
fn resolve_adts() {
    let m = parse("module m\ntype A = X | Y");
    let r = resolve::resolve_module(&m);
    assert!(r.adts.contains_key("A"));
    assert_eq!(r.adts["A"], vec!["X".to_string(), "Y".to_string()]);
}

#[test]
fn exhaustiveness_checker() {
    let m1 = parse("module m\ntype S = A | B\nfn f(x: S) -> Int { match x { A => 1 } }");
    let diags = check::check_exhaustiveness(&m1);
    assert!(!diags.is_empty());

    let m2 = parse("module m\ntype S = A | B\nfn f(x: S) -> Int { match x { A => 1, B => 2 } }");
    let diags2 = check::check_exhaustiveness(&m2);
    assert!(diags2.is_empty());
}

#[test]
fn resolve_and_check_visit_every_path() {
    let module = exhaustive_module();
    let resolved = resolve::resolve_module(&module);
    assert!(resolved.adts.contains_key("State"));
    let idle = resolved
        .variant_to_adt
        .get("Idle")
        .expect("Idle variant tracked");
    assert_eq!(idle, &vec![String::from("State")]);

    let diags = check::check_exhaustiveness(&module);
    assert!(diags.is_empty(), "expected no diagnostics, got {diags:?}");
}

#[test]
fn resolve_collects_symbols_and_paths() {
    let module = Module {
        name: vec!["demo".into()],
        items: vec![
            Item::Use(UseDecl {
                path: vec!["math".into(), "add".into()],
                alias: Some("plus".into()),
            }),
            Item::TypeAlias(TypeAlias {
                is_public: false,
                name: "Pair".into(),
                params: vec!["T".into()],
                value: TypeExpr::Tuple(vec![
                    TypeExpr::Name("T".into()),
                    TypeExpr::Name("T".into()),
                ]),
            }),
            Item::Function(Function {
                is_public: false,
                name: "project".into(),
                generics: vec![GenericParam {
                    name: "T".into(),
                    bounds: vec![],
                }],
                params: vec![Param {
                    name: "pair".into(),
                    ty: TypeExpr::Generic("Pair".into(), vec![TypeExpr::Name("T".into())]),
                    mutable: false,
                }],
                return_type: Some(TypeExpr::Name("T".into())),
                effect_row: vec!["io".into()],
                body: Block {
                    statements: vec![
                        Stmt::Let(LetStmt {
                            mutable: false,
                            name: "first".into(),
                            value: Expr::Path(Path {
                                segments: vec!["pair".into()],
                            }),
                        }),
                        Stmt::Expr(Expr::Path(Path {
                            segments: vec!["first".into()],
                        })),
                    ],
                },
            }),
        ],
    };

    let resolved = resolve::resolve_module(&module);

    assert!(
        resolved
            .imports
            .iter()
            .any(|import| import.alias.as_deref() == Some("plus"))
    );

    let type_symbol = resolved
        .symbols
        .iter()
        .find(|symbol| {
            matches!(symbol.category, SymbolCategory::Type { .. }) && symbol.name == "Pair"
        })
        .expect("type symbol recorded");
    assert!(matches!(type_symbol.scope, SymbolScope::Module(_)));

    assert!(
        resolved.symbols.iter().any(
            |symbol| matches!(symbol.category, SymbolCategory::TypeParam) && symbol.name == "T"
        )
    );

    let paths: Vec<_> = resolved
        .resolved_paths
        .iter()
        .filter(|path| path.segments == vec!["Pair".to_string()])
        .collect();
    assert!(
        !paths.is_empty(),
        "expected to see resolved path for Pair, got {paths:?}"
    );
    assert!(paths.iter().all(|path| matches!(path.kind, PathKind::Type)));

    assert!(resolved.capabilities.iter().any(|cap| {
        cap.name == "io"
            && matches!(
                cap.scope,
                CapabilityScope::Function { ref function, .. } if function == "project"
            )
    }));
}

#[test]
fn resolve_reports_unresolved_paths() {
    let module = Module {
        name: vec!["demo".into()],
        items: vec![Item::Function(Function {
            is_public: false,
            name: "broken".into(),
            generics: vec![],
            params: vec![],
            return_type: None,
            effect_row: vec![],
            body: Block {
                statements: vec![
                    Stmt::Expr(Expr::Path(path(["missing"]))),
                    Stmt::Expr(Expr::Ctor {
                        path: path(["Unknown", "Variant"]),
                        args: vec![],
                    }),
                ],
            },
        })],
    };

    let resolved = resolve::resolve_module(&module);

    assert_eq!(resolved.diagnostics.len(), 2, "expected two diagnostics");

    let missing_value = resolved
        .diagnostics
        .iter()
        .find(|diag| diag.path == vec!["missing".to_string()])
        .expect("missing value path diagnostic");
    assert_eq!(missing_value.kind, PathKind::Value);
    match &missing_value.scope {
        SymbolScope::Function { function, .. } => assert_eq!(function, "broken"),
        other => panic!("expected function scope, got {other:?}"),
    }
    assert!(missing_value.message.contains("unresolved value path"));

    let missing_variant = resolved
        .diagnostics
        .iter()
        .find(|diag| diag.path == vec!["Unknown".to_string(), "Variant".to_string()])
        .expect("missing variant path diagnostic");
    assert_eq!(missing_variant.kind, PathKind::Variant);
    assert!(missing_variant.message.contains("unresolved variant path"));
}

#[test]
fn type_alias_function_effect_row_records_capabilities() {
    let module = parse(
        "module demo\n\
         type Handler = fn(Int) -> Int !{io}\n",
    );

    let resolved = resolve::resolve_module(&module);

    let capability = resolved
        .capabilities
        .iter()
        .find(|cap| cap.name == "io")
        .expect("type alias capability recorded");

    match &capability.scope {
        CapabilityScope::TypeAlias { type_name, .. } => assert_eq!(type_name, "Handler"),
        other => panic!("expected type alias capability scope, got {other:?}"),
    }
}

#[test]
fn resolve_across_modules_uses_workspace_exports() {
    let provider = parse(
        "module provider\n\
         pub type Number = Int\n\
         pub fn zero() -> Number {\n\
           0\n\
         }\n",
    );

    let consumer = parse(
        "module consumer\n\
         use provider.Number;\n\
         fn fetch() -> Number {\n\
           provider::zero()\n\
         }\n",
    );

    let results = resolve::resolve_modules([&provider, &consumer]);
    let consumer_resolved = results
        .get(&vec!["consumer".into()])
        .expect("consumer module resolved");

    assert!(consumer_resolved
        .imports
        .iter()
        .any(|import| import.path == vec!["provider".to_string(), "Number".to_string()]));

    let number_alias = consumer_resolved
        .resolved_paths
        .iter()
        .find(|path| path.segments == vec!["Number".to_string()])
        .expect("resolved path recorded for imported type alias");
    let alias_symbol = number_alias
        .resolved
        .as_ref()
        .expect("import alias resolves to symbol info");
    match &alias_symbol.category {
        SymbolCategory::ImportAlias { target } => {
            assert_eq!(
                target,
                &vec!["provider".to_string(), "Number".to_string()]
            )
        }
        other => panic!("expected import alias symbol, got {other:?}"),
    }

    let provider_call = consumer_resolved
        .resolved_paths
        .iter()
        .find(|path| path.segments == vec!["provider".to_string(), "zero".to_string()])
        .expect("resolved path recorded for cross-module function call");
    let resolved_symbol = provider_call
        .resolved
        .as_ref()
        .expect("cross-module function resolves to symbol info");
    match &resolved_symbol.category {
        SymbolCategory::Function { is_public } => assert!(
            *is_public,
            "expected to resolve a public function exported from provider"
        ),
        other => panic!("expected function symbol, got {other:?}"),
    }
}

#[test]
fn resolve_variants_in_match_patterns_without_prefix() {
    let module = parse(
        "module demo\n\
         type Option[T] = Some(T) | None\n\
         fn unwrap(opt: Option[Int]) -> Int {\n\
           match opt {\n\
             Some(value) => value,\n\
             None => 0,\n\
           }\n\
         }\n",
    );

    let resolved = resolve::resolve_module(&module);

    let some_variant = resolved
        .resolved_paths
        .iter()
        .find(|path| path.segments == vec!["Some".to_string()])
        .expect("expected resolved path for bare Some variant");

    assert!(matches!(some_variant.kind, PathKind::Variant));
    let variant_symbol = some_variant
        .resolved
        .as_ref()
        .expect("variant should resolve to symbol");
    match &variant_symbol.category {
        SymbolCategory::Variant { parent } => assert_eq!(parent, "Option"),
        other => panic!("expected variant symbol, got {other:?}"),
    }

    assert!(resolved.symbols.iter().any(|symbol| {
        matches!(symbol.category, SymbolCategory::LocalBinding)
            && symbol.name == "value"
            && matches!(
                symbol.scope,
                SymbolScope::Function { ref function, .. } if function == "unwrap"
            )
    }));
}

#[test]
fn resolve_impl_items_and_trait_paths() {
    let module = parse(
        "module demo\n\
         type Option[T] = Some(T) | None\n\
         impl Display for Option[Int] {\n\
           fn fmt(self, other: Int, io: IO) -> Int !{io} {\n\
             match self {\n\
               Some(value) => {\n\
                 let result = other;\n\
                 result\n\
               },\n\
               None => {\n\
                 let fallback = 0;\n\
                 fallback\n\
               },\n\
             }\n\
           }\n\
         }\n",
    );

    let resolved = resolve::resolve_module(&module);

    assert!(
        resolved
            .resolved_paths
            .iter()
            .any(|path| path.segments == vec!["Display".to_string()]
                && matches!(path.kind, PathKind::Type))
    );

    assert!(
        resolved
            .resolved_paths
            .iter()
            .any(|path| path.segments == vec!["Option".to_string()]
                && matches!(path.kind, PathKind::Type))
    );

    let capability = resolved
        .capabilities
        .iter()
        .find(|cap| cap.name == "io")
        .expect("function capability recorded");
    match &capability.scope {
        CapabilityScope::Function { function, .. } => assert_eq!(function, "fmt"),
        other => panic!("expected function capability scope, got {other:?}"),
    }

    let some_variant = resolved
        .resolved_paths
        .iter()
        .find(|path| path.segments == vec!["Some".to_string()])
        .expect("expected variant resolution inside impl");
    let symbol = some_variant
        .resolved
        .as_ref()
        .expect("variant symbol present");
    match &symbol.category {
        SymbolCategory::Variant { parent } => assert_eq!(parent, "Option"),
        other => panic!("expected variant symbol, got {other:?}"),
    }

    assert!(resolved.symbols.iter().any(|symbol| {
        matches!(symbol.category, SymbolCategory::LocalBinding)
            && symbol.name == "value"
            && matches!(
                symbol.scope,
                SymbolScope::Function { ref function, .. } if function == "fmt"
            )
    }));

    assert!(resolved.symbols.iter().any(|symbol| {
        matches!(symbol.category, SymbolCategory::LocalBinding)
            && symbol.name == "fallback"
            && matches!(
                symbol.scope,
                SymbolScope::Function { ref function, .. } if function == "fmt"
            )
    }));
}

#[test]
fn resolve_cross_module_paths() {
    let math_module = Module {
        name: vec!["math".into()],
        items: vec![Item::Function(Function {
            is_public: true,
            name: "add".into(),
            generics: vec![],
            params: vec![
                Param {
                    name: "lhs".into(),
                    ty: TypeExpr::Name("Int".into()),
                    mutable: false,
                },
                Param {
                    name: "rhs".into(),
                    ty: TypeExpr::Name("Int".into()),
                    mutable: false,
                },
            ],
            return_type: Some(TypeExpr::Name("Int".into())),
            effect_row: vec![],
            body: Block {
                statements: vec![Stmt::Expr(Expr::Path(path(["lhs"])))],
            },
        })],
    };

    let result_module = Module {
        name: vec!["util".into(), "result".into()],
        items: vec![Item::TypeAlias(TypeAlias {
            is_public: true,
            name: "Outcome".into(),
            params: vec![],
            value: TypeExpr::Sum(vec![
                TypeVariant {
                    name: "Success".into(),
                    fields: vec![TypeExpr::Name("Int".into())],
                },
                TypeVariant {
                    name: "Failure".into(),
                    fields: vec![],
                },
            ]),
        })],
    };

    let app_module = Module {
        name: vec!["app".into()],
        items: vec![Item::Function(Function {
            is_public: false,
            name: "run".into(),
            generics: vec![],
            params: vec![],
            return_type: None,
            effect_row: vec![],
            body: Block {
                statements: vec![
                    Stmt::Let(LetStmt {
                        mutable: false,
                        name: "total".into(),
                        value: Expr::Call {
                            callee: Box::new(Expr::Path(path(["math", "add"]))),
                            args: vec![literal_int(1), literal_int(2)],
                        },
                    }),
                    Stmt::Expr(Expr::Ctor {
                        path: path(["util", "result", "Outcome", "Success"]),
                        args: vec![Expr::Path(path(["total"]))],
                    }),
                ],
            },
        })],
    };

    let modules = vec![&math_module, &result_module, &app_module];
    let resolved = resolve::resolve_modules(modules);

    let app_resolved = resolved.get(&app_module.name).expect("app module resolved");

    let add_path = app_resolved
        .resolved_paths
        .iter()
        .find(|path| path.segments == vec!["math".to_string(), "add".to_string()])
        .expect("math::add path resolved");
    let add_symbol = add_path.resolved.as_ref().expect("symbol for math::add");
    match &add_symbol.category {
        SymbolCategory::Function { is_public } => assert!(*is_public),
        other => panic!("expected function symbol, got {other:?}"),
    }
    match &add_symbol.scope {
        SymbolScope::Module(path) => assert_eq!(path, &vec!["math".to_string()]),
        other => panic!("expected module scope, got {other:?}"),
    }

    let ctor_path = app_resolved
        .resolved_paths
        .iter()
        .find(|path| {
            path.segments
                == vec![
                    "util".to_string(),
                    "result".to_string(),
                    "Outcome".to_string(),
                    "Success".to_string(),
                ]
        })
        .expect("Outcome::Success path resolved");
    let ctor_symbol = ctor_path.resolved.as_ref().expect("variant symbol present");
    match &ctor_symbol.category {
        SymbolCategory::Variant { parent } => assert_eq!(parent, "Outcome"),
        other => panic!("expected variant symbol, got {other:?}"),
    }
    match &ctor_symbol.scope {
        SymbolScope::Module(path) => {
            assert_eq!(path, &vec!["util".to_string(), "result".to_string()]);
        }
        other => panic!("expected module scope, got {other:?}"),
    }
}
