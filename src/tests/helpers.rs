use super::*;

pub fn path<I: IntoIterator<Item = &'static str>>(segments: I) -> Path {
    Path {
        segments: segments.into_iter().map(|s| s.to_string()).collect(),
    }
}

pub fn literal_int(value: i64) -> Expr {
    Expr::Literal(Literal::Int(value))
}

pub fn literal_bool(value: bool) -> Expr {
    Expr::Literal(Literal::Bool(value))
}

pub fn parse(src: &str) -> Module {
    parser::parse_module(src).expect("parse ok")
}
