#![allow(dead_code)]

use std::fmt;

/// A complete Mica module produced by the parser.
#[derive(Debug, Clone)]
pub struct Module {
    pub name: Vec<String>,
    pub items: Vec<Item>,
}

#[derive(Debug, Clone)]
pub enum Item {
    Function(Function),
    TypeAlias(TypeAlias),
    Use(UseDecl),
}

#[derive(Debug, Clone)]
pub struct UseDecl {
    pub path: Vec<String>,
    pub alias: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TypeAlias {
    pub is_public: bool,
    pub name: String,
    pub params: Vec<String>,
    pub value: TypeExpr,
}

#[derive(Debug, Clone)]
pub enum TypeExpr {
    Name(String),
    Generic(String, Vec<TypeExpr>),
    Record(Vec<(String, TypeExpr)>),
    Sum(Vec<TypeVariant>),
    List(Box<TypeExpr>),
    Tuple(Vec<TypeExpr>),
    Reference {
        is_mut: bool,
        inner: Box<TypeExpr>,
    },
    Function {
        params: Vec<TypeExpr>,
        return_type: Box<TypeExpr>,
        effect_row: Vec<String>,
    },
    Unit,
}

#[derive(Debug, Clone)]
pub struct TypeVariant {
    pub name: String,
    pub fields: Vec<TypeExpr>,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub is_public: bool,
    pub name: String,
    pub generics: Vec<String>,
    pub params: Vec<Param>,
    pub return_type: Option<TypeExpr>,
    pub effect_row: Vec<String>,
    pub body: Block,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub ty: TypeExpr,
    pub mutable: bool,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub statements: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Let(LetStmt),
    Expr(Expr),
    Return(Option<Expr>),
    Break,
    Continue,
}

#[derive(Debug, Clone)]
pub struct LetStmt {
    pub mutable: bool,
    pub name: String,
    pub value: Expr,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Block(Block),
    Literal(Literal),
    Path(Path),
    Binary {
        lhs: Box<Expr>,
        op: BinaryOp,
        rhs: Box<Expr>,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
    Field {
        expr: Box<Expr>,
        name: String,
    },
    Index {
        expr: Box<Expr>,
        index: Box<Expr>,
    },
    If {
        condition: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Option<Box<Expr>>,
    },
    Match {
        scrutinee: Box<Expr>,
        arms: Vec<MatchArm>,
    },
    For {
        binding: String,
        iterable: Box<Expr>,
        body: Box<Expr>,
    },
    While {
        condition: Box<Expr>,
        body: Box<Expr>,
    },
    Loop {
        body: Box<Expr>,
    },
    Assignment {
        target: Box<Expr>,
        value: Box<Expr>,
    },
    Spawn(Box<Expr>),
    Await(Box<Expr>),
    Chan {
        ty: Box<TypeExpr>,
        capacity: Option<Box<Expr>>,
    },
    Using {
        binding: Option<String>,
        expr: Box<Expr>,
        body: Block,
    },
    Try(Box<Expr>),
}

#[derive(Debug, Clone)]
pub struct Path {
    pub segments: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Expr>,
    pub body: Expr,
}

#[derive(Debug, Clone)]
pub enum Pattern {
    Wildcard,
    Binding(String),
    Literal(Literal),
    Tuple(Vec<Pattern>),
    Record(Vec<(String, Pattern)>),
    EnumVariant { path: Path, fields: Vec<Pattern> },
}

#[derive(Debug, Clone)]
pub enum Literal {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Unit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
    Ref,
    RefMut,
}

impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let symbol = match self {
            BinaryOp::Add => "+",
            BinaryOp::Sub => "-",
            BinaryOp::Mul => "*",
            BinaryOp::Div => "/",
            BinaryOp::Mod => "%",
            BinaryOp::Eq => "==",
            BinaryOp::Ne => "!=",
            BinaryOp::Lt => "<",
            BinaryOp::Le => "<=",
            BinaryOp::Gt => ">",
            BinaryOp::Ge => ">=",
            BinaryOp::And => "&&",
            BinaryOp::Or => "||",
        };
        write!(f, "{}", symbol)
    }
}
