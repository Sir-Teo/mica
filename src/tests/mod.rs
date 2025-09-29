#![cfg(test)]

use crate::diagnostics::error::{Error, ErrorKind};
use crate::semantics::{check, resolve};
use crate::syntax::{ast::*, lexer, parser, token::TokenKind};
use crate::{lower, pretty};

mod display_tests;
mod helpers;
mod lexer_tests;
mod lowering_tests;
mod parser_tests;
mod pretty_tests;
mod resolve_and_check_tests;
