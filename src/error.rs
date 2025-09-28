use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    pub kind: ErrorKind,
    pub span: Option<(usize, usize)>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind {
    Lex,
    Parse,
}

impl Error {
    pub fn lex<S: Into<String>>(span: Option<(usize, usize)>, message: S) -> Self {
        Self {
            kind: ErrorKind::Lex,
            span,
            message: message.into(),
        }
    }

    pub fn parse<S: Into<String>>(span: Option<(usize, usize)>, message: S) -> Self {
        Self {
            kind: ErrorKind::Parse,
            span,
            message: message.into(),
        }
    }
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.span {
            Some((start, end)) => write!(
                f,
                "{:?} error at {}..{}: {}",
                self.kind, start, end, self.message
            ),
            None => write!(f, "{:?} error: {}", self.kind, self.message),
        }
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            ErrorKind::Lex => "lexical",
            ErrorKind::Parse => "parse",
        };
        write!(f, "{}", label)
    }
}
