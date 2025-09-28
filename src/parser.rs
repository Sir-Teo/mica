use crate::ast::*;
use crate::error::{Error, Result};
use crate::lexer;
use crate::token::{Token, TokenKind};

pub fn parse_module(source: &str) -> Result<Module> {
    let tokens = lexer::lex(source)?;
    Parser::new(tokens).parse_module()
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn parse_module(&mut self) -> Result<Module> {
        self.expect_keyword(TokenKind::Module, "expected 'module' at top of file")?;
        let name = self.parse_module_path()?;
        let mut items = Vec::new();
        while !self.check(TokenKind::Eof) {
            items.push(self.parse_item()?);
        }
        Ok(Module { name, items })
    }

    fn parse_item(&mut self) -> Result<Item> {
        if self.match_keyword(TokenKind::Use) {
            let use_decl = self.parse_use_decl()?;
            self.expect_symbol(TokenKind::Semi, "expected ';' after use statement")?;
            Ok(Item::Use(use_decl))
        } else if self.match_keyword(TokenKind::Type) {
            let alias = self.parse_type_alias(false)?;
            Ok(Item::TypeAlias(alias))
        } else if self.match_keyword(TokenKind::Pub) {
            if self.check(TokenKind::Fn) {
                self.advance();
                let func = self.parse_function(true)?;
                Ok(Item::Function(func))
            } else if self.check(TokenKind::Type) {
                self.advance();
                let alias = self.parse_type_alias(true)?;
                Ok(Item::TypeAlias(alias))
            } else {
                Err(self.error_here("expected 'fn' or 'type' after 'pub'"))
            }
        } else if self.match_keyword(TokenKind::Fn) {
            let func = self.parse_function(false)?;
            Ok(Item::Function(func))
        } else {
            Err(self.error_here("unexpected item"))
        }
    }

    fn parse_use_decl(&mut self) -> Result<UseDecl> {
        let path = self.parse_module_path()?;
        let alias = if self.match_keyword(TokenKind::As) {
            Some(self.expect_identifier()?)
        } else {
            None
        };
        Ok(UseDecl { path, alias })
    }

    fn parse_type_alias(&mut self, is_public: bool) -> Result<TypeAlias> {
        let name = self.expect_identifier()?;
        let params = if self.match_symbol(TokenKind::LBracket) {
            let mut generics = Vec::new();
            if !self.check(TokenKind::RBracket) {
                loop {
                    generics.push(self.expect_identifier()?);
                    if self.match_symbol(TokenKind::Comma) {
                        continue;
                    }
                    break;
                }
            }
            self.expect_symbol(
                TokenKind::RBracket,
                "expected closing ']' in type parameters",
            )?;
            generics
        } else {
            Vec::new()
        };
        self.expect_symbol(TokenKind::Assign, "expected '=' in type alias")?;
        let value = self.parse_type_expr()?;
        Ok(TypeAlias {
            is_public,
            name,
            params,
            value,
        })
    }

    fn parse_function(&mut self, is_public: bool) -> Result<Function> {
        let name = self.expect_identifier()?;
        let generics = if self.match_symbol(TokenKind::LBracket) {
            let mut params = Vec::new();
            if !self.check(TokenKind::RBracket) {
                loop {
                    params.push(self.expect_identifier()?);
                    if self.match_symbol(TokenKind::Comma) {
                        continue;
                    }
                    break;
                }
            }
            self.expect_symbol(
                TokenKind::RBracket,
                "expected closing ']' in generic parameters",
            )?;
            params
        } else {
            Vec::new()
        };

        self.expect_symbol(TokenKind::LParen, "expected '(' in parameter list")?;
        let mut params = Vec::new();
        if !self.check(TokenKind::RParen) {
            loop {
                let mutable = self.match_keyword(TokenKind::Mut);
                let param_name = self.expect_identifier()?;
                self.expect_symbol(TokenKind::Colon, "expected ':' after parameter name")?;
                let ty = self.parse_type_expr()?;
                params.push(Param {
                    name: param_name,
                    ty,
                    mutable,
                });
                if self.match_symbol(TokenKind::Comma) {
                    continue;
                }
                break;
            }
        }
        self.expect_symbol(TokenKind::RParen, "expected ')' to close parameter list")?;

        let return_type = if self.match_symbol(TokenKind::ThinArrow) {
            Some(self.parse_type_expr()?)
        } else {
            None
        };

        let effect_row = if self.check(TokenKind::Bang) {
            self.advance();
            self.expect_symbol(TokenKind::LBrace, "expected '{' after '!' for effect row")?;
            let mut effects = Vec::new();
            if !self.check(TokenKind::RBrace) {
                loop {
                    effects.push(self.expect_identifier()?);
                    if self.match_symbol(TokenKind::Comma) {
                        continue;
                    }
                    break;
                }
            }
            self.expect_symbol(TokenKind::RBrace, "expected '}' to close effect row")?;
            effects
        } else {
            Vec::new()
        };

        let body = self.parse_block()?;
        Ok(Function {
            is_public,
            name,
            generics,
            params,
            return_type,
            effect_row,
            body,
        })
    }

    fn parse_block(&mut self) -> Result<Block> {
        self.expect_symbol(TokenKind::LBrace, "expected '{' to start block")?;
        let mut statements = Vec::new();
        while !self.check(TokenKind::RBrace) && !self.check(TokenKind::Eof) {
            statements.push(self.parse_statement()?);
        }
        self.expect_symbol(TokenKind::RBrace, "expected '}' to close block")?;
        Ok(Block { statements })
    }

    fn parse_statement(&mut self) -> Result<Stmt> {
        if self.match_keyword(TokenKind::Let) {
            let mutable = self.match_keyword(TokenKind::Mut);
            let name = self.expect_identifier()?;
            self.expect_symbol(TokenKind::Assign, "expected '=' in let binding")?;
            let value = self.parse_expression()?;
            self.optional_semicolon();
            Ok(Stmt::Let(LetStmt {
                mutable,
                name,
                value,
            }))
        } else if self.match_keyword(TokenKind::Return) {
            if self.match_symbol(TokenKind::Semi) {
                return Ok(Stmt::Return(None));
            }
            let expr = self.parse_expression()?;
            self.optional_semicolon();
            Ok(Stmt::Return(Some(expr)))
        } else if self.match_keyword(TokenKind::Break) {
            self.optional_semicolon();
            Ok(Stmt::Break)
        } else if self.match_keyword(TokenKind::Continue) {
            self.optional_semicolon();
            Ok(Stmt::Continue)
        } else {
            let expr = self.parse_expression()?;
            self.optional_semicolon();
            Ok(Stmt::Expr(expr))
        }
    }

    fn optional_semicolon(&mut self) {
        if self.check(TokenKind::Semi) {
            self.advance();
        }
    }

    fn parse_expression(&mut self) -> Result<Expr> {
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> Result<Expr> {
        let expr = self.parse_logical_or()?;
        if self.match_symbol(TokenKind::Assign) {
            let value = self.parse_assignment()?;
            Ok(Expr::Assignment {
                target: Box::new(expr),
                value: Box::new(value),
            })
        } else {
            Ok(expr)
        }
    }

    fn parse_logical_or(&mut self) -> Result<Expr> {
        let mut expr = self.parse_logical_and()?;
        while self.match_symbol(TokenKind::OrOr) {
            let rhs = self.parse_logical_and()?;
            expr = Expr::Binary {
                lhs: Box::new(expr),
                op: BinaryOp::Or,
                rhs: Box::new(rhs),
            };
        }
        Ok(expr)
    }

    fn parse_logical_and(&mut self) -> Result<Expr> {
        let mut expr = self.parse_equality()?;
        while self.match_symbol(TokenKind::AndAnd) {
            let rhs = self.parse_equality()?;
            expr = Expr::Binary {
                lhs: Box::new(expr),
                op: BinaryOp::And,
                rhs: Box::new(rhs),
            };
        }
        Ok(expr)
    }

    fn parse_equality(&mut self) -> Result<Expr> {
        let mut expr = self.parse_comparison()?;
        loop {
            if self.match_symbol(TokenKind::EqEq) {
                let rhs = self.parse_comparison()?;
                expr = Expr::Binary {
                    lhs: Box::new(expr),
                    op: BinaryOp::Eq,
                    rhs: Box::new(rhs),
                };
            } else if self.match_symbol(TokenKind::NotEq) {
                let rhs = self.parse_comparison()?;
                expr = Expr::Binary {
                    lhs: Box::new(expr),
                    op: BinaryOp::Ne,
                    rhs: Box::new(rhs),
                };
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_comparison(&mut self) -> Result<Expr> {
        let mut expr = self.parse_term()?;
        loop {
            if self.match_symbol(TokenKind::Lt) {
                let rhs = self.parse_term()?;
                expr = Expr::Binary {
                    lhs: Box::new(expr),
                    op: BinaryOp::Lt,
                    rhs: Box::new(rhs),
                };
            } else if self.match_symbol(TokenKind::Le) {
                let rhs = self.parse_term()?;
                expr = Expr::Binary {
                    lhs: Box::new(expr),
                    op: BinaryOp::Le,
                    rhs: Box::new(rhs),
                };
            } else if self.match_symbol(TokenKind::Gt) {
                let rhs = self.parse_term()?;
                expr = Expr::Binary {
                    lhs: Box::new(expr),
                    op: BinaryOp::Gt,
                    rhs: Box::new(rhs),
                };
            } else if self.match_symbol(TokenKind::Ge) {
                let rhs = self.parse_term()?;
                expr = Expr::Binary {
                    lhs: Box::new(expr),
                    op: BinaryOp::Ge,
                    rhs: Box::new(rhs),
                };
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_term(&mut self) -> Result<Expr> {
        let mut expr = self.parse_factor()?;
        loop {
            if self.match_symbol(TokenKind::Plus) {
                let rhs = self.parse_factor()?;
                expr = Expr::Binary {
                    lhs: Box::new(expr),
                    op: BinaryOp::Add,
                    rhs: Box::new(rhs),
                };
            } else if self.match_symbol(TokenKind::Minus) {
                let rhs = self.parse_factor()?;
                expr = Expr::Binary {
                    lhs: Box::new(expr),
                    op: BinaryOp::Sub,
                    rhs: Box::new(rhs),
                };
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_factor(&mut self) -> Result<Expr> {
        let mut expr = self.parse_unary()?;
        loop {
            if self.match_symbol(TokenKind::Star) {
                let rhs = self.parse_unary()?;
                expr = Expr::Binary {
                    lhs: Box::new(expr),
                    op: BinaryOp::Mul,
                    rhs: Box::new(rhs),
                };
            } else if self.match_symbol(TokenKind::Slash) {
                let rhs = self.parse_unary()?;
                expr = Expr::Binary {
                    lhs: Box::new(expr),
                    op: BinaryOp::Div,
                    rhs: Box::new(rhs),
                };
            } else if self.match_symbol(TokenKind::Percent) {
                let rhs = self.parse_unary()?;
                expr = Expr::Binary {
                    lhs: Box::new(expr),
                    op: BinaryOp::Mod,
                    rhs: Box::new(rhs),
                };
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<Expr> {
        if self.match_symbol(TokenKind::Bang) {
            let expr = self.parse_unary()?;
            Ok(Expr::Unary {
                op: UnaryOp::Not,
                expr: Box::new(expr),
            })
        } else if self.match_symbol(TokenKind::Minus) {
            let expr = self.parse_unary()?;
            Ok(Expr::Unary {
                op: UnaryOp::Neg,
                expr: Box::new(expr),
            })
        } else if self.match_symbol(TokenKind::Ampersand) {
            let expr = self.parse_unary()?;
            Ok(Expr::Unary {
                op: UnaryOp::Ref,
                expr: Box::new(expr),
            })
        } else if self.check(TokenKind::Await) {
            self.advance();
            let expr = self.parse_unary()?;
            Ok(Expr::Await(Box::new(expr)))
        } else if self.check(TokenKind::Spawn) {
            self.advance();
            let expr = self.parse_unary()?;
            Ok(Expr::Spawn(Box::new(expr)))
        } else {
            self.parse_postfix()
        }
    }

    fn parse_postfix(&mut self) -> Result<Expr> {
        let mut expr = self.parse_primary()?;
        loop {
            if self.match_symbol(TokenKind::LParen) {
                let mut args = Vec::new();
                if !self.check(TokenKind::RParen) {
                    loop {
                        args.push(self.parse_expression()?);
                        if self.match_symbol(TokenKind::Comma) {
                            continue;
                        }
                        break;
                    }
                }
                self.expect_symbol(TokenKind::RParen, "expected ')' after call arguments")?;
                expr = Expr::Call {
                    callee: Box::new(expr),
                    args,
                };
            } else if self.match_symbol(TokenKind::Dot) {
                let name = self.expect_identifier()?;
                expr = Expr::Field {
                    expr: Box::new(expr),
                    name,
                };
            } else if self.match_symbol(TokenKind::LBracket) {
                let index = self.parse_expression()?;
                self.expect_symbol(TokenKind::RBracket, "expected ']' after index expression")?;
                expr = Expr::Index {
                    expr: Box::new(expr),
                    index: Box::new(index),
                };
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr> {
        match self.current_kind() {
            TokenKind::Chan => {
                // chan[T](capacity?)
                self.advance();
                self.expect_symbol(TokenKind::LBracket, "expected '[' after 'chan'")?;
                let ty = self.parse_type_expr()?;
                self.expect_symbol(TokenKind::RBracket, "expected ']' after channel type")?;
                let capacity = if self.match_symbol(TokenKind::LParen) {
                    if self.check(TokenKind::RParen) {
                        self.advance();
                        None
                    } else {
                        let expr = self.parse_expression()?;
                        self.expect_symbol(TokenKind::RParen, "expected ')' after channel capacity")?;
                        Some(expr)
                    }
                } else {
                    None
                };
                Ok(Expr::Chan { ty: Box::new(ty), capacity: capacity.map(Box::new) })
            }
            TokenKind::IntLiteral(value) => {
                let value = *value;
                self.advance();
                Ok(Expr::Literal(Literal::Int(value)))
            }
            TokenKind::FloatLiteral(value) => {
                let value = *value;
                self.advance();
                Ok(Expr::Literal(Literal::Float(value)))
            }
            TokenKind::StringLiteral(value) => {
                let value = value.clone();
                self.advance();
                Ok(Expr::Literal(Literal::String(value)))
            }
            TokenKind::BoolLiteral(value) => {
                let value = *value;
                self.advance();
                Ok(Expr::Literal(Literal::Bool(value)))
            }
            TokenKind::Identifier(_) => {
                let path = self.parse_path_expr()?;
                Ok(Expr::Path(path))
            }
            TokenKind::LParen => {
                self.advance();
                if self.match_symbol(TokenKind::RParen) {
                    return Ok(Expr::Literal(Literal::Unit));
                }
                let expr = self.parse_expression()?;
                self.expect_symbol(
                    TokenKind::RParen,
                    "expected ')' to close grouping expression",
                )?;
                Ok(expr)
            }
            TokenKind::If => self.parse_if_expr(),
            TokenKind::Loop => self.parse_loop_expr(),
            TokenKind::While => self.parse_while_expr(),
            TokenKind::For => self.parse_for_expr(),
            TokenKind::Match => self.parse_match_expr(),
            TokenKind::LBrace => {
                let block = self.parse_block()?;
                Ok(Expr::Block(block))
            }
            other => Err(self.error_here(format!("unexpected token in expression: {:?}", other))),
        }
    }

    fn parse_if_expr(&mut self) -> Result<Expr> {
        self.expect_keyword(TokenKind::If, "expected 'if'")?;
        let condition = self.parse_expression()?;
        let then_branch = self.parse_block()?;
        let else_branch = if self.match_keyword(TokenKind::Else) {
            if self.check(TokenKind::If) {
                Some(Box::new(self.parse_if_expr()?))
            } else {
                Some(Box::new(Expr::Block(self.parse_block()?)))
            }
        } else {
            None
        };
        Ok(Expr::If {
            condition: Box::new(condition),
            then_branch: Box::new(Expr::Block(then_branch)),
            else_branch,
        })
    }

    fn parse_loop_expr(&mut self) -> Result<Expr> {
        self.expect_keyword(TokenKind::Loop, "expected 'loop'")?;
        let body = self.parse_block()?;
        Ok(Expr::Loop {
            body: Box::new(Expr::Block(body)),
        })
    }

    fn parse_while_expr(&mut self) -> Result<Expr> {
        self.expect_keyword(TokenKind::While, "expected 'while'")?;
        let condition = self.parse_expression()?;
        let body = self.parse_block()?;
        Ok(Expr::While {
            condition: Box::new(condition),
            body: Box::new(Expr::Block(body)),
        })
    }

    fn parse_for_expr(&mut self) -> Result<Expr> {
        self.expect_keyword(TokenKind::For, "expected 'for'")?;
        let binding = self.expect_identifier()?;
        self.expect_keyword(TokenKind::In, "expected 'in' in for loop")?;
        let iterable = self.parse_expression()?;
        let body = self.parse_block()?;
        Ok(Expr::For {
            binding,
            iterable: Box::new(iterable),
            body: Box::new(Expr::Block(body)),
        })
    }

    fn parse_match_expr(&mut self) -> Result<Expr> {
        self.expect_keyword(TokenKind::Match, "expected 'match'")?;
        let scrutinee = self.parse_expression()?;
        self.expect_symbol(TokenKind::LBrace, "expected '{' to start match arms")?;
        let mut arms = Vec::new();
        while !self.check(TokenKind::RBrace) {
            let pattern = self.parse_pattern()?;
            let guard = if self.match_keyword(TokenKind::If) {
                Some(self.parse_expression()?)
            } else {
                None
            };
            self.expect_symbol(TokenKind::FatArrow, "expected '=>' in match arm")?;
            let body = self.parse_expression()?;
            arms.push(MatchArm {
                pattern,
                guard,
                body,
            });
            if self.match_symbol(TokenKind::Comma) {
                continue;
            }
            break;
        }
        self.expect_symbol(TokenKind::RBrace, "expected '}' to close match")?;
        Ok(Expr::Match {
            scrutinee: Box::new(scrutinee),
            arms,
        })
    }

    fn parse_pattern(&mut self) -> Result<Pattern> {
        match self.current_kind() {
            TokenKind::Identifier(name) => {
                if name == "_" {
                    self.advance();
                    return Ok(Pattern::Wildcard);
                }
                let path = self.parse_path_expr()?;
                if self.match_symbol(TokenKind::LParen) {
                    let mut fields = Vec::new();
                    if !self.check(TokenKind::RParen) {
                        loop {
                            fields.push(self.parse_pattern()?);
                            if self.match_symbol(TokenKind::Comma) {
                                continue;
                            }
                            break;
                        }
                    }
                    self.expect_symbol(TokenKind::RParen, "expected ')' in pattern")?;
                    Ok(Pattern::EnumVariant { path, fields })
                } else if path.segments.len() == 1 {
                    Ok(Pattern::Binding(path.segments.into_iter().next().unwrap()))
                } else {
                    Ok(Pattern::EnumVariant {
                        path,
                        fields: Vec::new(),
                    })
                }
            }
            TokenKind::BoolLiteral(value) => {
                let value = *value;
                self.advance();
                Ok(Pattern::Literal(Literal::Bool(value)))
            }
            TokenKind::IntLiteral(value) => {
                let value = *value;
                self.advance();
                Ok(Pattern::Literal(Literal::Int(value)))
            }
            TokenKind::StringLiteral(value) => {
                let value = value.clone();
                self.advance();
                Ok(Pattern::Literal(Literal::String(value)))
            }
            _ => Err(self.error_here("unsupported pattern")),
        }
    }

    fn parse_path_expr(&mut self) -> Result<Path> {
        let mut segments = Vec::new();
        segments.push(self.expect_identifier()?);
        while self.match_path_sep() {
            segments.push(self.expect_identifier()?);
        }
        Ok(Path { segments })
    }

    fn match_path_sep(&mut self) -> bool {
        if self.check(TokenKind::Dot) || self.check(TokenKind::DoubleColon) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn parse_type_expr(&mut self) -> Result<TypeExpr> {
        if self.match_keyword(TokenKind::Fn) {
            self.expect_symbol(
                TokenKind::LParen,
                "expected '(' after 'fn' in function type",
            )?;
            let mut params = Vec::new();
            if !self.check(TokenKind::RParen) {
                loop {
                    params.push(self.parse_type_expr()?);
                    if self.match_symbol(TokenKind::Comma) {
                        continue;
                    }
                    break;
                }
            }
            self.expect_symbol(
                TokenKind::RParen,
                "expected ')' after function type parameters",
            )?;
            self.expect_symbol(TokenKind::ThinArrow, "expected '->' in function type")?;
            let return_type = self.parse_type_expr()?;
            let effect_row = if self.check(TokenKind::Bang) {
                self.advance();
                self.expect_symbol(
                    TokenKind::LBrace,
                    "expected '{' to open function type effect row",
                )?;
                let mut effects = Vec::new();
                if !self.check(TokenKind::RBrace) {
                    loop {
                        effects.push(self.expect_identifier()?);
                        if self.match_symbol(TokenKind::Comma) {
                            continue;
                        }
                        break;
                    }
                }
                self.expect_symbol(
                    TokenKind::RBrace,
                    "expected '}' to close function type effect row",
                )?;
                effects
            } else {
                Vec::new()
            };
            return Ok(TypeExpr::Function {
                params,
                return_type: Box::new(return_type),
                effect_row,
            });
        }
        if self.match_symbol(TokenKind::LBrace) {
            let mut fields = Vec::new();
            if !self.check(TokenKind::RBrace) {
                loop {
                    let name = self.expect_identifier()?;
                    self.expect_symbol(TokenKind::Colon, "expected ':' in record type")?;
                    let ty = self.parse_type_expr()?;
                    fields.push((name, ty));
                    if self.match_symbol(TokenKind::Comma) {
                        continue;
                    }
                    break;
                }
            }
            self.expect_symbol(TokenKind::RBrace, "expected '}' to close record type")?;
            return Ok(TypeExpr::Record(fields));
        }

        if self.match_symbol(TokenKind::Ampersand) {
            let is_mut = self.match_keyword(TokenKind::Mut);
            let inner = self.parse_type_expr()?;
            return Ok(TypeExpr::Reference {
                is_mut,
                inner: Box::new(inner),
            });
        }

        if self.match_symbol(TokenKind::LParen) {
            if self.match_symbol(TokenKind::RParen) {
                return Ok(TypeExpr::Unit);
            }
            let mut items = Vec::new();
            loop {
                items.push(self.parse_type_expr()?);
                if self.match_symbol(TokenKind::Comma) {
                    continue;
                }
                break;
            }
            self.expect_symbol(TokenKind::RParen, "expected ')' in tuple type")?;
            if items.len() == 1 {
                Ok(items.into_iter().next().unwrap())
            } else {
                Ok(TypeExpr::Tuple(items))
            }
        } else {
            let name = self.expect_identifier()?;
            if self.match_symbol(TokenKind::LBracket) {
                let mut args = Vec::new();
                if !self.check(TokenKind::RBracket) {
                    loop {
                        args.push(self.parse_type_expr()?);
                        if self.match_symbol(TokenKind::Comma) {
                            continue;
                        }
                        break;
                    }
                }
                self.expect_symbol(
                    TokenKind::RBracket,
                    "expected closing ']' in generic arguments",
                )?;
                Ok(TypeExpr::Generic(name, args))
            } else {
                Ok(TypeExpr::Name(name))
            }
        }
    }

    fn expect_keyword(&mut self, keyword: TokenKind, message: &str) -> Result<()> {
        if self.check(keyword.clone()) {
            self.advance();
            Ok(())
        } else {
            Err(self.error_here(message))
        }
    }

    fn match_keyword(&mut self, keyword: TokenKind) -> bool {
        if self.check(keyword.clone()) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn check(&self, kind: TokenKind) -> bool {
        matches!(self.peek_kind(0), Some(k) if *k == kind)
    }

    fn match_symbol(&mut self, kind: TokenKind) -> bool {
        self.match_keyword(kind)
    }

    fn expect_symbol(&mut self, kind: TokenKind, message: &str) -> Result<()> {
        self.expect_keyword(kind, message)
    }

    fn expect_identifier(&mut self) -> Result<String> {
        match self.current_kind() {
            TokenKind::Identifier(name) => {
                let name = name.clone();
                self.advance();
                Ok(name)
            }
            _ => Err(self.error_here("expected identifier")),
        }
    }

    fn parse_module_path(&mut self) -> Result<Vec<String>> {
        let mut segments = Vec::new();
        segments.push(self.expect_identifier()?);
        while self.match_symbol(TokenKind::Dot) {
            segments.push(self.expect_identifier()?);
        }
        Ok(segments)
    }

    fn current_kind(&self) -> &TokenKind {
        &self.tokens[self.pos].kind
    }

    fn current_span(&self) -> (usize, usize) {
        self.tokens[self.pos].span
    }

    fn peek_kind(&self, offset: usize) -> Option<&TokenKind> {
        self.tokens.get(self.pos + offset).map(|t| &t.kind)
    }

    fn advance(&mut self) -> &Token {
        let token = &self.tokens[self.pos];
        self.pos += 1;
        token
    }

    fn error_here<S: Into<String>>(&self, message: S) -> Error {
        Error::parse(Some(self.current_span()), message)
    }
}
