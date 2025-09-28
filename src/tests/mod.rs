#![cfg(test)]

use crate::ast::*;
use crate::error::{Error, ErrorKind};
use crate::token::TokenKind;
use crate::{check, lexer, lower, parser, pretty, resolve};

mod helpers;
mod lexer_tests;
mod parser_tests;
mod display_tests;
mod pretty_tests;
mod lowering_tests;
mod resolve_and_check_tests;
