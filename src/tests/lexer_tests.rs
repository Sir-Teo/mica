use super::*;

type TokenPredicate = Box<dyn Fn(&TokenKind) -> bool>;

#[test]
fn lex_double_colon_and_question() {
    let src = "File::open(\"/tmp/x\")?";
    let toks = lexer::lex(src).expect("lex ok");
    use crate::syntax::token::TokenKind::*;
    assert!(toks.iter().any(|t| matches!(t.kind, DoubleColon)));
    assert!(toks.iter().any(|t| matches!(t.kind, Question)));
}

#[test]
fn lexer_keywords_and_symbols_cover_every_branch() {
    let cases: Vec<(&str, TokenPredicate)> = vec![
        ("module", Box::new(|t| matches!(t, TokenKind::Module))),
        ("pub", Box::new(|t| matches!(t, TokenKind::Pub))),
        ("fn", Box::new(|t| matches!(t, TokenKind::Fn))),
        ("type", Box::new(|t| matches!(t, TokenKind::Type))),
        ("impl", Box::new(|t| matches!(t, TokenKind::Impl))),
        ("use", Box::new(|t| matches!(t, TokenKind::Use))),
        ("let", Box::new(|t| matches!(t, TokenKind::Let))),
        ("mut", Box::new(|t| matches!(t, TokenKind::Mut))),
        ("return", Box::new(|t| matches!(t, TokenKind::Return))),
        ("if", Box::new(|t| matches!(t, TokenKind::If))),
        ("else", Box::new(|t| matches!(t, TokenKind::Else))),
        ("match", Box::new(|t| matches!(t, TokenKind::Match))),
        ("for", Box::new(|t| matches!(t, TokenKind::For))),
        ("in", Box::new(|t| matches!(t, TokenKind::In))),
        ("loop", Box::new(|t| matches!(t, TokenKind::Loop))),
        ("while", Box::new(|t| matches!(t, TokenKind::While))),
        ("break", Box::new(|t| matches!(t, TokenKind::Break))),
        ("continue", Box::new(|t| matches!(t, TokenKind::Continue))),
        ("spawn", Box::new(|t| matches!(t, TokenKind::Spawn))),
        ("await", Box::new(|t| matches!(t, TokenKind::Await))),
        ("chan", Box::new(|t| matches!(t, TokenKind::Chan))),
        ("using", Box::new(|t| matches!(t, TokenKind::Using))),
        ("as", Box::new(|t| matches!(t, TokenKind::As))),
        (
            "true",
            Box::new(|t| matches!(t, TokenKind::BoolLiteral(true))),
        ),
        (
            "false",
            Box::new(|t| matches!(t, TokenKind::BoolLiteral(false))),
        ),
        (
            "identifier",
            Box::new(|t| matches!(t, TokenKind::Identifier(s) if s == "identifier")),
        ),
        ("123", Box::new(|t| matches!(t, TokenKind::IntLiteral(123)))),
        (
            "1_000",
            Box::new(|t| matches!(t, TokenKind::IntLiteral(1000))),
        ),
        (
            "3.14",
            Box::new(|t| {
                let expected = "3.14".parse::<f64>().unwrap();
                matches!(
                    t,
                    TokenKind::FloatLiteral(v) if (*v - expected).abs() < f64::EPSILON
                )
            }),
        ),
        (
            "6.02e23",
            Box::new(|t| matches!(t, TokenKind::FloatLiteral(v) if (*v - 6.02e23).abs() < 1.0)),
        ),
        (
            "\"hi\\n\"",
            Box::new(|t| matches!(t, TokenKind::StringLiteral(s) if s == "hi\n")),
        ),
        ("(", Box::new(|t| matches!(t, TokenKind::LParen))),
        (")", Box::new(|t| matches!(t, TokenKind::RParen))),
        ("{", Box::new(|t| matches!(t, TokenKind::LBrace))),
        ("}", Box::new(|t| matches!(t, TokenKind::RBrace))),
        ("[", Box::new(|t| matches!(t, TokenKind::LBracket))),
        ("]", Box::new(|t| matches!(t, TokenKind::RBracket))),
        (",", Box::new(|t| matches!(t, TokenKind::Comma))),
        (":", Box::new(|t| matches!(t, TokenKind::Colon))),
        (";", Box::new(|t| matches!(t, TokenKind::Semi))),
        (".", Box::new(|t| matches!(t, TokenKind::Dot))),
        ("->", Box::new(|t| matches!(t, TokenKind::ThinArrow))),
        ("=>", Box::new(|t| matches!(t, TokenKind::FatArrow))),
        ("=", Box::new(|t| matches!(t, TokenKind::Assign))),
        ("+", Box::new(|t| matches!(t, TokenKind::Plus))),
        ("-", Box::new(|t| matches!(t, TokenKind::Minus))),
        ("*", Box::new(|t| matches!(t, TokenKind::Star))),
        ("/", Box::new(|t| matches!(t, TokenKind::Slash))),
        ("%", Box::new(|t| matches!(t, TokenKind::Percent))),
        ("&", Box::new(|t| matches!(t, TokenKind::Ampersand))),
        ("&&", Box::new(|t| matches!(t, TokenKind::AndAnd))),
        ("|", Box::new(|t| matches!(t, TokenKind::Pipe))),
        ("||", Box::new(|t| matches!(t, TokenKind::OrOr))),
        ("::", Box::new(|t| matches!(t, TokenKind::DoubleColon))),
        ("?", Box::new(|t| matches!(t, TokenKind::Question))),
        ("!", Box::new(|t| matches!(t, TokenKind::Bang))),
        ("!=", Box::new(|t| matches!(t, TokenKind::NotEq))),
        ("==", Box::new(|t| matches!(t, TokenKind::EqEq))),
        ("<", Box::new(|t| matches!(t, TokenKind::Lt))),
        ("<=", Box::new(|t| matches!(t, TokenKind::Le))),
        (">", Box::new(|t| matches!(t, TokenKind::Gt))),
        (">=", Box::new(|t| matches!(t, TokenKind::Ge))),
    ];

    for (src, check) in cases {
        let toks = lexer::lex(src).expect("lex ok");
        assert!(!toks.is_empty());
        assert!(
            check(&toks[0].kind),
            "unexpected token for {src:?}: {:?}",
            toks[0].kind
        );
    }

    // Comments are skipped entirely
    let comment = lexer::lex("// comment\n").expect("lex ok");
    assert_eq!(comment.len(), 1);
    assert!(matches!(comment[0].kind, TokenKind::Eof));

    // Mixed identifiers and double colons
    let tokens = lexer::lex("path::to").expect("lex ok");
    assert_eq!(
        tokens
            .iter()
            .filter(|t| matches!(t.kind, TokenKind::DoubleColon))
            .count(),
        1
    );
}

#[test]
fn lexer_reports_errors_cleanly() {
    let unterminated = lexer::lex("\"abc");
    assert!(matches!(unterminated, Err(e) if e.message.contains("unterminated")));

    let bad_escape = lexer::lex("\"\\x\"");
    assert!(matches!(bad_escape, Err(e) if e.message.contains("unsupported escape")));

    let bad_char = lexer::lex("@");
    assert!(matches!(bad_char, Err(e) if e.message.contains("unexpected character")));
}
