pub mod backend;
pub mod diagnostics;
pub mod ir;
pub mod lower;
pub mod pretty;
pub mod runtime;
pub mod semantics;
pub mod syntax;
pub mod tooling;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

pub use diagnostics::error;
pub use diagnostics::{Error, ErrorKind, Result};
pub use semantics::{check, resolve};
pub use syntax::{ast, lexer, parser, token};

#[cfg(test)]
mod tests;
