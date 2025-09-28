use super::*;

#[test]
fn error_display_formats() {
    let err = Error::lex(Some((1, 3)), "bad");
    assert_eq!(format!("{}", err), "Lex error at 1..3: bad");

    let perr = Error::parse(None, "oops");
    assert_eq!(format!("{}", perr), "Parse error: oops");

    assert_eq!(format!("{}", ErrorKind::Lex), "lexical");
    assert_eq!(format!("{}", ErrorKind::Parse), "parse");
}

#[test]
fn binary_op_display_covers_all_symbols() {
    let ops = [
        (BinaryOp::Add, "+"),
        (BinaryOp::Sub, "-"),
        (BinaryOp::Mul, "*"),
        (BinaryOp::Div, "/"),
        (BinaryOp::Mod, "%"),
        (BinaryOp::Eq, "=="),
        (BinaryOp::Ne, "!="),
        (BinaryOp::Lt, "<"),
        (BinaryOp::Le, "<="),
        (BinaryOp::Gt, ">"),
        (BinaryOp::Ge, ">="),
        (BinaryOp::And, "&&"),
        (BinaryOp::Or, "||"),
    ];

    for (op, expected) in ops {
        assert_eq!(format!("{}", op), expected);
    }
}
