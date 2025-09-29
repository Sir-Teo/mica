use super::helpers::*;
use super::*;

#[test]
fn pretty_print_snapshot() {
    let m = parse("module m\npub type Option[T] = Some(T) | None");
    let s = pretty::module_to_string(&m);
    assert!(s.contains("pub type Option[T] = Some(T) | None"));
}

#[test]
fn pretty_print_handles_every_type_form() {
    let module = Module {
        name: vec!["demo".into()],
        items: vec![
            Item::Use(UseDecl {
                path: vec!["std".into(), "io".into()],
                alias: Some("io".into()),
            }),
            Item::TypeAlias(TypeAlias {
                is_public: true,
                name: "Result".into(),
                params: vec!["T".into(), "E".into()],
                value: TypeExpr::Sum(vec![
                    TypeVariant {
                        name: "Ok".into(),
                        fields: vec![TypeExpr::Name("T".into())],
                    },
                    TypeVariant {
                        name: "Err".into(),
                        fields: vec![TypeExpr::Name("E".into())],
                    },
                ]),
            }),
            Item::Function(Function {
                is_public: true,
                name: "process".into(),
                generics: vec![GenericParam {
                    name: "T".into(),
                    bounds: vec![path(["Ord"])],
                }],
                params: vec![
                    Param {
                        name: "input".into(),
                        mutable: false,
                        ty: TypeExpr::List(Box::new(TypeExpr::Name("T".into()))),
                    },
                    Param {
                        name: "handler".into(),
                        mutable: false,
                        ty: TypeExpr::Function {
                            params: vec![TypeExpr::Reference {
                                is_mut: true,
                                inner: Box::new(TypeExpr::Name("T".into())),
                            }],
                            return_type: Box::new(TypeExpr::Unit),
                            effect_row: vec!["io".into()],
                        },
                    },
                ],
                return_type: Some(TypeExpr::Tuple(vec![TypeExpr::Unit, TypeExpr::SelfType])),
                effect_row: vec!["io".into(), "net".into()],
                body: Block { statements: vec![] },
            }),
            Item::Impl(ImplBlock {
                trait_path: path(["Display"]),
                for_type: TypeExpr::Record(vec![
                    ("id".into(), TypeExpr::Name("Int".into())),
                    (
                        "data".into(),
                        TypeExpr::Generic("Vec".into(), vec![TypeExpr::Name("Bytes".into())]),
                    ),
                ]),
                items: vec![],
            }),
        ],
    };

    let printed = pretty::module_to_string(&module);
    assert!(printed.contains("use std.io as io"));
    assert!(printed.contains("pub type Result[T, E] = Ok(T) | Err(E)"));
    assert!(printed.contains("pub fn process[T: Ord](input: [T], handler: fn(&mut T) -> () !{io}) -> ((), Self) !{io, net}"));
    assert!(printed.contains("impl Display for { id: Int, data: Vec[Bytes] }"));
}
