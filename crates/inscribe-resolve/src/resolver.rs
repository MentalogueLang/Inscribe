use std::collections::HashMap;
use std::fmt;

use inscribe_ast::nodes::{
    Block, Expr, ExprKind, FunctionDecl, Item, Literal, MatchArm, Module, Pattern, PatternKind,
    Stmt, StructDecl, TypeRef,
};
use inscribe_ast::span::Span;

use crate::cycle_detect::detect_cycles;
use crate::import::ImportTable;
use crate::module_tree::ModuleTree;
use crate::scope::ScopeStack;

// TODO: Record path-to-symbol bindings explicitly once later phases want direct symbol ids.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolveError {
    pub message: String,
    pub span: Span,
}

impl ResolveError {
    pub fn new(message: impl Into<String>, span: Span) -> Self {
        Self {
            message: message.into(),
            span,
        }
    }
}

impl fmt::Display for ResolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} at line {}, column {}",
            self.message, self.span.start.line, self.span.start.column
        )
    }
}

impl std::error::Error for ResolveError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SymbolId(pub usize);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolKind {
    Import,
    Struct,
    Function,
    Local,
    Param,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symbol {
    pub id: SymbolId,
    pub name: String,
    pub kind: SymbolKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FunctionKey {
    pub receiver: Option<String>,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeName {
    pub path: Vec<String>,
    pub arguments: Vec<TypeName>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructInfo {
    pub symbol: SymbolId,
    pub name: String,
    pub fields: HashMap<String, TypeName>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParamInfo {
    pub name: String,
    pub ty: Option<TypeName>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionInfo {
    pub symbol: SymbolId,
    pub key: FunctionKey,
    pub params: Vec<ParamInfo>,
    pub return_type: Option<TypeName>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Builtins {
    types: Vec<&'static str>,
    constructors: Vec<&'static str>,
}

impl Default for Builtins {
    fn default() -> Self {
        Self {
            types: vec!["int", "float", "string", "bool", "Result", "Error"],
            constructors: vec!["Ok", "Err"],
        }
    }
}

impl Builtins {
    pub fn is_type(&self, name: &str) -> bool {
        self.types.iter().any(|entry| entry == &name)
    }

    pub fn is_constructor(&self, name: &str) -> bool {
        self.constructors.iter().any(|entry| entry == &name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedProgram {
    pub module_tree: ModuleTree,
    pub imports: ImportTable,
    pub symbols: Vec<Symbol>,
    pub structs: HashMap<String, StructInfo>,
    pub functions: HashMap<FunctionKey, FunctionInfo>,
    pub builtins: Builtins,
}

impl ResolvedProgram {
    pub fn has_struct(&self, name: &str) -> bool {
        self.structs.contains_key(name)
    }

    pub fn has_function(&self, name: &str) -> bool {
        self.functions
            .keys()
            .any(|key| key.receiver.is_none() && key.name == name)
    }

    pub fn has_method(&self, receiver: &str, name: &str) -> bool {
        self.functions.contains_key(&FunctionKey {
            receiver: Some(receiver.to_string()),
            name: name.to_string(),
        })
    }

    pub fn is_known_type_path(&self, path: &[String]) -> bool {
        path.last()
            .is_some_and(|last| self.builtins.is_type(last) || self.structs.contains_key(last))
    }
}

pub fn resolve_module(module: &Module) -> Result<ResolvedProgram, Vec<ResolveError>> {
    Resolver::default().resolve_module(module)
}

#[derive(Debug, Default)]
pub struct Resolver {
    symbols: Vec<Symbol>,
    structs: HashMap<String, StructInfo>,
    functions: HashMap<FunctionKey, FunctionInfo>,
    builtins: Builtins,
    errors: Vec<ResolveError>,
}

impl Resolver {
    pub fn resolve_module(mut self, module: &Module) -> Result<ResolvedProgram, Vec<ResolveError>> {
        let module_tree = ModuleTree::from_module(module);
        let imports = match ImportTable::from_module(module) {
            Ok(table) => table,
            Err(errors) => {
                self.errors.extend(errors);
                ImportTable::default()
            }
        };

        self.collect_items(module);
        self.validate_module_graph(&imports);
        self.validate_bodies(module, &imports);

        if self.errors.is_empty() {
            Ok(ResolvedProgram {
                module_tree,
                imports,
                symbols: self.symbols,
                structs: self.structs,
                functions: self.functions,
                builtins: self.builtins,
            })
        } else {
            Err(self.errors)
        }
    }

    fn collect_items(&mut self, module: &Module) {
        for item in &module.items {
            match item {
                Item::Import(import) => {
                    let alias = import
                        .path
                        .segments
                        .last()
                        .cloned()
                        .unwrap_or_else(|| "root".to_string());
                    self.push_symbol(alias, SymbolKind::Import, import.span);
                }
                Item::Struct(decl) => self.collect_struct(decl),
                Item::Function(function) => self.collect_function(function),
            }
        }
    }

    fn collect_struct(&mut self, decl: &StructDecl) {
        if self.structs.contains_key(&decl.name) {
            self.errors.push(ResolveError::new(
                format!("duplicate struct `{}`", decl.name),
                decl.span,
            ));
            return;
        }

        let symbol = self.push_symbol(decl.name.clone(), SymbolKind::Struct, decl.span);
        let mut fields = HashMap::new();
        for field in &decl.fields {
            if fields.contains_key(&field.name) {
                self.errors.push(ResolveError::new(
                    format!("duplicate field `{}` in struct `{}`", field.name, decl.name),
                    field.span,
                ));
                continue;
            }

            let type_name = self.resolve_type_name(&field.ty);
            fields.insert(field.name.clone(), type_name);
        }

        self.structs.insert(
            decl.name.clone(),
            StructInfo {
                symbol,
                name: decl.name.clone(),
                fields,
                span: decl.span,
            },
        );
    }

    fn collect_function(&mut self, function: &FunctionDecl) {
        let key = FunctionKey {
            receiver: function
                .receiver
                .as_ref()
                .map(|path| path.segments.join(".")),
            name: function.name.clone(),
        };

        if self.functions.contains_key(&key) {
            let display_name = key
                .receiver
                .as_ref()
                .map(|receiver| format!("{receiver}.{}", key.name))
                .unwrap_or_else(|| key.name.clone());
            self.errors.push(ResolveError::new(
                format!("duplicate function `{display_name}`"),
                function.span,
            ));
            return;
        }

        if let Some(receiver) = &key.receiver {
            if !self.structs.contains_key(receiver) {
                self.errors.push(ResolveError::new(
                    format!("unknown method receiver type `{receiver}`"),
                    function.span,
                ));
            }
        }

        let symbol = self.push_symbol(key.name.clone(), SymbolKind::Function, function.span);
        let mut params = Vec::new();
        let mut seen_params = HashMap::<String, Span>::new();

        for param in &function.params {
            if seen_params.insert(param.name.clone(), param.span).is_some() {
                self.errors.push(ResolveError::new(
                    format!("duplicate parameter `{}`", param.name),
                    param.span,
                ));
            }

            params.push(ParamInfo {
                name: param.name.clone(),
                ty: param.ty.as_ref().map(|ty| self.resolve_type_name(ty)),
                span: param.span,
            });
        }

        let return_type = function
            .return_type
            .as_ref()
            .map(|ty| self.resolve_type_name(ty));
        self.functions.insert(
            key.clone(),
            FunctionInfo {
                symbol,
                key,
                params,
                return_type,
                span: function.span,
            },
        );
    }

    fn validate_module_graph(&mut self, imports: &ImportTable) {
        let mut graph = HashMap::<String, Vec<String>>::new();
        graph.insert(
            "root".to_string(),
            imports.entries().map(|entry| entry.alias.clone()).collect(),
        );
        for entry in imports.entries() {
            graph.entry(entry.alias.clone()).or_default();
        }

        for cycle in detect_cycles(&graph) {
            self.errors.push(ResolveError::new(
                format!("import cycle detected: {}", cycle.join(" -> ")),
                Span::default(),
            ));
        }
    }

    fn validate_bodies(&mut self, module: &Module, imports: &ImportTable) {
        for item in &module.items {
            let Item::Function(function) = item else {
                continue;
            };

            let mut scope = ScopeStack::new();
            for param in &function.params {
                let inserted = scope.define(
                    param.name.clone(),
                    self.push_symbol(param.name.clone(), SymbolKind::Param, param.span),
                );
                if !inserted {
                    self.errors.push(ResolveError::new(
                        format!("duplicate parameter `{}`", param.name),
                        param.span,
                    ));
                }
            }

            if let Some(body) = &function.body {
                self.resolve_block(body, &mut scope, imports);
            }
        }
    }

    fn resolve_block(
        &mut self,
        block: &Block,
        scope: &mut ScopeStack<SymbolId>,
        imports: &ImportTable,
    ) {
        scope.push();
        for statement in &block.statements {
            self.resolve_statement(statement, scope, imports);
        }
        scope.pop();
    }

    fn resolve_statement(
        &mut self,
        statement: &Stmt,
        scope: &mut ScopeStack<SymbolId>,
        imports: &ImportTable,
    ) {
        match statement {
            Stmt::Let(stmt) => {
                self.resolve_expr(&stmt.value, scope, imports);
                let inserted = scope.define(
                    stmt.name.clone(),
                    self.push_symbol(stmt.name.clone(), SymbolKind::Local, stmt.span),
                );
                if !inserted {
                    self.errors.push(ResolveError::new(
                        format!("duplicate local `{}`", stmt.name),
                        stmt.span,
                    ));
                }
                if let Some(ty) = &stmt.ty {
                    let _ = self.resolve_type_name(ty);
                }
            }
            Stmt::Const(stmt) => {
                self.resolve_expr(&stmt.value, scope, imports);
                let inserted = scope.define(
                    stmt.name.clone(),
                    self.push_symbol(stmt.name.clone(), SymbolKind::Local, stmt.span),
                );
                if !inserted {
                    self.errors.push(ResolveError::new(
                        format!("duplicate local `{}`", stmt.name),
                        stmt.span,
                    ));
                }
                if let Some(ty) = &stmt.ty {
                    let _ = self.resolve_type_name(ty);
                }
            }
            Stmt::For(stmt) => {
                self.resolve_expr(&stmt.iterable, scope, imports);
                scope.push();
                self.bind_pattern(&stmt.pattern, scope);
                self.resolve_block(&stmt.body, scope, imports);
                scope.pop();
            }
            Stmt::While(stmt) => {
                self.resolve_expr(&stmt.condition, scope, imports);
                self.resolve_block(&stmt.body, scope, imports);
            }
            Stmt::Return(stmt) => {
                if let Some(value) = &stmt.value {
                    self.resolve_expr(value, scope, imports);
                }
            }
            Stmt::Expr(expr) => self.resolve_expr(expr, scope, imports),
        }
    }

    fn resolve_expr(
        &mut self,
        expr: &Expr,
        scope: &mut ScopeStack<SymbolId>,
        imports: &ImportTable,
    ) {
        match &expr.kind {
            ExprKind::Literal(_) => {}
            ExprKind::Path(path) => {
                if !self.resolve_value_path(path.segments.as_slice(), scope, imports) {
                    self.errors.push(ResolveError::new(
                        format!("unknown name `{}`", path.segments.join(".")),
                        path.span,
                    ));
                }
            }
            ExprKind::Unary { expr, .. } | ExprKind::Try(expr) => {
                self.resolve_expr(expr, scope, imports);
            }
            ExprKind::Binary { left, right, .. } => {
                self.resolve_expr(left, scope, imports);
                self.resolve_expr(right, scope, imports);
            }
            ExprKind::Call { callee, args } => {
                self.resolve_expr(callee, scope, imports);
                for arg in args {
                    self.resolve_expr(arg, scope, imports);
                }
            }
            ExprKind::Field { base, .. } => self.resolve_expr(base, scope, imports),
            ExprKind::StructLiteral { path, fields } => {
                if !self
                    .structs
                    .contains_key(path.segments.last().unwrap_or(&String::new()))
                {
                    self.errors.push(ResolveError::new(
                        format!("unknown struct `{}`", path.segments.join(".")),
                        path.span,
                    ));
                }

                let known_fields = path
                    .segments
                    .last()
                    .and_then(|name| self.structs.get(name))
                    .map(|info| info.fields.clone())
                    .unwrap_or_default();

                for field in fields {
                    if !known_fields.contains_key(&field.name) {
                        self.errors.push(ResolveError::new(
                            format!(
                                "unknown field `{}` for struct `{}`",
                                field.name,
                                path.segments.join(".")
                            ),
                            field.span,
                        ));
                    }
                    self.resolve_expr(&field.value, scope, imports);
                }
            }
            ExprKind::If {
                condition,
                then_block,
                else_branch,
            } => {
                self.resolve_expr(condition, scope, imports);
                self.resolve_block(then_block, scope, imports);
                if let Some(else_branch) = else_branch {
                    self.resolve_expr(else_branch, scope, imports);
                }
            }
            ExprKind::Match { value, arms } => {
                self.resolve_expr(value, scope, imports);
                for MatchArm {
                    pattern,
                    value,
                    span: _,
                } in arms
                {
                    scope.push();
                    self.bind_pattern(pattern, scope);
                    self.resolve_expr(value, scope, imports);
                    scope.pop();
                }
            }
            ExprKind::Block(block) => self.resolve_block(block, scope, imports),
        }
    }

    fn bind_pattern(&mut self, pattern: &Pattern, scope: &mut ScopeStack<SymbolId>) {
        match &pattern.kind {
            PatternKind::Wildcard
            | PatternKind::Literal(Literal::Bool(_))
            | PatternKind::Literal(Literal::Integer(_))
            | PatternKind::Literal(Literal::Float(_))
            | PatternKind::Literal(Literal::String(_))
            | PatternKind::Path(_) => {}
            PatternKind::Binding(name) => {
                let inserted = scope.define(
                    name.clone(),
                    self.push_symbol(name.clone(), SymbolKind::Local, pattern.span),
                );
                if !inserted {
                    self.errors.push(ResolveError::new(
                        format!("duplicate binding `{name}` in pattern"),
                        pattern.span,
                    ));
                }
            }
            PatternKind::Constructor { path, arguments } => {
                if let Some(head) = path.segments.first() {
                    if !self.builtins.is_constructor(head)
                        && !self.structs.contains_key(head)
                        && !self.functions.contains_key(&FunctionKey {
                            receiver: None,
                            name: head.clone(),
                        })
                    {
                        self.errors.push(ResolveError::new(
                            format!("unknown pattern constructor `{}`", path.segments.join(".")),
                            path.span,
                        ));
                    }
                }
                for argument in arguments {
                    self.bind_pattern(argument, scope);
                }
            }
        }
    }

    fn resolve_value_path(
        &self,
        segments: &[String],
        scope: &ScopeStack<SymbolId>,
        imports: &ImportTable,
    ) -> bool {
        let Some(head) = segments.first() else {
            return false;
        };

        if segments.len() == 1 {
            scope.lookup(head).is_some()
                || self.structs.contains_key(head)
                || self.has_toplevel_function(head)
                || imports.contains_alias(head)
                || self.builtins.is_constructor(head)
        } else {
            scope.lookup(head).is_some()
                || imports.contains_alias(head)
                || self.structs.contains_key(head)
                || self.has_toplevel_function(head)
        }
    }

    fn resolve_type_name(&mut self, ty: &TypeRef) -> TypeName {
        for argument in &ty.arguments {
            let _ = self.resolve_type_name(argument);
        }

        if !self.is_known_type_path(ty.path.segments.as_slice())
            && ty.path.segments.first().is_some()
        {
            self.errors.push(ResolveError::new(
                format!("unknown type `{}`", ty.path.segments.join(".")),
                ty.span,
            ));
        }

        TypeName {
            path: ty.path.segments.clone(),
            arguments: ty
                .arguments
                .iter()
                .map(|argument| self.resolve_type_name(argument))
                .collect(),
            span: ty.span,
        }
    }

    fn is_known_type_path(&self, path: &[String]) -> bool {
        path.last()
            .is_some_and(|last| self.builtins.is_type(last) || self.structs.contains_key(last))
    }

    fn has_toplevel_function(&self, name: &str) -> bool {
        self.functions
            .keys()
            .any(|key| key.receiver.is_none() && key.name == name)
    }

    fn push_symbol(&mut self, name: String, kind: SymbolKind, span: Span) -> SymbolId {
        let id = SymbolId(self.symbols.len());
        self.symbols.push(Symbol {
            id,
            name,
            kind,
            span,
        });
        id
    }
}
