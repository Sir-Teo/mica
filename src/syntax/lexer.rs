use crate::diagnostics::{Error, Result};
use crate::syntax::token::{Token, TokenKind};

pub fn lex(input: &str) -> Result<Vec<Token>> {
    Lexer::new(input).tokenize()
}

struct Lexer<'a> {
    input: &'a str,
    cursor: usize,
    tokens: Vec<Token>,
}

impl<'a> Lexer<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input,
            cursor: 0,
            tokens: Vec::new(),
        }
    }

    fn tokenize(mut self) -> Result<Vec<Token>> {
        while let Some(ch) = self.peek_char() {
            match ch {
                ' ' | '\t' | '\r' | '\n' => {
                    self.bump_char();
                }
                '/' => {
                    if self.peek_next_char() == Some('/') {
                        self.bump_char();
                        self.bump_char();
                        self.skip_line_comment();
                    } else {
                        let span = self.span_start();
                        self.bump_char();
                        self.push_token(TokenKind::Slash, span);
                    }
                }
                '0'..='9' => self.lex_number()?,
                'a'..='z' | 'A'..='Z' | '_' => self.lex_identifier_or_keyword()?,
                '"' => self.lex_string()?,
                '(' => self.simple_token(TokenKind::LParen),
                ')' => self.simple_token(TokenKind::RParen),
                '{' => self.simple_token(TokenKind::LBrace),
                '}' => self.simple_token(TokenKind::RBrace),
                '[' => self.simple_token(TokenKind::LBracket),
                ']' => self.simple_token(TokenKind::RBracket),
                ',' => self.simple_token(TokenKind::Comma),
                ':' => {
                    let span = self.span_start();
                    self.bump_char();
                    if self.peek_char() == Some(':') {
                        self.bump_char();
                        self.push_token(TokenKind::DoubleColon, span);
                    } else {
                        self.push_token(TokenKind::Colon, span);
                    }
                }
                ';' => self.simple_token(TokenKind::Semi),
                '.' => self.simple_token(TokenKind::Dot),
                '+' => self.simple_token(TokenKind::Plus),
                '%' => self.simple_token(TokenKind::Percent),
                '?' => self.simple_token(TokenKind::Question),
                '!' => {
                    let span = self.span_start();
                    self.bump_char();
                    if self.peek_char() == Some('=') {
                        self.bump_char();
                        self.push_token(TokenKind::NotEq, span);
                    } else {
                        self.push_token(TokenKind::Bang, span);
                    }
                }
                '-' => {
                    let span = self.span_start();
                    self.bump_char();
                    if self.peek_char() == Some('>') {
                        self.bump_char();
                        self.push_token(TokenKind::ThinArrow, span);
                    } else {
                        self.push_token(TokenKind::Minus, span);
                    }
                }
                '=' => {
                    let span = self.span_start();
                    self.bump_char();
                    if self.peek_char() == Some('>') {
                        self.bump_char();
                        self.push_token(TokenKind::FatArrow, span);
                    } else if self.peek_char() == Some('=') {
                        self.bump_char();
                        self.push_token(TokenKind::EqEq, span);
                    } else {
                        self.push_token(TokenKind::Assign, span);
                    }
                }
                '*' => self.simple_token(TokenKind::Star),
                '<' => {
                    let span = self.span_start();
                    self.bump_char();
                    if self.peek_char() == Some('=') {
                        self.bump_char();
                        self.push_token(TokenKind::Le, span);
                    } else {
                        self.push_token(TokenKind::Lt, span);
                    }
                }
                '>' => {
                    let span = self.span_start();
                    self.bump_char();
                    if self.peek_char() == Some('=') {
                        self.bump_char();
                        self.push_token(TokenKind::Ge, span);
                    } else {
                        self.push_token(TokenKind::Gt, span);
                    }
                }
                '&' => {
                    let span = self.span_start();
                    self.bump_char();
                    if self.peek_char() == Some('&') {
                        self.bump_char();
                        self.push_token(TokenKind::AndAnd, span);
                    } else {
                        self.push_token(TokenKind::Ampersand, span);
                    }
                }
                '|' => {
                    let span = self.span_start();
                    self.bump_char();
                    if self.peek_char() == Some('|') {
                        self.bump_char();
                        self.push_token(TokenKind::OrOr, span);
                    } else {
                        self.push_token(TokenKind::Pipe, span);
                    }
                }
                _ => {
                    let span = self.span_start();
                    self.bump_char();
                    return Err(Error::lex(
                        Some(span),
                        format!("unexpected character '{}'", ch),
                    ));
                }
            }
        }

        let end = self.cursor;
        self.tokens.push(Token::new(TokenKind::Eof, (end, end)));
        Ok(self.tokens)
    }

    fn lex_number(&mut self) -> Result<()> {
        let start = self.cursor;
        self.consume_digits();

        let mut is_float = false;
        if self.peek_char() == Some('.')
            && self
                .peek_next_char()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
        {
            is_float = true;
            self.bump_char();
            self.consume_digits();
        }

        if matches!(self.peek_char(), Some('e') | Some('E')) {
            is_float = true;
            self.bump_char();
            if matches!(self.peek_char(), Some('+') | Some('-')) {
                self.bump_char();
            }
            self.consume_digits();
        }

        let literal = &self.input[start..self.cursor];
        let filtered: String = literal.chars().filter(|c| *c != '_').collect();

        let span = (start, self.cursor);
        if is_float {
            let value: f64 = filtered
                .parse()
                .map_err(|_| Error::lex(Some(span), "invalid float literal"))?;
            self.push_token(TokenKind::FloatLiteral(value), span);
        } else {
            let value: i64 = filtered
                .parse()
                .map_err(|_| Error::lex(Some(span), "invalid integer literal"))?;
            self.push_token(TokenKind::IntLiteral(value), span);
        }

        Ok(())
    }

    fn lex_identifier_or_keyword(&mut self) -> Result<()> {
        let start = self.cursor;
        self.bump_char();
        while matches!(self.peek_char(), Some(c) if c.is_ascii_alphanumeric() || c == '_') {
            self.bump_char();
        }

        let ident = &self.input[start..self.cursor];
        let span = (start, self.cursor);
        let token = match ident {
            "module" => TokenKind::Module,
            "pub" => TokenKind::Pub,
            "fn" => TokenKind::Fn,
            "type" => TokenKind::Type,
            "impl" => TokenKind::Impl,
            "use" => TokenKind::Use,
            "let" => TokenKind::Let,
            "mut" => TokenKind::Mut,
            "return" => TokenKind::Return,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "match" => TokenKind::Match,
            "for" => TokenKind::For,
            "in" => TokenKind::In,
            "loop" => TokenKind::Loop,
            "while" => TokenKind::While,
            "break" => TokenKind::Break,
            "continue" => TokenKind::Continue,
            "spawn" => TokenKind::Spawn,
            "await" => TokenKind::Await,
            "chan" => TokenKind::Chan,
            "using" => TokenKind::Using,
            "as" => TokenKind::As,
            "true" => TokenKind::BoolLiteral(true),
            "false" => TokenKind::BoolLiteral(false),
            _ => TokenKind::Identifier(ident.to_string()),
        };
        self.push_token(token, span);
        Ok(())
    }

    fn lex_string(&mut self) -> Result<()> {
        let start = self.cursor;
        self.bump_char(); // opening quote
        let mut value = String::new();

        while let Some(ch) = self.peek_char() {
            match ch {
                '"' => {
                    self.bump_char();
                    let span = (start, self.cursor);
                    self.push_token(TokenKind::StringLiteral(value), span);
                    return Ok(());
                }
                '\\' => {
                    self.bump_char();
                    let escaped = self.peek_char().ok_or_else(|| {
                        Error::lex(
                            Some((self.cursor, self.cursor)),
                            "unterminated escape sequence",
                        )
                    })?;
                    self.bump_char();
                    let translated = match escaped {
                        '\\' => '\\',
                        '"' => '"',
                        'n' => '\n',
                        'r' => '\r',
                        't' => '\t',
                        '0' => '\0',
                        other => {
                            return Err(Error::lex(
                                Some((self.cursor, self.cursor)),
                                format!("unsupported escape character '{}'", other),
                            ));
                        }
                    };
                    value.push(translated);
                }
                _ => {
                    self.bump_char();
                    value.push(ch);
                }
            }
        }

        Err(Error::lex(
            Some((start, self.cursor)),
            "unterminated string literal",
        ))
    }

    fn skip_line_comment(&mut self) {
        while let Some(ch) = self.peek_char() {
            if ch == '\n' {
                break;
            }
            self.bump_char();
        }
    }

    fn consume_digits(&mut self) {
        while matches!(self.peek_char(), Some(c) if c.is_ascii_digit() || c == '_') {
            self.bump_char();
        }
    }

    fn simple_token(&mut self, kind: TokenKind) {
        let span = self.span_start();
        self.bump_char();
        self.push_token(kind, span);
    }

    fn span_start(&self) -> (usize, usize) {
        (self.cursor, self.cursor)
    }

    fn push_token(&mut self, kind: TokenKind, span: (usize, usize)) {
        let end = self.cursor;
        self.tokens.push(Token::new(kind, (span.0, end)));
    }

    fn peek_char(&self) -> Option<char> {
        self.input[self.cursor..].chars().next()
    }

    fn peek_next_char(&self) -> Option<char> {
        let mut iter = self.input[self.cursor..].chars();
        iter.next()?;
        iter.next()
    }

    fn bump_char(&mut self) -> Option<char> {
        let mut iter = self.input[self.cursor..].char_indices();
        let (_, ch) = iter.next()?;
        self.cursor += ch.len_utf8();
        Some(ch)
    }
}
