use inscribe_ast::nodes::{
    Expr, ExprKind, FunctionDecl, Item, Literal, Module, Pattern, PatternKind, Stmt,
};
use inscribe_resolve::{FunctionKey, ResolvedProgram};
use inscribe_typeck::{expr_key, FunctionSignature, Type, TypeCheckResult};

use crate::nodes::{
    HirBinding, HirBlock, HirExpr, HirExprKind, HirField, HirFor, HirFunction, HirImport, HirItem,
    HirMatchArm, HirParam, HirProgram, HirStmt, HirStruct, HirWhile,
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
        Item::Function(function) => HirItem::Function(lower_function(function, typed)),
    }
}

fn lower_function(function: &FunctionDecl, typed: &TypeCheckResult) -> HirFunction {
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
        receiver: key.receiver.clone(),
        name: key.name,
        signature,
        params,
        is_declaration: function.body.is_none(),
        body: function.body.as_ref().map(|body| lower_block(body, typed)),
        span: function.span,
    }
}

fn lower_block(block: &inscribe_ast::nodes::Block, typed: &TypeCheckResult) -> HirBlock {
    let statements = block
        .statements
        .iter()
        .map(|statement| lower_statement(statement, typed))
        .collect::<Vec<_>>();
    let ty = statements.last().map(statement_type).unwrap_or(Type::Unit);

    HirBlock {
        statements,
        ty,
        span: block.span,
    }
}

fn lower_statement(statement: &Stmt, typed: &TypeCheckResult) -> HirStmt {
    match statement {
        Stmt::Let(stmt) => HirStmt::Let(HirBinding {
            name: stmt.name.clone(),
            ty: lookup_type(typed, stmt.value.span),
            value: lower_expr(&stmt.value, typed),
            span: stmt.span,
        }),
        Stmt::Const(stmt) => HirStmt::Const(HirBinding {
            name: stmt.name.clone(),
            ty: lookup_type(typed, stmt.value.span),
            value: lower_expr(&stmt.value, typed),
            span: stmt.span,
        }),
        Stmt::For(stmt) => HirStmt::For(HirFor {
            binding: pattern_display(&stmt.pattern),
            binding_ty: lookup_type(typed, stmt.iterable.span),
            iterable: lower_expr(&stmt.iterable, typed),
            body: lower_block(&stmt.body, typed),
            span: stmt.span,
        }),
        Stmt::While(stmt) => HirStmt::While(HirWhile {
            condition: lower_expr(&stmt.condition, typed),
            body: lower_block(&stmt.body, typed),
            span: stmt.span,
        }),
        Stmt::Return(value) => HirStmt::Return(
            value.value.as_ref().map(|expr| lower_expr(expr, typed)),
            value.span,
        ),
        Stmt::Expr(expr) => HirStmt::Expr(lower_expr(expr, typed)),
    }
}

fn lower_expr(expr: &Expr, typed: &TypeCheckResult) -> HirExpr {
    let ty = lookup_type(typed, expr.span);
    let kind = match &expr.kind {
        ExprKind::Literal(literal) => HirExprKind::Literal(literal_display(literal)),
        ExprKind::Path(path) => HirExprKind::Path(path.segments.clone()),
        ExprKind::Unary { op, expr } => HirExprKind::Unary {
            op: format!("{op:?}"),
            expr: Box::new(lower_expr(expr, typed)),
        },
        ExprKind::Binary { op, left, right } => lower_binary_expr(*op, left, right, typed),
        ExprKind::Call { callee, args } => HirExprKind::Call {
            callee: Box::new(lower_expr(callee, typed)),
            args: args.iter().map(|arg| lower_expr(arg, typed)).collect(),
        },
        ExprKind::Field { base, field } => HirExprKind::Field {
            base: Box::new(lower_expr(base, typed)),
            field: field.clone(),
        },
        ExprKind::StructLiteral { path, fields } => HirExprKind::StructLiteral {
            path: path.segments.clone(),
            fields: fields
                .iter()
                .map(|field| (field.name.clone(), lower_expr(&field.value, typed)))
                .collect(),
        },
        ExprKind::If {
            condition,
            then_block,
            else_branch,
        } => HirExprKind::If {
            condition: Box::new(lower_expr(condition, typed)),
            then_block: lower_block(then_block, typed),
            else_branch: else_branch
                .as_ref()
                .map(|expr| Box::new(lower_expr(expr, typed))),
        },
        ExprKind::Match { value, arms } => HirExprKind::Match {
            value: Box::new(lower_expr(value, typed)),
            arms: arms
                .iter()
                .map(|arm| HirMatchArm {
                    pattern: pattern_display(&arm.pattern),
                    value: lower_expr(&arm.value, typed),
                    span: arm.span,
                })
                .collect(),
        },
        ExprKind::Block(block) => HirExprKind::Block(lower_block(block, typed)),
        ExprKind::Try(inner) => HirExprKind::Try(Box::new(lower_expr(inner, typed))),
    };

    HirExpr {
        kind,
        ty,
        span: expr.span,
    }
}

fn lower_binary_expr(
    op: inscribe_ast::nodes::BinaryOp,
    left: &Expr,
    right: &Expr,
    typed: &TypeCheckResult,
) -> HirExprKind {
    match op {
        inscribe_ast::nodes::BinaryOp::And => HirExprKind::If {
            condition: Box::new(lower_expr(left, typed)),
            then_block: block_with_expr(lower_expr(right, typed), right.span),
            else_branch: Some(Box::new(bool_literal_expr(false, left.span))),
        },
        inscribe_ast::nodes::BinaryOp::Or => HirExprKind::If {
            condition: Box::new(lower_expr(left, typed)),
            then_block: block_with_expr(bool_literal_expr(true, left.span), left.span),
            else_branch: Some(Box::new(lower_expr(right, typed))),
        },
        _ => HirExprKind::Binary {
            op: format!("{op:?}"),
            left: Box::new(lower_expr(left, typed)),
            right: Box::new(lower_expr(right, typed)),
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
        Literal::String(value) => format!("{value:?}"),
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
    let head = name.path.last().cloned().unwrap_or_default();
    match head.as_str() {
        "int" => Type::Int,
        "float" => Type::Float,
        "string" => Type::String,
        "bool" => Type::Bool,
        "Error" => Type::Error,
        "Result" => {
            let ok = name
                .arguments
                .first()
                .map(|argument| type_from_resolved_name(resolved, argument))
                .unwrap_or(Type::Unknown);
            let err = name
                .arguments
                .get(1)
                .map(|argument| type_from_resolved_name(resolved, argument))
                .unwrap_or(Type::Error);
            Type::Result(Box::new(ok), Box::new(err))
        }
        _ if resolved.structs.contains_key(&head) => Type::Struct(head),
        _ => Type::Unknown,
    }
}
