use inscribe_ast::nodes::{
    Expr, ExprKind, FunctionDecl, Item, Literal, Module, Pattern, PatternKind, Stmt,
};
use inscribe_resolve::{FunctionKey, ResolvedProgram};
use inscribe_typeck::{expr_key, FunctionSignature, Type, TypeCheckResult};

use crate::nodes::{
    HirBinding, HirBlock, HirEnum, HirExpr, HirExprKind, HirField, HirFor, HirFunction,
    HirImport, HirItem, HirMatchArm, HirParam, HirProgram, HirStmt, HirStruct, HirWhile,
};

pub fn lower_module(
    module: &Module,
    resolved: &ResolvedProgram,
    typed: &TypeCheckResult,
) -> HirProgram {
    let items = module
        .items
        .iter()
        .map(|item| lower_item(item, resolved, typed))
        .collect();

    HirProgram {
        items,
        span: module.span,
    }
}

fn lower_item(item: &Item, resolved: &ResolvedProgram, typed: &TypeCheckResult) -> HirItem {
    match item {
        Item::Import(import) => HirItem::Import(HirImport {
            path: import.path.segments.clone(),
            span: import.span,
        }),
        Item::Struct(decl) => HirItem::Struct(HirStruct {
            name: decl.name.clone(),
            fields: decl
                .fields
                .iter()
                .map(|field| HirField {
                    name: field.name.clone(),
                    ty: resolved
                        .structs
                        .get(&decl.name)
                        .and_then(|info| info.fields.get(&field.name))
                        .map(|ty| type_from_resolved_name(resolved, ty))
                        .unwrap_or(Type::Unknown),
                    span: field.span,
                })
                .collect(),
            span: decl.span,
        }),
        Item::Enum(decl) => HirItem::Enum(HirEnum {
            name: decl.name.clone(),
            variants: resolved
                .enums
                .get(&decl.name)
                .map(|info| {
                    decl.variants
                        .iter()
                        .filter_map(|variant| {
                            info.variants
                                .get(&variant.name)
                                .map(|discriminant| (variant.name.clone(), *discriminant))
                        })
                        .collect()
                })
                .unwrap_or_default(),
            span: decl.span,
        }),
        Item::Function(function) => HirItem::Function(lower_function(function, resolved, typed)),
    }
}

fn lower_function(
    function: &FunctionDecl,
    resolved: &ResolvedProgram,
    typed: &TypeCheckResult,
) -> HirFunction {
    let key = FunctionKey {
        receiver: function
            .receiver
            .as_ref()
            .map(|path| path.segments.join(".")),
        name: function.name.clone(),
    };
    let signature = typed
        .function_signatures
        .get(&key)
        .cloned()
        .unwrap_or(FunctionSignature {
            key: key.clone(),
            params: Vec::new(),
            return_type: Box::new(Type::Unknown),
        });

    let params = function
        .params
        .iter()
        .enumerate()
        .map(|(index, param)| HirParam {
            name: param.name.clone(),
            ty: signature
                .params
                .get(index)
                .cloned()
                .unwrap_or(Type::Unknown),
            span: param.span,
        })
        .collect();

    HirFunction {
        visibility: function.visibility,
        receiver: key.receiver.clone(),
        name: key.name,
        signature,
        params,
        is_declaration: function.body.is_none(),
        body: function
            .body
            .as_ref()
            .map(|body| lower_block(body, resolved, typed)),
        span: function.span,
    }
}

fn lower_block(
    block: &inscribe_ast::nodes::Block,
    resolved: &ResolvedProgram,
    typed: &TypeCheckResult,
) -> HirBlock {
    let statements = block
        .statements
        .iter()
        .map(|statement| lower_statement(statement, resolved, typed))
        .collect::<Vec<_>>();
    let ty = statements.last().map(statement_type).unwrap_or(Type::Unit);

    HirBlock {
        statements,
        ty,
        span: block.span,
    }
}

fn lower_statement(statement: &Stmt, resolved: &ResolvedProgram, typed: &TypeCheckResult) -> HirStmt {
    match statement {
        Stmt::Let(stmt) => HirStmt::Let(HirBinding {
            name: stmt.name.clone(),
            ty: lookup_type(typed, stmt.value.span),
            value: lower_expr(&stmt.value, resolved, typed),
            span: stmt.span,
        }),
        Stmt::Const(stmt) => HirStmt::Const(HirBinding {
            name: stmt.name.clone(),
            ty: lookup_type(typed, stmt.value.span),
            value: lower_expr(&stmt.value, resolved, typed),
            span: stmt.span,
        }),
        Stmt::For(stmt) => HirStmt::For(HirFor {
            binding: pattern_display(&stmt.pattern),
            binding_ty: lookup_type(typed, stmt.iterable.span),
            iterable: lower_expr(&stmt.iterable, resolved, typed),
            body: lower_block(&stmt.body, resolved, typed),
            span: stmt.span,
        }),
        Stmt::While(stmt) => HirStmt::While(HirWhile {
            condition: lower_expr(&stmt.condition, resolved, typed),
            body: lower_block(&stmt.body, resolved, typed),
            span: stmt.span,
        }),
        Stmt::Return(value) => HirStmt::Return(
            value
                .value
                .as_ref()
                .map(|expr| lower_expr(expr, resolved, typed)),
            value.span,
        ),
        Stmt::Expr(expr) => HirStmt::Expr(lower_expr(expr, resolved, typed)),
    }
}

fn lower_expr(expr: &Expr, resolved: &ResolvedProgram, typed: &TypeCheckResult) -> HirExpr {
    let ty = lookup_type(typed, expr.span);
    let kind = match &expr.kind {
        ExprKind::Literal(literal) => HirExprKind::Literal(literal_display(literal)),
        ExprKind::Path(path) => lower_enum_variant_path(&path.segments, &ty, resolved)
            .unwrap_or_else(|| HirExprKind::Path(path.segments.clone())),
        ExprKind::Array(items) => {
            HirExprKind::Array(
                items
                    .iter()
                    .map(|item| lower_expr(item, resolved, typed))
                    .collect(),
            )
        }
        ExprKind::RepeatArray { value, length } => HirExprKind::RepeatArray {
            value: Box::new(lower_expr(value, resolved, typed)),
            length: *length,
        },
        ExprKind::Cast { expr, .. } => HirExprKind::Cast {
            expr: Box::new(lower_expr(expr, resolved, typed)),
        },
        ExprKind::Unary { op, expr } => HirExprKind::Unary {
            op: format!("{op:?}"),
            expr: Box::new(lower_expr(expr, resolved, typed)),
        },
        ExprKind::Binary { op, left, right } => lower_binary_expr(*op, left, right, resolved, typed),
        ExprKind::Call { callee, args } => HirExprKind::Call {
            callee: Box::new(lower_expr(callee, resolved, typed)),
            args: args
                .iter()
                .map(|arg| lower_expr(arg, resolved, typed))
                .collect(),
        },
        ExprKind::Field { base, field } => lower_enum_variant_field(base, field, &ty, resolved)
            .unwrap_or_else(|| HirExprKind::Field {
                base: Box::new(lower_expr(base, resolved, typed)),
                field: field.clone(),
            }),
        ExprKind::Index { target, index } => HirExprKind::Index {
            target: Box::new(lower_expr(target, resolved, typed)),
            index: Box::new(lower_expr(index, resolved, typed)),
        },
        ExprKind::StructLiteral { path, fields } => HirExprKind::StructLiteral {
            path: path.segments.clone(),
            fields: fields
                .iter()
                .map(|field| (field.name.clone(), lower_expr(&field.value, resolved, typed)))
                .collect(),
        },
        ExprKind::If {
            condition,
            then_block,
            else_branch,
        } => HirExprKind::If {
            condition: Box::new(lower_expr(condition, resolved, typed)),
            then_block: lower_block(then_block, resolved, typed),
            else_branch: else_branch
                .as_ref()
                .map(|expr| Box::new(lower_expr(expr, resolved, typed))),
        },
        ExprKind::Match { value, arms } => HirExprKind::Match {
            value: Box::new(lower_expr(value, resolved, typed)),
            arms: arms
                .iter()
                .map(|arm| HirMatchArm {
                    pattern: pattern_display(&arm.pattern),
                    value: lower_expr(&arm.value, resolved, typed),
                    span: arm.span,
                })
                .collect(),
        },
        ExprKind::Block(block) => HirExprKind::Block(lower_block(block, resolved, typed)),
        ExprKind::Try(inner) => HirExprKind::Try(Box::new(lower_expr(inner, resolved, typed))),
    };

    HirExpr {
        kind,
        ty,
        span: expr.span,
    }
}

fn lower_enum_variant_path(
    segments: &[String],
    ty: &Type,
    resolved: &ResolvedProgram,
) -> Option<HirExprKind> {
    let Type::Enum(enum_name) = ty else {
        return None;
    };
    let [head, variant] = segments else {
        return None;
    };
    if head != enum_name {
        return None;
    }
    let discriminant = resolved
        .enums
        .get(enum_name)
        .and_then(|info| info.variants.get(variant))
        .copied()?;
    Some(HirExprKind::EnumVariant {
        enum_name: enum_name.clone(),
        variant: variant.clone(),
        discriminant,
    })
}

fn lower_enum_variant_field(
    base: &Expr,
    field: &str,
    ty: &Type,
    resolved: &ResolvedProgram,
) -> Option<HirExprKind> {
    let ExprKind::Path(path) = &base.kind else {
        return None;
    };
    let [enum_name] = path.segments.as_slice() else {
        return None;
    };
    lower_enum_variant_path(&[enum_name.clone(), field.to_string()], ty, resolved)
}

fn lower_binary_expr(
    op: inscribe_ast::nodes::BinaryOp,
    left: &Expr,
    right: &Expr,
    resolved: &ResolvedProgram,
    typed: &TypeCheckResult,
) -> HirExprKind {
    match op {
        inscribe_ast::nodes::BinaryOp::And => HirExprKind::If {
            condition: Box::new(lower_expr(left, resolved, typed)),
            then_block: block_with_expr(lower_expr(right, resolved, typed), right.span),
            else_branch: Some(Box::new(bool_literal_expr(false, left.span))),
        },
        inscribe_ast::nodes::BinaryOp::Or => HirExprKind::If {
            condition: Box::new(lower_expr(left, resolved, typed)),
            then_block: block_with_expr(bool_literal_expr(true, left.span), left.span),
            else_branch: Some(Box::new(lower_expr(right, resolved, typed))),
        },
        _ => HirExprKind::Binary {
            op: format!("{op:?}"),
            left: Box::new(lower_expr(left, resolved, typed)),
            right: Box::new(lower_expr(right, resolved, typed)),
        },
    }
}

fn block_with_expr(expr: HirExpr, span: inscribe_ast::span::Span) -> HirBlock {
    let ty = expr.ty.clone();
    HirBlock {
        statements: vec![HirStmt::Expr(expr)],
        ty,
        span,
    }
}

fn bool_literal_expr(value: bool, span: inscribe_ast::span::Span) -> HirExpr {
    HirExpr {
        kind: HirExprKind::Literal(value.to_string()),
        ty: Type::Bool,
        span,
    }
}

fn statement_type(statement: &HirStmt) -> Type {
    match statement {
        HirStmt::Let(_) | HirStmt::Const(_) | HirStmt::For(_) | HirStmt::While(_) => Type::Unit,
        HirStmt::Return(Some(expr), _) | HirStmt::Expr(expr) => expr.ty.clone(),
        HirStmt::Return(None, _) => Type::Unit,
    }
}

fn lookup_type(typed: &TypeCheckResult, span: inscribe_ast::span::Span) -> Type {
    typed
        .expr_types
        .get(&expr_key(span))
        .cloned()
        .unwrap_or(Type::Unknown)
}

fn literal_display(literal: &Literal) -> String {
    match literal {
        Literal::Integer(value) => value.clone(),
        Literal::Float(value) => value.clone(),
        Literal::String(value) => value.clone(),
        Literal::Bool(value) => value.to_string(),
    }
}

fn pattern_display(pattern: &Pattern) -> String {
    match &pattern.kind {
        PatternKind::Wildcard => "_".to_string(),
        PatternKind::Binding(name) => name.clone(),
        PatternKind::Literal(literal) => literal_display(literal),
        PatternKind::Path(path) => path.segments.join("."),
        PatternKind::Constructor { path, arguments } => {
            let args = arguments
                .iter()
                .map(pattern_display)
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}({args})", path.segments.join("."))
        }
    }
}

fn type_from_resolved_name(resolved: &ResolvedProgram, name: &inscribe_resolve::TypeName) -> Type {
    match name {
        inscribe_resolve::TypeName::Named {
            path, arguments, ..
        } => {
            let head = path.last().cloned().unwrap_or_default();
            match head.as_str() {
                "int" => Type::Int,
                "byte" => Type::Byte,
                "float" => Type::Float,
                "string" => Type::String,
                "bool" => Type::Bool,
                "Error" => Type::Error,
                "Result" => {
                    let ok = arguments
                        .first()
                        .map(|argument| type_from_resolved_name(resolved, argument))
                        .unwrap_or(Type::Unknown);
                    let err = arguments
                        .get(1)
                        .map(|argument| type_from_resolved_name(resolved, argument))
                        .unwrap_or(Type::Error);
                    Type::Result(Box::new(ok), Box::new(err))
                }
                _ if resolved.structs.contains_key(&head) => Type::Struct(head),
                _ if resolved.enums.contains_key(&head) => Type::Enum(head),
                _ => Type::Unknown,
            }
        }
        inscribe_resolve::TypeName::Array {
            element, length, ..
        } => Type::Array(Box::new(type_from_resolved_name(resolved, element)), *length),
    }
}
