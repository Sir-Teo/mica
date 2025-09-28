#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: (usize, usize),
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Keywords
    Module,
    Pub,
    Fn,
    Type,
    Impl,
    Use,
    Let,
    Mut,
    Return,
    If,
    Else,
    Match,
    For,
    In,
    Loop,
    While,
    Break,
    Continue,
    Spawn,
    Await,
    Chan,
    Using,
    As,

    // Literals and identifiers
    Identifier(String),
    IntLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    BoolLiteral(bool),

    // Symbols
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Colon,
    Semi,
    Dot,
    ThinArrow,
    FatArrow,
    Assign,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Ampersand,
    Pipe,
    Question,
    Bang,
    EqEq,
    NotEq,
    Lt,
    Le,
    Gt,
    Ge,
    AndAnd,
    OrOr,

    Eof,
}

impl Token {
    pub fn new(kind: TokenKind, span: (usize, usize)) -> Self {
        Self { kind, span }
    }
}
