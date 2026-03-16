use std::collections::HashMap;

use inscribe_ast::nodes::{
    Expr, ExprKind, FunctionDecl, Item, Literal, Module, Pattern, PatternKind, Stmt,
};
use inscribe_ast::span::Span;
use inscribe_resolve::scope::ScopeStack;
use inscribe_resolve::{FunctionKey, ResolvedProgram};
use inscribe_typeck::{expr_key, FunctionSignature, Type, TypeCheckResult};

use crate::nodes::{
    HirBinding, HirBlock, HirEnum, HirEnumVariant, HirExpr, HirExprKind, HirField, HirFor,
    HirFunction, HirImport, HirItem, HirMatchArm, HirParam, HirProgram, HirStmt, HirStruct,
    HirSymbol, HirSymbolId, HirSymbolKind, HirWhile,
};

pub fn lower_module(
    module: &Module,
    resolved: &ResolvedProgram,
    typed: &TypeCheckResult,
) -> HirProgram {
    let mut lowerer = HirLowerer::new(resolved, typed);
    lowerer.predeclare(module);
    let items = module
        .items
        .iter()
        .map(|item| lowerer.lower_item(item))
        .collect();

    HirProgram {
        items,
        symbols: lowerer.symbols,
        span: module.span,
    }
}

struct HirLowerer<'a> {
    resolved: &'a ResolvedProgram,
    typed: &'a TypeCheckResult,
    symbols: Vec<HirSymbol>,
    structs: HashMap<String, HirSymbolId>,
    enums: HashMap<String, HirSymbolId>,
    functions: HashMap<FunctionKey, HirSymbolId>,
    imports: HashMap<String, HirSymbolId>,
    struct_fields: HashMap<HirSymbolId, HashMap<String, HirSymbolId>>,
    enum_variants: HashMap<HirSymbolId, HashMap<String, HirSymbolId>>,
    unresolved_paths: HashMap<String, HirSymbolId>,
    unresolved_fields: HashMap<String, HirSymbolId>,
}

impl<'a> HirLowerer<'a> {
    fn new(resolved: &'a ResolvedProgram, typed: &'a TypeCheckResult) -> Self {
        Self {
            resolved,
            typed,
            symbols: Vec::new(),
            structs: HashMap::new(),
            enums: HashMap::new(),
            functions: HashMap::new(),
            imports: HashMap::new(),
            struct_fields: HashMap::new(),
            enum_variants: HashMap::new(),
            unresolved_paths: HashMap::new(),
            unresolved_fields: HashMap::new(),
        }
    }

    fn predeclare(&mut self, module: &Module) {
        for item in &module.items {
            match item {
                Item::Import(import) => {
                    let alias = import
                        .path
                        .segments
                        .last()
                        .cloned()
                        .unwrap_or_else(|| "root".to_string());
                    let symbol = self.push_symbol(alias.clone(), HirSymbolKind::Import, import.span);
                    self.imports.insert(alias, symbol);
                }
                Item::Struct(decl) => {
                    let symbol =
                        self.push_symbol(decl.name.clone(), HirSymbolKind::Struct, decl.name_span);
                    self.structs.insert(decl.name.clone(), symbol);
                    let mut fields = HashMap::new();
                    for field in &decl.fields {
                        let field_symbol = self.push_symbol(
                            field.name.clone(),
                            HirSymbolKind::Field,
                            field.name_span,
                        );
                        fields.insert(field.name.clone(), field_symbol);
                    }
                    self.struct_fields.insert(symbol, fields);
                }
                Item::Enum(decl) => {
                    let symbol =
                        self.push_symbol(decl.name.clone(), HirSymbolKind::Enum, decl.name_span);
                    self.enums.insert(decl.name.clone(), symbol);
                    let mut variants = HashMap::new();
                    for variant in &decl.variants {
                        let variant_symbol = self.push_symbol(
                            variant.name.clone(),
                            HirSymbolKind::Variant,
                            variant.name_span,
                        );
                        variants.insert(variant.name.clone(), variant_symbol);
                    }
                    self.enum_variants.insert(symbol, variants);
                }
                Item::Function(function) => {
                    let key = function_key(function);
                    let display = function_display_name(&key);
                    let symbol =
                        self.push_symbol(display, HirSymbolKind::Function, function.name_span);
                    self.functions.insert(key, symbol);
                }
            }
        }
    }

    fn lower_item(&mut self, item: &Item) -> HirItem {
        match item {
            Item::Import(import) => {
                let alias = import
                    .path
                    .segments
                    .last()
                    .cloned()
                    .unwrap_or_else(|| "root".to_string());
                let symbol = self
                    .imports
                    .get(&alias)
                    .copied()
                    .unwrap_or_else(|| self.push_symbol(alias, HirSymbolKind::Import, import.span));
                HirItem::Import(HirImport {
                    symbol,
                    path: import.path.segments.clone(),
                    span: import.span,
                })
            }
            Item::Struct(decl) => {
                let symbol = self
                    .structs
                    .get(&decl.name)
                    .copied()
                    .unwrap_or_else(|| {
                        let id =
                            self.push_symbol(decl.name.clone(), HirSymbolKind::Struct, decl.name_span);
                        self.structs.insert(decl.name.clone(), id);
                        id
                    });
                let fields = decl
                    .fields
                    .iter()
                    .map(|field| {
                        let field_symbol = self
                            .struct_fields
                            .get(&symbol)
                            .and_then(|fields| fields.get(&field.name))
                            .copied()
                            .unwrap_or_else(|| {
                                self.push_symbol(
                                    field.name.clone(),
                                    HirSymbolKind::Field,
                                    field.name_span,
                                )
                            });
                        HirField {
                            symbol: field_symbol,
                            ty: self
                                .resolved
                                .structs
                                .get(&decl.name)
                                .and_then(|info| info.fields.get(&field.name))
                                .map(|ty| type_from_resolved_name(self.resolved, ty))
                                .unwrap_or(Type::Unknown),
                            span: field.span,
                        }
                    })
                    .collect();
                HirItem::Struct(HirStruct {
                    symbol,
                    fields,
                    span: decl.span,
                })
            }
            Item::Enum(decl) => {
                let symbol = self
                    .enums
                    .get(&decl.name)
                    .copied()
                    .unwrap_or_else(|| {
                        let id =
                            self.push_symbol(decl.name.clone(), HirSymbolKind::Enum, decl.name_span);
                        self.enums.insert(decl.name.clone(), id);
                        id
                    });
                let variants = decl
                    .variants
                    .iter()
                    .filter_map(|variant| {
                        let variant_symbol = self
                            .enum_variants
                            .get(&symbol)
                            .and_then(|variants| variants.get(&variant.name))
                            .copied()
                            .unwrap_or_else(|| {
                                self.push_symbol(
                                    variant.name.clone(),
                                    HirSymbolKind::Variant,
                                    variant.name_span,
                                )
                            });
                        let discriminant = self
                            .resolved
                            .enums
                            .get(&decl.name)
                            .and_then(|info| info.variants.get(&variant.name))
                            .copied()?;
                        Some(HirEnumVariant {
                            symbol: variant_symbol,
                            discriminant,
                            span: variant.span,
                        })
                    })
                    .collect();
                HirItem::Enum(HirEnum {
                    symbol,
                    variants,
                    span: decl.span,
                })
            }
            Item::Function(function) => HirItem::Function(self.lower_function(function)),
        }
    }

    fn lower_function(&mut self, function: &FunctionDecl) -> HirFunction {
        let key = function_key(function);
        let symbol = self
            .functions
            .get(&key)
            .copied()
            .unwrap_or_else(|| {
                let id = self.push_symbol(
                    function_display_name(&key),
                    HirSymbolKind::Function,
                    function.name_span,
                );
                self.functions.insert(key.clone(), id);
                id
            });
        let signature = self
            .typed
            .function_signatures
            .get(&key)
            .cloned()
            .unwrap_or(FunctionSignature {
                key: key.clone(),
                params: Vec::new(),
                return_type: Box::new(Type::Unknown),
            });

        let receiver = key
            .receiver
            .as_ref()
            .and_then(|receiver| self.structs.get(receiver).copied())
            .or_else(|| {
                key.receiver.as_ref().map(|name| {
                    self.push_symbol(name.clone(), HirSymbolKind::Struct, function.name_span)
                })
            });

        let mut scope = ScopeStack::new();
        let params = function
            .params
            .iter()
            .enumerate()
            .map(|(index, param)| {
                let param_symbol =
                    self.push_symbol(param.name.clone(), HirSymbolKind::Param, param.span);
                scope.define(param.name.clone(), param_symbol);
                HirParam {
                    symbol: param_symbol,
                    ty: signature
                        .params
                        .get(index)
                        .cloned()
                        .unwrap_or(Type::Unknown),
                    span: param.span,
                }
            })
            .collect();

        HirFunction {
            symbol,
            visibility: function.visibility,
            receiver,
            signature,
            params,
            is_declaration: function.body.is_none(),
            body: function
                .body
                .as_ref()
                .map(|body| self.lower_block(body, &mut scope)),
            span: function.span,
        }
    }

    fn lower_block(
        &mut self,
        block: &inscribe_ast::nodes::Block,
        scope: &mut ScopeStack<HirSymbolId>,
    ) -> HirBlock {
        scope.push();
        let statements = block
            .statements
            .iter()
            .map(|statement| self.lower_statement(statement, scope))
            .collect::<Vec<_>>();
        let ty = statements.last().map(statement_type).unwrap_or(Type::Unit);
        scope.pop();

        HirBlock {
            statements,
            ty,
            span: block.span,
        }
    }

    fn lower_statement(&mut self, statement: &Stmt, scope: &mut ScopeStack<HirSymbolId>) -> HirStmt {
        match statement {
            Stmt::Let(stmt) => {
                let symbol = self.push_symbol(stmt.name.clone(), HirSymbolKind::Local, stmt.span);
                let value = self.lower_expr(&stmt.value, scope);
                scope.define(stmt.name.clone(), symbol);
                HirStmt::Let(HirBinding {
                    symbol,
                    ty: lookup_type(self.typed, stmt.value.span),
                    value,
                    span: stmt.span,
                })
            }
            Stmt::Const(stmt) => {
                let symbol = self.push_symbol(stmt.name.clone(), HirSymbolKind::Local, stmt.span);
                let value = self.lower_expr(&stmt.value, scope);
                scope.define(stmt.name.clone(), symbol);
                HirStmt::Const(HirBinding {
                    symbol,
                    ty: lookup_type(self.typed, stmt.value.span),
                    value,
                    span: stmt.span,
                })
            }
            Stmt::For(stmt) => {
                let binding_name = pattern_display(&stmt.pattern);
                let binding_symbol =
                    self.push_symbol(binding_name.clone(), HirSymbolKind::Local, stmt.pattern.span);
                let iterable = self.lower_expr(&stmt.iterable, scope);
                scope.push();
                scope.define(binding_name, binding_symbol);
                let body = self.lower_block(&stmt.body, scope);
                scope.pop();
                HirStmt::For(HirFor {
                    binding: binding_symbol,
                    binding_ty: lookup_type(self.typed, stmt.iterable.span),
                    iterable,
                    body,
                    span: stmt.span,
                })
            }
            Stmt::While(stmt) => HirStmt::While(HirWhile {
                condition: self.lower_expr(&stmt.condition, scope),
                body: self.lower_block(&stmt.body, scope),
                span: stmt.span,
            }),
            Stmt::Return(value) => HirStmt::Return(
                value
                    .value
                    .as_ref()
                    .map(|expr| self.lower_expr(expr, scope)),
                value.span,
            ),
            Stmt::Expr(expr) => HirStmt::Expr(self.lower_expr(expr, scope)),
        }
    }

    fn lower_expr(&mut self, expr: &Expr, scope: &mut ScopeStack<HirSymbolId>) -> HirExpr {
        let ty = lookup_type(self.typed, expr.span);
        let kind = match &expr.kind {
            ExprKind::Literal(literal) => HirExprKind::Literal(literal_display(literal)),
            ExprKind::Path(path) => self
                .lower_enum_variant_path(&path.segments, &ty, expr.span)
                .unwrap_or_else(|| HirExprKind::Path(self.resolve_path_symbol(&path.segments, expr.span, scope))),
            ExprKind::Array(items) => {
                HirExprKind::Array(
                    items
                        .iter()
                        .map(|item| self.lower_expr(item, scope))
                        .collect(),
                )
            }
            ExprKind::RepeatArray { value, length } => HirExprKind::RepeatArray {
                value: Box::new(self.lower_expr(value, scope)),
                length: *length,
            },
            ExprKind::Cast { expr, .. } => HirExprKind::Cast {
                expr: Box::new(self.lower_expr(expr, scope)),
            },
            ExprKind::Unary { op, expr } => HirExprKind::Unary {
                op: format!("{op:?}"),
                expr: Box::new(self.lower_expr(expr, scope)),
            },
            ExprKind::Binary { op, left, right } => self.lower_binary_expr(*op, left, right, scope),
            ExprKind::Call { callee, args } => HirExprKind::Call {
                callee: Box::new(self.lower_expr(callee, scope)),
                args: args.iter().map(|arg| self.lower_expr(arg, scope)).collect(),
            },
            ExprKind::Field { base, field } => self
                .lower_enum_variant_field(base, field, &ty, expr.span)
                .unwrap_or_else(|| {
                    let base_expr = self.lower_expr(base, scope);
                    let field_symbol = self.resolve_field_symbol(&base_expr.ty, field, expr.span);
                    HirExprKind::Field {
                        base: Box::new(base_expr),
                        field: field_symbol,
                    }
                }),
            ExprKind::Index { target, index } => HirExprKind::Index {
                target: Box::new(self.lower_expr(target, scope)),
                index: Box::new(self.lower_expr(index, scope)),
            },
            ExprKind::StructLiteral { path, fields } => {
                let struct_name = path.segments.last().cloned().unwrap_or_default();
                let struct_id = self
                    .structs
                    .get(&struct_name)
                    .copied()
                    .unwrap_or_else(|| {
                        let id =
                            self.push_symbol(struct_name.clone(), HirSymbolKind::Struct, path.span);
                        self.structs.insert(struct_name.clone(), id);
                        id
                    });
                let mut lowered_fields = Vec::new();
                for field in fields {
                    let known_field = self
                        .struct_fields
                        .get(&struct_id)
                        .and_then(|fields| fields.get(&field.name))
                        .copied();
                    let field_symbol = known_field.unwrap_or_else(|| {
                        self.push_symbol(
                            field.name.clone(),
                            HirSymbolKind::Field,
                            field.name_span,
                        )
                    });
                    lowered_fields.push((field_symbol, self.lower_expr(&field.value, scope)));
                }
                HirExprKind::StructLiteral {
                    struct_id,
                    fields: lowered_fields,
                }
            }
            ExprKind::If {
                condition,
                then_block,
                else_branch,
            } => HirExprKind::If {
                condition: Box::new(self.lower_expr(condition, scope)),
                then_block: self.lower_block(then_block, scope),
                else_branch: else_branch
                    .as_ref()
                    .map(|expr| Box::new(self.lower_expr(expr, scope))),
            },
            ExprKind::Match { value, arms } => HirExprKind::Match {
                value: Box::new(self.lower_expr(value, scope)),
                arms: arms
                    .iter()
                    .map(|arm| HirMatchArm {
                        pattern: pattern_display(&arm.pattern),
                        value: self.lower_expr(&arm.value, scope),
                        span: arm.span,
                    })
                    .collect(),
            },
            ExprKind::Block(block) => HirExprKind::Block(self.lower_block(block, scope)),
            ExprKind::Try(inner) => HirExprKind::Try(Box::new(self.lower_expr(inner, scope))),
        };

        HirExpr {
            kind,
            ty,
            span: expr.span,
        }
    }

    fn lower_enum_variant_path(
        &mut self,
        segments: &[String],
        ty: &Type,
        span: Span,
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
        let enum_id = self.enums.get(enum_name).copied().unwrap_or_else(|| {
            let id = self.push_symbol(enum_name.clone(), HirSymbolKind::Enum, span);
            self.enums.insert(enum_name.clone(), id);
            id
        });
        let variant_id = self
            .enum_variants
            .get(&enum_id)
            .and_then(|variants| variants.get(variant))
            .copied()
            .unwrap_or_else(|| {
                let id = self.push_symbol(variant.clone(), HirSymbolKind::Variant, span);
                self.enum_variants
                    .entry(enum_id)
                    .or_default()
                    .insert(variant.clone(), id);
                id
            });
        let discriminant = self
            .resolved
            .enums
            .get(enum_name)
            .and_then(|info| info.variants.get(variant))
            .copied()?;
        Some(HirExprKind::EnumVariant {
            enum_id,
            variant_id,
            discriminant,
        })
    }

    fn lower_enum_variant_field(
        &mut self,
        base: &Expr,
        field: &str,
        ty: &Type,
        span: Span,
    ) -> Option<HirExprKind> {
        let ExprKind::Path(path) = &base.kind else {
            return None;
        };
        let [enum_name] = path.segments.as_slice() else {
            return None;
        };
        self.lower_enum_variant_path(&[enum_name.clone(), field.to_string()], ty, span)
    }

    fn lower_binary_expr(
        &mut self,
        op: inscribe_ast::nodes::BinaryOp,
        left: &Expr,
        right: &Expr,
        scope: &mut ScopeStack<HirSymbolId>,
    ) -> HirExprKind {
        match op {
            inscribe_ast::nodes::BinaryOp::And => HirExprKind::If {
                condition: Box::new(self.lower_expr(left, scope)),
                then_block: block_with_expr(self.lower_expr(right, scope), right.span),
                else_branch: Some(Box::new(bool_literal_expr(false, left.span))),
            },
            inscribe_ast::nodes::BinaryOp::Or => HirExprKind::If {
                condition: Box::new(self.lower_expr(left, scope)),
                then_block: block_with_expr(bool_literal_expr(true, left.span), left.span),
                else_branch: Some(Box::new(self.lower_expr(right, scope))),
            },
            _ => HirExprKind::Binary {
                op: format!("{op:?}"),
                left: Box::new(self.lower_expr(left, scope)),
                right: Box::new(self.lower_expr(right, scope)),
            },
        }
    }

    fn resolve_path_symbol(
        &mut self,
        segments: &[String],
        span: Span,
        scope: &ScopeStack<HirSymbolId>,
    ) -> HirSymbolId {
        let Some(head) = segments.first() else {
            return self.push_symbol("_".to_string(), HirSymbolKind::Unresolved, span);
        };

        if segments.len() == 1 {
            if let Some(symbol) = scope.lookup(head) {
                return symbol;
            }
            if let Some(symbol) = self.imports.get(head) {
                return *symbol;
            }
            if let Some(symbol) = self
                .functions
                .get(&FunctionKey { receiver: None, name: head.clone() })
            {
                return *symbol;
            }
            if let Some(symbol) = self.structs.get(head) {
                return *symbol;
            }
            if let Some(symbol) = self.enums.get(head) {
                return *symbol;
            }
        } else if segments.len() == 2 {
            let key = FunctionKey {
                receiver: Some(segments[0].clone()),
                name: segments[1].clone(),
            };
            if let Some(symbol) = self.functions.get(&key) {
                return *symbol;
            }
        }

        let joined = segments.join(".");
        if let Some(symbol) = self.unresolved_paths.get(&joined) {
            return *symbol;
        }
        let symbol = self.push_symbol(joined.clone(), HirSymbolKind::Unresolved, span);
        self.unresolved_paths.insert(joined, symbol);
        symbol
    }

    fn resolve_field_symbol(&mut self, base_ty: &Type, field: &str, span: Span) -> HirSymbolId {
        if let Type::Struct(struct_name) = base_ty {
            if let Some(struct_id) = self.structs.get(struct_name) {
                if let Some(fields) = self.struct_fields.get(struct_id) {
                    if let Some(field_id) = fields.get(field) {
                        return *field_id;
                    }
                }
            }
        }

        if let Some(symbol) = self.unresolved_fields.get(field) {
            return *symbol;
        }
        let symbol = self.push_symbol(field.to_string(), HirSymbolKind::Field, span);
        self.unresolved_fields.insert(field.to_string(), symbol);
        symbol
    }

    fn push_symbol(&mut self, name: String, kind: HirSymbolKind, span: Span) -> HirSymbolId {
        let id = HirSymbolId(self.symbols.len());
        self.symbols.push(HirSymbol {
            id,
            name,
            kind,
            span,
        });
        id
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

fn function_key(function: &FunctionDecl) -> FunctionKey {
    FunctionKey {
        receiver: function
            .receiver
            .as_ref()
            .map(|path| path.segments.join(".")),
        name: function.name.clone(),
    }
}

fn function_display_name(key: &FunctionKey) -> String {
    match &key.receiver {
        Some(receiver) => format!("{receiver}.{}", key.name),
        None => key.name.clone(),
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
