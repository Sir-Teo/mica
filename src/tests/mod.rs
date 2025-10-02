use crate::diagnostics::error::{Error, ErrorKind};
use crate::semantics::{check, resolve};
use crate::syntax::{ast::*, lexer, parser, token::TokenKind};
use crate::{backend, ir, lower, pretty};

mod backend_tests;
mod display_tests;
mod helpers;
mod ir_tests;
mod lexer_tests;
mod lowering_tests;
mod parser_tests;
mod pipeline_tests;
mod pretty_tests;
mod resolve_and_check_tests;
