use std::collections::HashMap;

use inscribe_ast::nodes::{
    Block, Expr, ExprKind, FunctionDecl, Item, Literal, MatchArm, Module, Pattern, PatternKind,
    Stmt,
};
use inscribe_ast::span::Span;
use inscribe_resolve::{FunctionInfo, FunctionKey, ParamInfo, ResolvedProgram, TypeName};

use crate::errors::TypeError;
use crate::infer::{expr_key, BindingInfo, BindingKind, FunctionSignature, Type, TypeCheckResult};
use crate::ownership::ensure_assignable;
use crate::unify::unify;

// TODO: Split the checker into smaller passes once the language surface grows.

pub fn check_module(
    module: &Module,
    resolved: &ResolvedProgram,
) -> Result<TypeCheckResult, Vec<TypeError>> {
    TypeChecker::new(resolved).check_module(module)
}

struct TypeChecker<'a> {
    resolved: &'a ResolvedProgram,
    result: TypeCheckResult,
    errors: Vec<TypeError>,
    scopes: Vec<HashMap<String, BindingInfo>>,
    current_return: Type,
}

impl<'a> TypeChecker<'a> {
    fn new(resolved: &'a ResolvedProgram) -> Self {
        Self {
            resolved,
            result: TypeCheckResult::default(),
            errors: Vec::new(),
            scopes: vec![HashMap::new()],
            current_return: Type::Unit,
        }
    }

    fn check_module(mut self, module: &Module) -> Result<TypeCheckResult, Vec<TypeError>> {
        self.seed_item_types();
        self.seed_function_signatures();

        for item in &module.items {
            if let Item::Function(function) = item {
                self.check_function(function);
            }
        }

        if self.errors.is_empty() {
            Ok(self.result)
        } else {
            Err(self.errors)
        }
    }

    fn seed_item_types(&mut self) {
        for name in self.resolved.structs.keys() {
            self.result
                .item_types
                .insert(name.clone(), Type::Struct(name.clone()));
        }
        for name in self.resolved.enums.keys() {
            self.result
                .item_types
                .insert(name.clone(), Type::Enum(name.clone()));
        }
    }

    fn seed_function_signatures(&mut self) {
        for (key, function) in &self.resolved.functions {
            let signature = self.signature_for_function(function);
            self.result
                .function_signatures
                .insert(key.clone(), signature.clone());
            self.result
                .item_types
                .insert(display_function_name(key), Type::Function(signature));
        }
    }

    fn check_function(&mut self, function: &FunctionDecl) {
        let key = FunctionKey {
            receiver: function
                .receiver
                .as_ref()
                .map(|path| path.segments.join(".")),
            name: function.name.clone(),
        };

        let Some(signature) = self.result.function_signatures.get(&key).cloned() else {
            self.errors.push(TypeError::new(
                format!(
                    "missing signature for function `{}`",
                    display_function_name(&key)
                ),
                function.span,
            ));
            return;
        };

        let previous_return = self.current_return.clone();
        self.current_return = (*signature.return_type).clone();
        self.push_scope();

        for (param, expected_type) in function.params.iter().zip(signature.params.iter()) {
            self.define_binding(
                param.name.clone(),
                BindingInfo {
                    ty: expected_type.clone(),
                    kind: BindingKind::Param,
                },
            );
        }

        if let Some(body) = &function.body {
            let block_type = self.check_block(body);
            let expected = function
                .return_type
                .as_ref()
                .map(|_| self.current_return.clone())
                .unwrap_or(Type::Unit);
            self.expect_type(&expected, &block_type, body.span);
        }

        self.pop_scope();
        self.current_return = previous_return;
    }

    fn check_block(&mut self, block: &Block) -> Type {
        self.push_scope();
        let mut last_type = Type::Unit;
        for statement in &block.statements {
            last_type = self.check_statement(statement);
        }
        self.pop_scope();
        last_type
    }

    fn check_statement(&mut self, statement: &Stmt) -> Type {
        match statement {
            Stmt::Let(stmt) => {
                let value_ty = self.check_expr(&stmt.value);
                let binding_ty = if let Some(annotation) = &stmt.ty {
                    let annotated = self.type_from_ast_ref(annotation);
                    self.expect_type(&annotated, &value_ty, stmt.span);
                    annotated
                } else {
                    value_ty
                };
                self.define_binding(
                    stmt.name.clone(),
                    BindingInfo {
                        ty: binding_ty,
                        kind: BindingKind::Let,
                    },
                );
                Type::Unit
            }
            Stmt::Const(stmt) => {
                let value_ty = self.check_expr(&stmt.value);
                let binding_ty = if let Some(annotation) = &stmt.ty {
                    let annotated = self.type_from_ast_ref(annotation);
                    self.expect_type(&annotated, &value_ty, stmt.span);
                    annotated
                } else {
                    value_ty
                };
                self.define_binding(
                    stmt.name.clone(),
                    BindingInfo {
                        ty: binding_ty,
                        kind: BindingKind::Const,
                    },
                );
                Type::Unit
            }
            Stmt::For(stmt) => {
                let iterable_ty = self.check_expr(&stmt.iterable);
                let item_ty = match iterable_ty {
                    Type::Range(inner) => *inner,
                    Type::Unknown => Type::Unknown,
                    other => {
                        self.errors.push(TypeError::new(
                            format!("cannot iterate over `{}`", other.display_name()),
                            stmt.span,
                        ));
                        Type::Unknown
                    }
                };

                self.push_scope();
                self.bind_pattern(&stmt.pattern, &item_ty);
                let _ = self.check_block(&stmt.body);
                self.pop_scope();
                Type::Unit
            }
            Stmt::While(stmt) => {
                let condition_ty = self.check_expr(&stmt.condition);
                self.expect_type(&Type::Bool, &condition_ty, stmt.condition.span);
                let _ = self.check_block(&stmt.body);
                Type::Unit
            }
            Stmt::Return(stmt) => {
                let value_ty = stmt
                    .value
                    .as_ref()
                    .map(|expr| self.check_expr(expr))
                    .unwrap_or(Type::Unit);
                let expected = self.current_return.clone();
                self.expect_type(&expected, &value_ty, stmt.span);
                value_ty
            }
            Stmt::Expr(expr) => self.check_expr(expr),
        }
    }

    fn check_expr(&mut self, expr: &Expr) -> Type {
        let ty = match &expr.kind {
            ExprKind::Literal(literal) => self.type_of_literal(literal),
            ExprKind::Path(path) => self.check_path_segments(&path.segments, expr.span),
            ExprKind::Array(items) => self.check_array(expr.span, items),
            ExprKind::RepeatArray { value, length } => {
                let item_ty = self.check_expr(value);
                Type::Array(Box::new(item_ty), *length)
            }
            ExprKind::Unary { op, expr } => {
                let value_ty = self.check_expr(expr);
                match op {
                    inscribe_ast::nodes::UnaryOp::Negate => {
                        if matches!(value_ty, Type::Int | Type::Byte | Type::Float | Type::Unknown)
                        {
                            value_ty
                        } else {
                            self.errors.push(TypeError::new(
                                format!("cannot negate `{}`", value_ty.display_name()),
                                expr.span,
                            ));
                            Type::Unknown
                        }
                    }
                    inscribe_ast::nodes::UnaryOp::Not => {
                        self.expect_type(&Type::Bool, &value_ty, expr.span);
                        Type::Bool
                    }
                }
            }
            ExprKind::Binary { op, left, right } => self.check_binary(expr.span, *op, left, right),
            ExprKind::Call { callee, args } => self.check_call(expr.span, callee, args),
            ExprKind::Field { base, field } => self.check_field(expr.span, base, field),
            ExprKind::Index { target, index } => self.check_index(expr.span, target, index),
            ExprKind::StructLiteral { path, fields } => self.check_struct_literal(
                expr.span,
                path.segments.last().cloned().unwrap_or_default(),
                fields,
            ),
            ExprKind::If {
                condition,
                then_block,
                else_branch,
            } => {
                let condition_ty = self.check_expr(condition);
                self.expect_type(&Type::Bool, &condition_ty, condition.span);
                let then_ty = self.check_block(then_block);
                let else_ty = else_branch
                    .as_ref()
                    .map(|expr| self.check_expr(expr))
                    .unwrap_or(Type::Unit);
                match unify(&then_ty, &else_ty, expr.span) {
                    Ok(ty) => ty,
                    Err(error) => {
                        self.errors.push(error);
                        Type::Unknown
                    }
                }
            }
            ExprKind::Match { value, arms } => self.check_match(expr.span, value, arms),
            ExprKind::Block(block) => self.check_block(block),
            ExprKind::Try(inner) => self.check_try(expr.span, inner),
        };

        self.result
            .expr_types
            .insert(expr_key(expr.span), ty.clone());
        ty
    }

    fn check_binary(
        &mut self,
        span: Span,
        op: inscribe_ast::nodes::BinaryOp,
        left: &Expr,
        right: &Expr,
    ) -> Type {
        if matches!(op, inscribe_ast::nodes::BinaryOp::Assign) {
            let right_ty = self.check_expr(right);
            return self.check_assignment_target(left, &right_ty, span);
        }

        let left_ty = self.check_expr(left);
        let right_ty = self.check_expr(right);

        match op {
            inscribe_ast::nodes::BinaryOp::Range => {
                self.expect_type(&Type::Int, &left_ty, left.span);
                self.expect_type(&Type::Int, &right_ty, right.span);
                Type::Range(Box::new(Type::Int))
            }
            inscribe_ast::nodes::BinaryOp::Or | inscribe_ast::nodes::BinaryOp::And => {
                self.expect_type(&Type::Bool, &left_ty, left.span);
                self.expect_type(&Type::Bool, &right_ty, right.span);
                Type::Bool
            }
            inscribe_ast::nodes::BinaryOp::Equal
            | inscribe_ast::nodes::BinaryOp::NotEqual
            | inscribe_ast::nodes::BinaryOp::Less
            | inscribe_ast::nodes::BinaryOp::LessEqual
            | inscribe_ast::nodes::BinaryOp::Greater
            | inscribe_ast::nodes::BinaryOp::GreaterEqual => {
                self.expect_type(&left_ty, &right_ty, span);
                Type::Bool
            }
            inscribe_ast::nodes::BinaryOp::Add
            | inscribe_ast::nodes::BinaryOp::Subtract
            | inscribe_ast::nodes::BinaryOp::Multiply
            | inscribe_ast::nodes::BinaryOp::Divide => {
                if matches!(left_ty, Type::Int | Type::Byte | Type::Float | Type::String | Type::Unknown)
                    && matches!(right_ty, Type::Int | Type::Byte | Type::Float | Type::String | Type::Unknown)
                {
                    match unify(&left_ty, &right_ty, span) {
                        Ok(ty) => ty,
                        Err(error) => {
                            self.errors.push(error);
                            Type::Unknown
                        }
                    }
                } else {
                    self.errors.push(TypeError::new(
                        format!(
                            "operator requires numeric or string operands, found `{}` and `{}`",
                            left_ty.display_name(),
                            right_ty.display_name()
                        ),
                        span,
                    ));
                    Type::Unknown
                }
            }
            inscribe_ast::nodes::BinaryOp::Assign => unreachable!("assignment handled early"),
        }
    }

    fn check_call(&mut self, span: Span, callee: &Expr, args: &[Expr]) -> Type {
        if let ExprKind::Path(path) = &callee.kind {
            if path.segments.len() == 1 {
                match path.segments[0].as_str() {
                    "Ok" => return self.check_result_constructor(span, args, true),
                    "Err" => return self.check_result_constructor(span, args, false),
                    _ => {}
                }
            }
        }

        let callee_ty = self.check_expr(callee);
        match callee_ty {
            Type::Function(signature) => self.check_signature_call(span, &signature, args),
            other => {
                self.errors.push(TypeError::new(
                    format!("cannot call `{}` as a function", other.display_name()),
                    span,
                ));
                Type::Unknown
            }
        }
    }

    fn check_result_constructor(&mut self, span: Span, args: &[Expr], is_ok: bool) -> Type {
        if args.len() != 1 {
            self.errors.push(TypeError::new(
                format!(
                    "constructor `{}` expects 1 argument, found {}",
                    if is_ok { "Ok" } else { "Err" },
                    args.len()
                ),
                span,
            ));
            return Type::Unknown;
        }

        let payload = self.check_expr(&args[0]);
        if is_ok {
            Type::Result(Box::new(payload), Box::new(Type::Error))
        } else {
            Type::Result(Box::new(Type::Unknown), Box::new(payload))
        }
    }

    fn check_signature_call(
        &mut self,
        span: Span,
        signature: &FunctionSignature,
        args: &[Expr],
    ) -> Type {
        if signature.params.len() != args.len() {
            self.errors.push(TypeError::new(
                format!(
                    "function `{}` expects {} arguments, found {}",
                    display_function_name(&signature.key),
                    signature.params.len(),
                    args.len()
                ),
                span,
            ));
        }

        for (expected, arg) in signature.params.iter().zip(args) {
            let actual = self.check_expr(arg);
            self.expect_type(expected, &actual, arg.span);
        }

        (*signature.return_type).clone()
    }

    fn check_field(&mut self, span: Span, base: &Expr, field: &str) -> Type {
        let base_ty = self.check_expr(base);
        match base_ty {
            Type::Struct(name) => {
                if let Some(struct_info) = self.resolved.structs.get(&name) {
                    if let Some(field_ty) = struct_info.fields.get(field) {
                        return self.type_from_name(field_ty);
                    }
                }

                let key = FunctionKey {
                    receiver: Some(name.clone()),
                    name: field.to_string(),
                };
                if let Some(signature) = self.result.function_signatures.get(&key) {
                    return Type::Function(bound_method_signature(signature));
                }

                self.errors.push(TypeError::new(
                    format!("unknown field or method `{field}` on `{name}`"),
                    span,
                ));
                Type::Unknown
            }
            Type::Enum(name) => {
                if self
                    .resolved
                    .enums
                    .get(&name)
                    .is_some_and(|info| info.variants.contains_key(field))
                {
                    Type::Enum(name)
                } else {
                    self.errors.push(TypeError::new(
                        format!("unknown variant `{field}` on `{name}`"),
                        span,
                    ));
                    Type::Unknown
                }
            }
            other => {
                self.errors.push(TypeError::new(
                    format!(
                        "cannot access field `{field}` on `{}`",
                        other.display_name()
                    ),
                    span,
                ));
                Type::Unknown
            }
        }
    }

    fn check_struct_literal(
        &mut self,
        span: Span,
        struct_name: String,
        fields: &[inscribe_ast::nodes::StructLiteralField],
    ) -> Type {
        let Some(struct_info) = self.resolved.structs.get(&struct_name).cloned() else {
            self.errors.push(TypeError::new(
                format!("unknown struct `{struct_name}`"),
                span,
            ));
            return Type::Unknown;
        };

        for field in fields {
            match struct_info.fields.get(&field.name) {
                Some(expected) => {
                    let actual = self.check_expr(&field.value);
                    let expected_ty = self.type_from_name(expected);
                    self.expect_type(&expected_ty, &actual, field.span);
                }
                None => {
                    self.errors.push(TypeError::new(
                        format!("unknown field `{}` for struct `{struct_name}`", field.name),
                        field.span,
                    ));
                }
            }
        }

        for required in struct_info.fields.keys() {
            if !fields.iter().any(|field| &field.name == required) {
                self.errors.push(TypeError::new(
                    format!("missing field `{required}` in `{struct_name}` literal"),
                    span,
                ));
            }
        }

        Type::Struct(struct_name)
    }

    fn check_match(&mut self, span: Span, value: &Expr, arms: &[MatchArm]) -> Type {
        let scrutinee_ty = self.check_expr(value);
        let mut arm_ty = Type::Unknown;

        for arm in arms {
            self.push_scope();
            self.bind_pattern(&arm.pattern, &scrutinee_ty);
            let value_ty = self.check_expr(&arm.value);
            arm_ty = match unify(&arm_ty, &value_ty, arm.span) {
                Ok(ty) => ty,
                Err(error) => {
                    self.errors.push(error);
                    Type::Unknown
                }
            };
            self.pop_scope();
        }

        if arms.is_empty() {
            self.errors.push(TypeError::new(
                "match expression must have at least one arm",
                span,
            ));
            Type::Unknown
        } else {
            arm_ty
        }
    }

    fn check_try(&mut self, span: Span, inner: &Expr) -> Type {
        let inner_ty = self.check_expr(inner);
        let current_return = self.current_return.clone();
        match inner_ty {
            Type::Result(ok, err) => match current_return {
                Type::Result(expected_ok, expected_err) => {
                    self.expect_type(&expected_ok, &ok, span);
                    self.expect_type(&expected_err, &err, span);
                    *ok
                }
                other => {
                    self.errors.push(TypeError::new(
                        format!(
                            "`?` requires the enclosing function to return `Result`, found `{}`",
                            other.display_name()
                        ),
                        span,
                    ));
                    Type::Unknown
                }
            },
            other => {
                self.errors.push(TypeError::new(
                    format!(
                        "`?` can only be applied to `Result`, found `{}`",
                        other.display_name()
                    ),
                    span,
                ));
                Type::Unknown
            }
        }
    }

    fn check_array(&mut self, _span: Span, items: &[Expr]) -> Type {
        let mut element_ty = Type::Unknown;
        for item in items {
            let item_ty = self.check_expr(item);
            element_ty = match unify(&element_ty, &item_ty, item.span) {
                Ok(ty) => ty,
                Err(error) => {
                    self.errors.push(error);
                    Type::Unknown
                }
            };
        }
        Type::Array(Box::new(element_ty), items.len())
    }

    fn check_index(&mut self, span: Span, target: &Expr, index: &Expr) -> Type {
        let target_ty = self.check_expr(target);
        let index_ty = self.check_expr(index);
        self.expect_type(&Type::Int, &index_ty, index.span);
        match target_ty {
            Type::Array(element, _) => *element,
            Type::String => Type::Byte,
            Type::Unknown => Type::Unknown,
            other => {
                self.errors.push(TypeError::new(
                    format!("cannot index into `{}`", other.display_name()),
                    span,
                ));
                Type::Unknown
            }
        }
    }

    fn check_assignment_target(&mut self, left: &Expr, right_ty: &Type, span: Span) -> Type {
        match &left.kind {
            ExprKind::Path(path) => {
                if let Some(binding) = self.lookup_binding(&path.segments[0]) {
                    if let Err(error) = ensure_assignable(&binding, &path.segments[0], span) {
                        self.errors.push(error);
                    }
                    self.expect_type(&binding.ty, right_ty, span);
                    binding.ty
                } else {
                    self.errors.push(TypeError::new(
                        format!("cannot assign to unknown binding `{}`", path.segments.join(".")),
                        left.span,
                    ));
                    Type::Unknown
                }
            }
            ExprKind::Index { target, index } => {
                let target_ty = self.check_expr(target);
                let index_ty = self.check_expr(index);
                self.expect_type(&Type::Int, &index_ty, index.span);
                match &target.kind {
                    ExprKind::Path(path) => {
                        if let Some(binding) = self.lookup_binding(&path.segments[0]) {
                            if let Err(error) = ensure_assignable(&binding, &path.segments[0], span)
                            {
                                self.errors.push(error);
                            }
                        }
                    }
                    _ => {
                        self.errors.push(TypeError::new(
                            "indexed assignment requires a binding target",
                            target.span,
                        ));
                    }
                }
                match target_ty {
                    Type::Array(element, _) => {
                        self.expect_type(&element, right_ty, span);
                        *element
                    }
                    other => {
                        self.errors.push(TypeError::new(
                            format!("cannot assign through index on `{}`", other.display_name()),
                            span,
                        ));
                        Type::Unknown
                    }
                }
            }
            _ => {
                self.errors.push(TypeError::new(
                    "left-hand side of assignment must be a binding or index expression",
                    left.span,
                ));
                Type::Unknown
            }
        }
    }

    fn bind_pattern(&mut self, pattern: &Pattern, expected: &Type) {
        match &pattern.kind {
            PatternKind::Wildcard | PatternKind::Path(_) => {}
            PatternKind::Binding(name) => {
                self.define_binding(
                    name.clone(),
                    BindingInfo {
                        ty: expected.clone(),
                        kind: BindingKind::Let,
                    },
                );
            }
            PatternKind::Literal(literal) => {
                let literal_ty = self.type_of_literal(literal);
                self.expect_type(expected, &literal_ty, pattern.span);
            }
            PatternKind::Constructor { path, arguments } => {
                let Some(head) = path.segments.first() else {
                    return;
                };

                match (head.as_str(), expected) {
                    ("Ok", Type::Result(ok, _)) if arguments.len() == 1 => {
                        self.bind_pattern(&arguments[0], ok);
                    }
                    ("Err", Type::Result(_, err)) if arguments.len() == 1 => {
                        self.bind_pattern(&arguments[0], err);
                    }
                    _ => {
                        self.errors.push(TypeError::new(
                            format!(
                                "pattern constructor `{}` does not match `{}`",
                                path.segments.join("."),
                                expected.display_name()
                            ),
                            pattern.span,
                        ));
                    }
                }
            }
        }
    }

    fn expect_type(&mut self, expected: &Type, actual: &Type, span: Span) {
        if let Err(error) = unify(expected, actual, span) {
            self.errors.push(error);
        }
    }

    fn signature_for_function(&self, function: &FunctionInfo) -> FunctionSignature {
        let params = function
            .params
            .iter()
            .enumerate()
            .map(|(index, param)| self.type_for_param(function, param, index))
            .collect::<Vec<_>>();
        let return_type = Box::new(
            function
                .return_type
                .as_ref()
                .map(|ty| self.type_from_name(ty))
                .unwrap_or(Type::Unit),
        );

        FunctionSignature {
            key: function.key.clone(),
            params,
            return_type,
        }
    }

    fn type_for_param(&self, function: &FunctionInfo, param: &ParamInfo, index: usize) -> Type {
        if index == 0
            && param.name == "self"
            && function.key.receiver.is_some()
            && param.ty.is_none()
        {
            Type::Struct(function.key.receiver.clone().unwrap_or_default())
        } else {
            param
                .ty
                .as_ref()
                .map(|ty| self.type_from_name(ty))
                .unwrap_or(Type::Unknown)
        }
    }

    fn type_from_name(&self, ty: &TypeName) -> Type {
        match ty {
            TypeName::Named { path, arguments, .. } => {
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
                            .map(|argument| self.type_from_name(argument))
                            .unwrap_or(Type::Unknown);
                        let err = arguments
                            .get(1)
                            .map(|argument| self.type_from_name(argument))
                            .unwrap_or(Type::Error);
                        Type::Result(Box::new(ok), Box::new(err))
                    }
                    _ if self.resolved.structs.contains_key(&head) => Type::Struct(head),
                    _ if self.resolved.enums.contains_key(&head) => Type::Enum(head),
                    _ => Type::Unknown,
                }
            }
            TypeName::Array {
                element, length, ..
            } => Type::Array(Box::new(self.type_from_name(element)), *length),
        }
    }

    fn type_from_ast_ref(&self, ty: &inscribe_ast::nodes::TypeRef) -> Type {
        match &ty.kind {
            inscribe_ast::nodes::TypeRefKind::Path { path, arguments } => {
                let head = path.segments.last().cloned().unwrap_or_default();
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
                            .map(|argument| self.type_from_ast_ref(argument))
                            .unwrap_or(Type::Unknown);
                        let err = arguments
                            .get(1)
                            .map(|argument| self.type_from_ast_ref(argument))
                            .unwrap_or(Type::Error);
                        Type::Result(Box::new(ok), Box::new(err))
                    }
                    _ if self.resolved.structs.contains_key(&head) => Type::Struct(head),
                    _ if self.resolved.enums.contains_key(&head) => Type::Enum(head),
                    _ => Type::Unknown,
                }
            }
            inscribe_ast::nodes::TypeRefKind::Array { element, length } => {
                Type::Array(Box::new(self.type_from_ast_ref(element)), *length)
            }
        }
    }

    fn type_of_literal(&self, literal: &Literal) -> Type {
        match literal {
            Literal::Integer(_) => Type::Int,
            Literal::Float(_) => Type::Float,
            Literal::String(_) => Type::String,
            Literal::Bool(_) => Type::Bool,
        }
    }

    fn lookup_name(&mut self, name: &str, span: Span) -> Type {
        if let Some(binding) = self.lookup_binding(name) {
            return binding.ty;
        }

        if self.resolved.structs.contains_key(name) {
            return Type::Struct(name.to_string());
        }

        if self.resolved.enums.contains_key(name) {
            return Type::Enum(name.to_string());
        }

        if let Some(signature) = self
            .result
            .function_signatures
            .get(&FunctionKey {
                receiver: None,
                name: name.to_string(),
            })
            .cloned()
        {
            return Type::Function(signature);
        }

        self.errors
            .push(TypeError::new(format!("unknown name `{name}`"), span));
        Type::Unknown
    }

    fn check_path_segments(&mut self, segments: &[String], span: Span) -> Type {
        let Some((first, rest)) = segments.split_first() else {
            return Type::Unknown;
        };

        let mut current = self.lookup_name(first, span);
        for segment in rest {
            current = self.follow_member(current, segment, span);
        }
        current
    }

    fn follow_member(&mut self, base_ty: Type, member: &str, span: Span) -> Type {
        match base_ty {
            Type::Struct(name) => {
                if let Some(struct_info) = self.resolved.structs.get(&name) {
                    if let Some(field_ty) = struct_info.fields.get(member) {
                        return self.type_from_name(field_ty);
                    }
                }

                let key = FunctionKey {
                    receiver: Some(name.clone()),
                    name: member.to_string(),
                };
                if let Some(signature) = self.result.function_signatures.get(&key) {
                    return Type::Function(bound_method_signature(signature));
                }

                self.errors.push(TypeError::new(
                    format!("unknown field or method `{member}` on `{name}`"),
                    span,
                ));
                Type::Unknown
            }
            Type::Enum(name) => {
                if self
                    .resolved
                    .enums
                    .get(&name)
                    .is_some_and(|info| info.variants.contains_key(member))
                {
                    Type::Enum(name)
                } else {
                    self.errors.push(TypeError::new(
                        format!("unknown variant `{member}` on `{name}`"),
                        span,
                    ));
                    Type::Unknown
                }
            }
            Type::Unknown => Type::Unknown,
            other => {
                self.errors.push(TypeError::new(
                    format!("cannot access `{member}` on `{}`", other.display_name()),
                    span,
                ));
                Type::Unknown
            }
        }
    }

    fn lookup_binding(&self, name: &str) -> Option<BindingInfo> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).cloned())
    }

    fn define_binding(&mut self, name: String, binding: BindingInfo) {
        let scope = self
            .scopes
            .last_mut()
            .expect("type checker should always keep one scope frame");
        scope.insert(name, binding);
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            let _ = self.scopes.pop();
        }
    }
}

fn bound_method_signature(signature: &FunctionSignature) -> FunctionSignature {
    FunctionSignature {
        key: FunctionKey {
            receiver: None,
            name: signature.key.name.clone(),
        },
        params: signature.params.iter().skip(1).cloned().collect(),
        return_type: signature.return_type.clone(),
    }
}

fn display_function_name(key: &FunctionKey) -> String {
    key.receiver
        .as_ref()
        .map(|receiver| format!("{receiver}.{}", key.name))
        .unwrap_or_else(|| key.name.clone())
}
