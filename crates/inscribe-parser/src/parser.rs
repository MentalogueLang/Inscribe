use std::fmt;

use inscribe_ast::nodes::{
    Block, ConstStmt, EnumDecl, EnumVariant, Expr, ExprKind, ForStmt, FunctionDecl, Import, Item,
    LetStmt, Literal, MatchArm, Module, Param, Path, Pattern, PatternKind, ReturnStmt, Stmt,
    StructDecl, StructField, StructLiteralField, TypeRef, TypeRefKind, UnaryOp, Visibility,
    WhileStmt,
};
use inscribe_ast::span::{Position, Span};
use inscribe_lexer::token::{Span as TokenSpan, Token, TokenKind};

use crate::expr::{binary_operator, Associativity, Precedence};
use crate::item::starts_item;
use crate::recovery::is_recovery_boundary;
use crate::stmt::{is_statement_boundary, starts_statement};

// TODO: Add multi-error recovery once the diagnostics pipeline is in place.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub message: String,
    pub span: Span,
}

impl ParseError {
    pub fn new(message: impl Into<String>, span: Span) -> Self {
        Self {
            message: message.into(),
            span,
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} at line {}, column {}",
            self.message, self.span.start.line, self.span.start.column
        )
    }
}

impl std::error::Error for ParseError {}

pub fn parse_module(tokens: Vec<Token>) -> Result<Module, ParseError> {
    Parser::new(tokens).parse_module()
}

#[derive(Debug, Clone)]
pub struct Parser {
    tokens: Vec<Token>,
    index: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, index: 0 }
    }

    pub fn parse_module(&mut self) -> Result<Module, ParseError> {
        self.skip_separators();
        let start = self.current_span().start;
        let mut items = Vec::new();

        while !self.at_eof() {
            if !starts_item(self.peek()) {
                return Err(self.error_here("expected a top-level item"));
            }

            items.push(self.parse_item()?);
            self.skip_top_level_separators()?;
        }

        let end = self.current_span().end;
        Ok(Module {
            items,
            span: Span::new(start, end),
        })
    }

    fn parse_item(&mut self) -> Result<Item, ParseError> {
        match self.peek() {
            TokenKind::Import => self.parse_import().map(Item::Import),
            TokenKind::Struct => self.parse_struct().map(Item::Struct),
            TokenKind::Enum => self.parse_enum().map(Item::Enum),
            TokenKind::Fn => self
                .parse_function(Visibility::Public, None)
                .map(Item::Function),
            TokenKind::Private => self.parse_private_function().map(Item::Function),
            _ => Err(self.error_here("expected `import`, `struct`, `enum`, `priv`, or `fn`")),
        }
    }

    fn parse_private_function(&mut self) -> Result<FunctionDecl, ParseError> {
        let start = convert_position(self.expect_simple(TokenKind::Private)?.span.start);
        self.parse_function(Visibility::Private, Some(start))
    }

    fn parse_import(&mut self) -> Result<Import, ParseError> {
        let start = self.expect_simple(TokenKind::Import)?.span.start;
        let path = self.parse_path()?;
        Ok(Import {
            path: path.clone(),
            span: Span::new(convert_position(start), path.span.end),
        })
    }

    fn parse_struct(&mut self) -> Result<StructDecl, ParseError> {
        let start = self.expect_simple(TokenKind::Struct)?.span.start;
        let (name, name_span) = self.expect_identifier()?;
        self.expect_simple(TokenKind::LBrace)?;
        self.skip_separators();

        let mut fields = Vec::new();
        while !self.check(TokenKind::RBrace) && !self.at_eof() {
            let field_start = self.current_span().start;
            let (field_name, field_name_span) = self.expect_identifier()?;
            self.expect_simple(TokenKind::Colon)?;
            let ty = self.parse_type()?;
            let span = Span::new(field_start, ty.span.end);
            fields.push(StructField {
                name: field_name,
                name_span: field_name_span,
                ty,
                span,
            });

            if self.check(TokenKind::RBrace) {
                break;
            }
            self.consume_list_separator("struct field")?;
        }

        let end = self.expect_simple(TokenKind::RBrace)?.span.end;
        Ok(StructDecl {
            name,
            name_span,
            fields,
            span: Span::new(convert_position(start), convert_position(end)),
        })
    }

    fn parse_enum(&mut self) -> Result<EnumDecl, ParseError> {
        let start = self.expect_simple(TokenKind::Enum)?.span.start;
        let (name, name_span) = self.expect_identifier()?;
        self.expect_simple(TokenKind::LBrace)?;
        self.skip_separators();

        let mut variants = Vec::new();
        while !self.check(TokenKind::RBrace) && !self.at_eof() {
            let variant_start = self.current_span().start;
            let (variant_name, variant_name_span) = self.expect_identifier()?;
            variants.push(EnumVariant {
                name: variant_name,
                name_span: variant_name_span,
                span: Span::new(variant_start, variant_name_span.end),
            });

            if self.check(TokenKind::RBrace) {
                break;
            }
            self.consume_list_separator("enum variant")?;
        }

        let end = self.expect_simple(TokenKind::RBrace)?.span.end;
        Ok(EnumDecl {
            name,
            name_span,
            variants,
            span: Span::new(convert_position(start), convert_position(end)),
        })
    }

    fn parse_function(
        &mut self,
        visibility: Visibility,
        start_override: Option<Position>,
    ) -> Result<FunctionDecl, ParseError> {
        let start = start_override.unwrap_or_else(|| convert_position(self.current().span.start));
        let _ = self.expect_simple(TokenKind::Fn)?;
        let name_path = self.parse_path()?;
        let (receiver, name, name_span) = split_function_name(name_path)?;
        self.expect_simple(TokenKind::LParen)?;
        let params = self.parse_params()?;
        self.expect_simple(TokenKind::RParen)?;
        let return_type = if self.match_simple(TokenKind::Arrow) {
            Some(self.parse_type()?)
        } else {
            None
        };
        let body = if self.check(TokenKind::LBrace) {
            Some(self.parse_block()?)
        } else {
            None
        };

        let end = body
            .as_ref()
            .map(|block| block.span.end)
            .or_else(|| return_type.as_ref().map(|ty| ty.span.end))
            .unwrap_or_else(|| self.previous_span().end);

        Ok(FunctionDecl {
            visibility,
            receiver,
            name,
            name_span,
            params,
            return_type,
            body,
            span: Span::new(start, end),
        })
    }

    fn parse_params(&mut self) -> Result<Vec<Param>, ParseError> {
        let mut params = Vec::new();
        self.skip_inline_separators();
        while !self.check(TokenKind::RParen) && !self.at_eof() {
            let start = self.current_span().start;
            let (name, name_span) = self.expect_identifier()?;
            let ty = if self.match_simple(TokenKind::Colon) {
                Some(self.parse_type()?)
            } else {
                None
            };
            let end = ty
                .as_ref()
                .map(|ty| ty.span.end)
                .unwrap_or_else(|| self.previous_span().end);
            params.push(Param {
                name,
                name_span,
                ty,
                span: Span::new(start, end),
            });

            self.skip_inline_separators();
            if self.check(TokenKind::RParen) {
                break;
            }
            self.expect_simple(TokenKind::Comma)?;
            self.skip_inline_separators();
        }
        Ok(params)
    }

    fn parse_type(&mut self) -> Result<TypeRef, ParseError> {
        if self.check(TokenKind::LBracket) {
            let start = self.expect_simple(TokenKind::LBracket)?.span.start;
            let element = self.parse_type()?;
            self.expect_simple(TokenKind::Semicolon)?;
            let length = self.expect_array_length()?;
            let end = self.expect_simple(TokenKind::RBracket)?.span.end;
            return Ok(TypeRef {
                kind: TypeRefKind::Array {
                    element: Box::new(element),
                    length,
                },
                span: Span::new(convert_position(start), convert_position(end)),
            });
        }

        let path = self.parse_path()?;
        let mut arguments = Vec::new();

        if self.match_simple(TokenKind::Less) {
            self.skip_inline_separators();
            while !self.check(TokenKind::Greater) && !self.at_eof() {
                arguments.push(self.parse_type()?);
                self.skip_inline_separators();
                if self.check(TokenKind::Greater) {
                    break;
                }
                self.expect_simple(TokenKind::Comma)?;
                self.skip_inline_separators();
            }
            self.expect_simple(TokenKind::Greater)?;
        }

        let end = if let Some(last) = arguments.last() {
            last.span.end
        } else {
            path.span.end
        };

        Ok(TypeRef {
            kind: TypeRefKind::Path {
                path: path.clone(),
                arguments,
            },
            span: Span::new(path.span.start, end),
        })
    }

    fn parse_block(&mut self) -> Result<Block, ParseError> {
        let start = self.expect_simple(TokenKind::LBrace)?.span.start;
        self.skip_separators();
        let mut statements = Vec::new();

        while !self.check(TokenKind::RBrace) && !self.at_eof() {
            if !starts_statement(self.peek()) {
                return Err(self.error_here("expected a statement"));
            }

            statements.push(self.parse_statement()?);

            if self.check(TokenKind::RBrace) {
                break;
            }

            if !self.consume_statement_separator() {
                return Err(self.error_here("expected a newline, `;`, or `}` after a statement"));
            }

            self.skip_separators();
        }

        let end = self.expect_simple(TokenKind::RBrace)?.span.end;
        Ok(Block {
            statements,
            span: Span::new(convert_position(start), convert_position(end)),
        })
    }

    fn parse_statement(&mut self) -> Result<Stmt, ParseError> {
        match self.peek() {
            TokenKind::Let => self.parse_let().map(Stmt::Let),
            TokenKind::Const => self.parse_const().map(Stmt::Const),
            TokenKind::For => self.parse_for().map(Stmt::For),
            TokenKind::While => self.parse_while().map(Stmt::While),
            TokenKind::Return => self.parse_return().map(Stmt::Return),
            _ => self
                .parse_expression(Precedence::Assignment.level())
                .map(Stmt::Expr),
        }
    }

    fn parse_let(&mut self) -> Result<LetStmt, ParseError> {
        let start = self.expect_simple(TokenKind::Let)?.span.start;
        let (name, name_span) = self.expect_identifier()?;
        let ty = if self.match_simple(TokenKind::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };
        self.expect_simple(TokenKind::Equal)?;
        let value = self.parse_expression(Precedence::Assignment.level())?;
        Ok(LetStmt {
            name,
            name_span,
            ty,
            span: Span::new(convert_position(start), value.span.end),
            value,
        })
    }

    fn parse_const(&mut self) -> Result<ConstStmt, ParseError> {
        let start = self.expect_simple(TokenKind::Const)?.span.start;
        let (name, name_span) = self.expect_identifier()?;
        let ty = if self.match_simple(TokenKind::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };
        self.expect_simple(TokenKind::Equal)?;
        let value = self.parse_expression(Precedence::Assignment.level())?;
        Ok(ConstStmt {
            name,
            name_span,
            ty,
            span: Span::new(convert_position(start), value.span.end),
            value,
        })
    }

    fn parse_for(&mut self) -> Result<ForStmt, ParseError> {
        let start = self.expect_simple(TokenKind::For)?.span.start;
        let pattern = self.parse_pattern()?;
        self.expect_simple(TokenKind::In)?;
        let iterable = self.parse_header_expression()?;
        let body = self.parse_block()?;
        Ok(ForStmt {
            pattern,
            iterable,
            body: body.clone(),
            span: Span::new(convert_position(start), body.span.end),
        })
    }

    fn parse_while(&mut self) -> Result<WhileStmt, ParseError> {
        let start = self.expect_simple(TokenKind::While)?.span.start;
        let condition = self.parse_header_expression()?;
        let body = self.parse_block()?;
        Ok(WhileStmt {
            condition,
            body: body.clone(),
            span: Span::new(convert_position(start), body.span.end),
        })
    }

    fn parse_return(&mut self) -> Result<ReturnStmt, ParseError> {
        let start = self.expect_simple(TokenKind::Return)?.span.start;
        if is_statement_boundary(self.peek()) {
            return Ok(ReturnStmt {
                value: None,
                span: Span::new(convert_position(start), self.previous_span().end),
            });
        }

        let value = self.parse_expression(Precedence::Assignment.level())?;
        Ok(ReturnStmt {
            value: Some(value.clone()),
            span: Span::new(convert_position(start), value.span.end),
        })
    }

    fn parse_expression(&mut self, min_precedence: u8) -> Result<Expr, ParseError> {
        self.parse_expression_with_options(min_precedence, true)
    }

    fn parse_header_expression(&mut self) -> Result<Expr, ParseError> {
        self.parse_expression_with_options(Precedence::Assignment.level(), false)
    }

    fn parse_expression_with_options(
        &mut self,
        min_precedence: u8,
        allow_trailing_struct_literal: bool,
    ) -> Result<Expr, ParseError> {
        let mut expr = self.parse_prefix()?;

        loop {
            expr = match self.peek() {
                TokenKind::LParen => self.finish_call(expr)?,
                TokenKind::Dot => self.finish_field(expr)?,
                TokenKind::LBracket => self.finish_index(expr)?,
                TokenKind::Question => self.finish_try(expr),
                TokenKind::LBrace
                    if allow_trailing_struct_literal && matches!(expr.kind, ExprKind::Path(_)) =>
                {
                    self.finish_struct_literal(expr)?
                }
                kind => {
                    let Some((precedence, associativity, op)) = binary_operator(kind) else {
                        break;
                    };
                    if precedence.level() < min_precedence {
                        break;
                    }

                    self.advance();
                    let next_min = match associativity {
                        Associativity::Left => precedence.level() + 1,
                        Associativity::Right => precedence.level(),
                    };
                    let right = self.parse_expression(next_min)?;
                    let span = Span::new(expr.span.start, right.span.end);
                    Expr::new(
                        ExprKind::Binary {
                            op,
                            left: Box::new(expr),
                            right: Box::new(right),
                        },
                        span,
                    )
                }
            };
        }

        Ok(expr)
    }

    fn parse_prefix(&mut self) -> Result<Expr, ParseError> {
        match self.peek() {
            TokenKind::Minus => {
                let start = self.advance().span.start;
                let expr = self.parse_expression(Precedence::Factor.level())?;
                Ok(Expr::new(
                    ExprKind::Unary {
                        op: UnaryOp::Negate,
                        expr: Box::new(expr.clone()),
                    },
                    Span::new(convert_position(start), expr.span.end),
                ))
            }
            TokenKind::Bang => {
                let start = self.advance().span.start;
                let expr = self.parse_expression(Precedence::Factor.level())?;
                Ok(Expr::new(
                    ExprKind::Unary {
                        op: UnaryOp::Not,
                        expr: Box::new(expr.clone()),
                    },
                    Span::new(convert_position(start), expr.span.end),
                ))
            }
            TokenKind::If => self.parse_if_expression(),
            TokenKind::Match => self.parse_match_expression(),
            TokenKind::LBrace => self
                .parse_block()
                .map(|block| Expr::new(ExprKind::Block(block.clone()), block.span)),
            TokenKind::LBracket => self.parse_array_expression(),
            TokenKind::LParen => {
                self.advance();
                let expr = self.parse_expression(Precedence::Assignment.level())?;
                self.expect_simple(TokenKind::RParen)?;
                Ok(expr)
            }
            TokenKind::Integer(_) | TokenKind::Float(_) | TokenKind::String(_) => {
                self.parse_literal_expression()
            }
            TokenKind::True | TokenKind::False => self.parse_bool_expression(),
            TokenKind::Identifier(_) => self.parse_path_expression(),
            _ => Err(self.error_here("expected an expression")),
        }
    }

    fn parse_if_expression(&mut self) -> Result<Expr, ParseError> {
        let start = self.expect_simple(TokenKind::If)?.span.start;
        let condition = self.parse_header_expression()?;
        let then_block = self.parse_block()?;
        let else_branch = if self.match_simple(TokenKind::Else) {
            Some(Box::new(if self.check(TokenKind::If) {
                self.parse_if_expression()?
            } else if self.check(TokenKind::LBrace) {
                let block = self.parse_block()?;
                Expr::new(ExprKind::Block(block.clone()), block.span)
            } else {
                return Err(self.error_here("expected `if` or a block after `else`"));
            }))
        } else {
            None
        };

        let end = else_branch
            .as_ref()
            .map(|expr| expr.span.end)
            .unwrap_or(then_block.span.end);

        Ok(Expr::new(
            ExprKind::If {
                condition: Box::new(condition),
                then_block,
                else_branch,
            },
            Span::new(convert_position(start), end),
        ))
    }

    fn parse_match_expression(&mut self) -> Result<Expr, ParseError> {
        let start = self.expect_simple(TokenKind::Match)?.span.start;
        let value = self.parse_header_expression()?;
        self.expect_simple(TokenKind::LBrace)?;
        self.skip_separators();

        let mut arms = Vec::new();
        while !self.check(TokenKind::RBrace) && !self.at_eof() {
            let arm_start = self.current_span().start;
            let pattern = self.parse_pattern()?;
            self.expect_simple(TokenKind::FatArrow)?;
            let value_expr = self.parse_expression(Precedence::Assignment.level())?;
            arms.push(MatchArm {
                pattern,
                value: value_expr.clone(),
                span: Span::new(arm_start, value_expr.span.end),
            });

            if self.check(TokenKind::RBrace) {
                break;
            }
            self.consume_list_separator("match arm")?;
        }

        let end = self.expect_simple(TokenKind::RBrace)?.span.end;
        Ok(Expr::new(
            ExprKind::Match {
                value: Box::new(value),
                arms,
            },
            Span::new(convert_position(start), convert_position(end)),
        ))
    }

    fn parse_pattern(&mut self) -> Result<Pattern, ParseError> {
        match self.peek() {
            TokenKind::Identifier(name) if name == "_" => {
                let token = self.advance().clone();
                Ok(Pattern::new(
                    PatternKind::Wildcard,
                    convert_span(token.span),
                ))
            }
            TokenKind::Integer(_) | TokenKind::Float(_) | TokenKind::String(_) => {
                let expr = self.parse_literal_expression()?;
                if let ExprKind::Literal(literal) = expr.kind {
                    Ok(Pattern::new(PatternKind::Literal(literal), expr.span))
                } else {
                    unreachable!("literal parser returned a non-literal expression")
                }
            }
            TokenKind::True | TokenKind::False => {
                let expr = self.parse_bool_expression()?;
                if let ExprKind::Literal(literal) = expr.kind {
                    Ok(Pattern::new(PatternKind::Literal(literal), expr.span))
                } else {
                    unreachable!("bool parser returned a non-literal expression")
                }
            }
            TokenKind::Identifier(_) => {
                let path = self.parse_path()?;
                if self.match_simple(TokenKind::LParen) {
                    let mut arguments = Vec::new();
                    self.skip_inline_separators();
                    while !self.check(TokenKind::RParen) && !self.at_eof() {
                        arguments.push(self.parse_pattern()?);
                        self.skip_inline_separators();
                        if self.check(TokenKind::RParen) {
                            break;
                        }
                        self.expect_simple(TokenKind::Comma)?;
                        self.skip_inline_separators();
                    }
                    let end = self.expect_simple(TokenKind::RParen)?.span.end;
                    Ok(Pattern::new(
                        PatternKind::Constructor {
                            path: path.clone(),
                            arguments,
                        },
                        Span::new(path.span.start, convert_position(end)),
                    ))
                } else if path.segments.len() == 1 {
                    Ok(Pattern::new(
                        PatternKind::Binding(path.segments[0].clone()),
                        path.span,
                    ))
                } else {
                    Ok(Pattern::new(PatternKind::Path(path.clone()), path.span))
                }
            }
            _ => Err(self.error_here("expected a match pattern")),
        }
    }

    fn parse_literal_expression(&mut self) -> Result<Expr, ParseError> {
        let token = self.advance().clone();
        let literal = match token.kind {
            TokenKind::Integer(value) => Literal::Integer(value),
            TokenKind::Float(value) => Literal::Float(value),
            TokenKind::String(value) => Literal::String(value),
            _ => {
                return Err(ParseError::new(
                    "expected a literal",
                    convert_span(token.span),
                ))
            }
        };
        Ok(Expr::new(
            ExprKind::Literal(literal),
            convert_span(token.span),
        ))
    }

    fn parse_bool_expression(&mut self) -> Result<Expr, ParseError> {
        let token = self.advance().clone();
        let literal = match token.kind {
            TokenKind::True => Literal::Bool(true),
            TokenKind::False => Literal::Bool(false),
            _ => {
                return Err(ParseError::new(
                    "expected a boolean literal",
                    convert_span(token.span),
                ))
            }
        };
        Ok(Expr::new(
            ExprKind::Literal(literal),
            convert_span(token.span),
        ))
    }

    fn parse_path_expression(&mut self) -> Result<Expr, ParseError> {
        let start = self.current_span().start;
        let (name, span) = self.expect_identifier()?;
        let path = Path::with_segment_spans(vec![name], vec![span], Span::new(start, span.end));
        Ok(Expr::new(ExprKind::Path(path.clone()), path.span))
    }

    fn parse_array_expression(&mut self) -> Result<Expr, ParseError> {
        let start = self.expect_simple(TokenKind::LBracket)?.span.start;
        self.skip_inline_separators();
        if self.check(TokenKind::RBracket) {
            return Err(self.error_here("array literals must contain at least one element"));
        }

        let first = self.parse_expression(Precedence::Assignment.level())?;
        self.skip_inline_separators();

        if self.match_simple(TokenKind::Semicolon) {
            self.skip_inline_separators();
            let length = self.expect_array_length()?;
            let end = self.expect_simple(TokenKind::RBracket)?.span.end;
            return Ok(Expr::new(
                ExprKind::RepeatArray {
                    value: Box::new(first),
                    length,
                },
                Span::new(convert_position(start), convert_position(end)),
            ));
        }

        let mut items = vec![first];
        while !self.check(TokenKind::RBracket) && !self.at_eof() {
            self.expect_simple(TokenKind::Comma)?;
            self.skip_inline_separators();
            if self.check(TokenKind::RBracket) {
                break;
            }
            items.push(self.parse_expression(Precedence::Assignment.level())?);
            self.skip_inline_separators();
        }
        let end = self.expect_simple(TokenKind::RBracket)?.span.end;
        Ok(Expr::new(
            ExprKind::Array(items),
            Span::new(convert_position(start), convert_position(end)),
        ))
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr, ParseError> {
        let start = callee.span.start;
        self.expect_simple(TokenKind::LParen)?;
        let mut args = Vec::new();
        self.skip_inline_separators();
        while !self.check(TokenKind::RParen) && !self.at_eof() {
            args.push(self.parse_expression(Precedence::Assignment.level())?);
            self.skip_inline_separators();
            if self.check(TokenKind::RParen) {
                break;
            }
            self.expect_simple(TokenKind::Comma)?;
            self.skip_inline_separators();
        }
        let end = self.expect_simple(TokenKind::RParen)?.span.end;
        Ok(Expr::new(
            ExprKind::Call {
                callee: Box::new(callee),
                args,
            },
            Span::new(start, convert_position(end)),
        ))
    }

    fn finish_field(&mut self, base: Expr) -> Result<Expr, ParseError> {
        let start = base.span.start;
        self.expect_simple(TokenKind::Dot)?;
        let (field, span) = self.expect_identifier()?;
        Ok(Expr::new(
            ExprKind::Field {
                base: Box::new(base),
                field,
            },
            Span::new(start, span.end),
        ))
    }

    fn finish_index(&mut self, target: Expr) -> Result<Expr, ParseError> {
        let start = target.span.start;
        self.expect_simple(TokenKind::LBracket)?;
        let index = self.parse_expression(Precedence::Assignment.level())?;
        let end = self.expect_simple(TokenKind::RBracket)?.span.end;
        Ok(Expr::new(
            ExprKind::Index {
                target: Box::new(target),
                index: Box::new(index),
            },
            Span::new(start, convert_position(end)),
        ))
    }

    fn finish_struct_literal(&mut self, base: Expr) -> Result<Expr, ParseError> {
        let ExprKind::Path(path) = &base.kind else {
            return Err(self.error_here("expected a type name before a struct literal"));
        };
        self.expect_simple(TokenKind::LBrace)?;
        self.skip_separators();
        let mut fields = Vec::new();
        while !self.check(TokenKind::RBrace) && !self.at_eof() {
            let start = self.current_span().start;
            let (name, name_span) = self.expect_identifier()?;
            self.expect_simple(TokenKind::Colon)?;
            let value = self.parse_expression(Precedence::Assignment.level())?;
            fields.push(StructLiteralField {
                name,
                name_span,
                value: value.clone(),
                span: Span::new(start, value.span.end),
            });

            if self.check(TokenKind::RBrace) {
                break;
            }
            self.consume_list_separator("struct literal field")?;
        }
        let end = self.expect_simple(TokenKind::RBrace)?.span.end;
        Ok(Expr::new(
            ExprKind::StructLiteral {
                path: path.clone(),
                fields,
            },
            Span::new(base.span.start, convert_position(end)),
        ))
    }

    fn finish_try(&mut self, expr: Expr) -> Expr {
        let _ = self.advance();
        let span = Span::new(expr.span.start, self.previous_span().end);
        Expr::new(ExprKind::Try(Box::new(expr)), span)
    }

    fn parse_path(&mut self) -> Result<Path, ParseError> {
        let start = self.current_span().start;
        let mut segments = Vec::new();
        let mut segment_spans = Vec::new();
        let (first, first_span) = self.expect_identifier()?;
        segments.push(first);
        segment_spans.push(first_span);
        while self.match_simple(TokenKind::Dot) {
            let (segment, segment_span) = self.expect_identifier()?;
            segments.push(segment);
            segment_spans.push(segment_span);
        }
        Ok(Path::with_segment_spans(
            segments,
            segment_spans,
            Span::new(start, self.previous_span().end),
        ))
    }

    fn expect_array_length(&mut self) -> Result<usize, ParseError> {
        match &self.current().kind {
            TokenKind::Integer(value) => {
                let parsed = value.parse::<usize>().map_err(|_| {
                    ParseError::new(
                        format!("invalid array length `{value}`"),
                        convert_span(self.current().span),
                    )
                })?;
                let _ = self.advance();
                Ok(parsed)
            }
            _ => Err(self.error_here("expected an integer array length")),
        }
    }

    fn skip_top_level_separators(&mut self) -> Result<(), ParseError> {
        if self.at_eof() {
            return Ok(());
        }
        if self.consume_statement_separator() {
            self.skip_separators();
            return Ok(());
        }
        if starts_item(self.peek()) {
            return Ok(());
        }
        Err(self.error_here("expected a newline or end of file after the item"))
    }

    fn consume_statement_separator(&mut self) -> bool {
        let mut consumed = false;
        while matches!(self.peek(), TokenKind::Newline | TokenKind::Semicolon) {
            self.advance();
            consumed = true;
        }
        consumed
    }

    fn consume_list_separator(&mut self, context: &str) -> Result<(), ParseError> {
        if self.match_simple(TokenKind::Comma) {
            self.skip_separators();
            return Ok(());
        }
        if self.consume_statement_separator() {
            self.skip_separators();
            return Ok(());
        }
        Err(self.error_here(format!("expected a separator after {context}")))
    }

    fn skip_separators(&mut self) {
        while matches!(self.peek(), TokenKind::Newline | TokenKind::Semicolon) {
            self.advance();
        }
    }

    fn skip_inline_separators(&mut self) {
        while matches!(self.peek(), TokenKind::Newline) {
            self.advance();
        }
    }

    fn expect_identifier(&mut self) -> Result<(String, Span), ParseError> {
        match &self.current().kind {
            TokenKind::Identifier(value) => {
                let value = value.clone();
                let span = convert_span(self.advance().span);
                Ok((value, span))
            }
            _ => Err(self.error_here("expected an identifier")),
        }
    }

    fn expect_simple(&mut self, expected: TokenKind) -> Result<&Token, ParseError> {
        if self.check(expected.clone()) {
            Ok(self.advance())
        } else {
            Err(ParseError::new(
                format!("expected {}", expected.describe()),
                self.current_span(),
            ))
        }
    }

    fn match_simple(&mut self, expected: TokenKind) -> bool {
        if self.check(expected) {
            let _ = self.advance();
            true
        } else {
            false
        }
    }

    fn check(&self, expected: TokenKind) -> bool {
        std::mem::discriminant(self.peek()) == std::mem::discriminant(&expected)
    }

    fn peek(&self) -> &TokenKind {
        &self.current().kind
    }

    fn current(&self) -> &Token {
        &self.tokens[self.index]
    }

    fn current_span(&self) -> Span {
        convert_span(self.current().span)
    }

    fn previous_span(&self) -> Span {
        if self.index == 0 {
            self.current_span()
        } else {
            convert_span(self.tokens[self.index - 1].span)
        }
    }

    fn advance(&mut self) -> &Token {
        if !self.at_eof() {
            self.index += 1;
        }
        &self.tokens[self.index.saturating_sub(1)]
    }

    fn at_eof(&self) -> bool {
        matches!(self.peek(), TokenKind::Eof)
    }

    fn error_here(&mut self, message: impl Into<String>) -> ParseError {
        let error = ParseError::new(message, self.current_span());
        if !is_recovery_boundary(self.peek()) && !self.at_eof() {
            let _ = self.advance();
        }
        error
    }
}

fn split_function_name(path: Path) -> Result<(Option<Path>, String, Span), ParseError> {
    let mut segments = path.segments;
    let mut segment_spans = path.segment_spans;
    let name = segments
        .pop()
        .ok_or_else(|| ParseError::new("expected a function name", path.span))?;
    let name_span = segment_spans
        .pop()
        .ok_or_else(|| ParseError::new("expected a function name", path.span))?;
    let receiver = if segments.is_empty() {
        None
    } else {
        let receiver_end = segment_spans.last().map(|span| span.end).unwrap_or(path.span.end);
        Some(Path::with_segment_spans(
            segments,
            segment_spans,
            Span::new(path.span.start, receiver_end),
        ))
    };
    Ok((receiver, name, name_span))
}

fn convert_span(span: TokenSpan) -> Span {
    Span::new(convert_position(span.start), convert_position(span.end))
}

fn convert_position(position: inscribe_lexer::token::Position) -> Position {
    Position::new(position.offset, position.line, position.column)
}
