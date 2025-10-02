use std::fmt;

use crate::ir;

pub mod llvm;
pub mod native;
pub mod text;

#[derive(Debug, Clone, Default)]
pub struct BackendOptions {
    pub optimize: bool,
    pub debug_info: bool,
    pub target_triple: Option<String>,
}

pub trait Backend {
    type Output;

    fn compile(&self, module: &ir::Module, options: &BackendOptions)
    -> BackendResult<Self::Output>;
}

pub type BackendResult<T> = Result<T, BackendError>;

#[derive(Debug, Clone)]
pub enum BackendError {
    Unsupported(String),
    Internal(String),
}

impl BackendError {
    pub fn unsupported<T: Into<String>>(message: T) -> Self {
        BackendError::Unsupported(message.into())
    }

    pub fn internal<T: Into<String>>(message: T) -> Self {
        BackendError::Internal(message.into())
    }
}

impl fmt::Display for BackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BackendError::Unsupported(msg) => write!(f, "unsupported backend feature: {}", msg),
            BackendError::Internal(msg) => write!(f, "backend error: {}", msg),
        }
    }
}

impl std::error::Error for BackendError {}

pub fn run<B: Backend>(
    backend: &B,
    module: &ir::Module,
    options: &BackendOptions,
) -> BackendResult<B::Output> {
    backend.compile(module, options)
}
