use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use inscribe_ast::nodes::{
    Block, EnumDecl, EnumVariant, Expr, ExprKind, FunctionDecl, Import, Item, MatchArm, Module,
    Param, Path as AstPath, Pattern, PatternKind, Stmt, StructDecl, StructField,
    StructLiteralField, TypeRef, TypeRefKind, Visibility,
};
use inscribe_ast::span::{Position, Span};
use inscribe_parser::parse_module;
use inscribe_session::SessionError;
use serde::{Deserialize, Serialize};

use crate::module_tree::{ImportNode, ItemNode, ModuleNode, ModuleTree};
use crate::resolver::FunctionKey;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct SourceModule {
    pub path: PathBuf,
    pub module: Module,
    pub imports: Vec<PathBuf>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct LoadedModuleGraph {
    pub entry: PathBuf,
    pub modules: Vec<SourceModule>,
    pub merged: Module,
    pub tree: ModuleTree,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ModuleLoadOptions {
    pub stdlib_root: PathBuf,
}

impl Default for ModuleLoadOptions {
    fn default() -> Self {
        Self {
            stdlib_root: workspace_root().join("stdlib"),
        }
    }
}

pub fn load_module_graph(entry: &Path) -> Result<LoadedModuleGraph, SessionError> {
    load_module_graph_with_options(entry, &ModuleLoadOptions::default())
}

pub fn load_module_graph_with_options(
    entry: &Path,
    options: &ModuleLoadOptions,
) -> Result<LoadedModuleGraph, SessionError> {
    let entry = entry.canonicalize().map_err(|error| {
        SessionError::new(
            "io",
            format!("failed to resolve `{}`: {error}", entry.display()),
        )
    })?;
    let mut loaded = HashMap::new();
    let mut stack = HashSet::new();
    let mut next_span_base = 0;
    load_recursive(
        &entry,
        options,
        &mut loaded,
        &mut stack,
        &mut next_span_base,
    )?;

    let mut order = Vec::new();
    collect_order(&entry, &loaded, &mut HashSet::new(), &mut order);

    let mut items = Vec::new();
    let mut modules = Vec::new();
    let mut nodes = Vec::new();

    for path in &order {
        let source = loaded.get(path).expect("ordered module should exist");
        items.extend(merged_items_for_module(&entry, source));
        modules.push(source.clone());
        nodes.push(module_node(source));
    }

    let span = loaded
        .get(&entry)
        .map(|source| source.module.span)
        .unwrap_or_default();

    Ok(LoadedModuleGraph {
        entry: entry.clone(),
        modules,
        merged: Module { items, span },
        tree: ModuleTree::from_nodes(entry, nodes),
    })
}

fn load_recursive(
    path: &Path,
    options: &ModuleLoadOptions,
    loaded: &mut HashMap<PathBuf, SourceModule>,
    stack: &mut HashSet<PathBuf>,
    next_span_base: &mut usize,
) -> Result<(), SessionError> {
    if loaded.contains_key(path) {
        return Ok(());
    }

    if !stack.insert(path.to_path_buf()) {
        return Err(SessionError::new(
            "include",
            format!("import cycle detected while loading `{}`", path.display()),
        ));
    }

    let source_text = fs::read_to_string(path).map_err(|error| {
        SessionError::new(
            "io",
            format!("failed to read `{}`: {error}", path.display()),
        )
    })?;
    let tokens = inscribe_lexer::lex(&source_text)
        .map_err(|error| SessionError::new("lex", error.to_string()))?;
    let mut module =
        parse_module(tokens).map_err(|error| SessionError::new("parse", error.to_string()))?;
    rebase_module_spans(&mut module, *next_span_base);
    *next_span_base += source_text.len().max(1) + 1;

    let mut imports = Vec::new();
    for item in &module.items {
        let Item::Import(import) = item else {
            continue;
        };

        let import_path = resolve_import_path(path, &import.path.segments, options)?;
        imports.push(import_path.clone());
        load_recursive(&import_path, options, loaded, stack, next_span_base)?;
    }

    stack.remove(path);
    loaded.insert(
        path.to_path_buf(),
        SourceModule {
            path: path.to_path_buf(),
            module,
            imports,
        },
    );
    Ok(())
}

fn collect_order(
    path: &PathBuf,
    loaded: &HashMap<PathBuf, SourceModule>,
    visited: &mut HashSet<PathBuf>,
    order: &mut Vec<PathBuf>,
) {
    if !visited.insert(path.clone()) {
        return;
    }

    if let Some(module) = loaded.get(path) {
        for import in &module.imports {
            collect_order(import, loaded, visited, order);
        }
    }

    order.push(path.clone());
}

fn module_node(source: &SourceModule) -> ModuleNode {
    let imports = source
        .module
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Import(import) => Some(ImportNode {
                path: import.path.segments.clone(),
                resolved_path: source
                    .imports
                    .iter()
                    .find(|candidate| {
                        candidate
                            .file_stem()
                            .and_then(|stem| stem.to_str())
                            .is_some_and(|stem| {
                                stem == import.path.segments.last().unwrap_or(&String::new())
                            })
                    })
                    .cloned(),
                span: import.span,
            }),
            _ => None,
        })
        .collect();

    let items = source
        .module
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Struct(decl) => Some(ItemNode::Struct {
                name: decl.name.clone(),
                span: decl.span,
            }),
            Item::Enum(decl) => Some(ItemNode::Enum {
                name: decl.name.clone(),
                span: decl.span,
            }),
            Item::Function(function) => Some(ItemNode::Function {
                key: FunctionKey {
                    receiver: function
                        .receiver
                        .as_ref()
                        .map(|path| path.segments.join(".")),
                    name: function.name.clone(),
                },
                visibility: function.visibility,
                span: function.span,
            }),
            Item::Import(_) => None,
        })
        .collect();

    ModuleNode {
        name: source
            .path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("root")
            .to_string(),
        path: source.path.clone(),
        imports,
        items,
        span: source.module.span,
    }
}

fn merged_items_for_module(entry: &Path, source: &SourceModule) -> Vec<Item> {
    let mut module = source.module.clone();
    if source.path != entry {
        rewrite_private_functions(&mut module, &source.path);
    }

    module
        .items
        .into_iter()
        .filter(|item| !matches!(item, Item::Import(_)))
        .collect()
}

fn rewrite_private_functions(module: &mut Module, source_path: &Path) {
    let private_names = module
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Function(function)
                if function.visibility == Visibility::Private && function.receiver.is_none() =>
            {
                Some((
                    function.name.clone(),
                    hidden_private_name(source_path, &function.name),
                ))
            }
            _ => None,
        })
        .collect::<HashMap<_, _>>();

    if private_names.is_empty() {
        return;
    }

    for item in &mut module.items {
        let Item::Function(function) = item else {
            continue;
        };

        if let Some(hidden) = private_names.get(&function.name) {
            if function.visibility == Visibility::Private && function.receiver.is_none() {
                function.name = hidden.clone();
            }
        }

        if let Some(body) = &mut function.body {
            rewrite_block_private_paths(body, &private_names);
        }
    }
}

fn rewrite_block_private_paths(block: &mut Block, private_names: &HashMap<String, String>) {
    for statement in &mut block.statements {
        rewrite_statement_private_paths(statement, private_names);
    }
}

fn rewrite_statement_private_paths(statement: &mut Stmt, private_names: &HashMap<String, String>) {
    match statement {
        Stmt::Let(stmt) => rewrite_expr_private_paths(&mut stmt.value, private_names),
        Stmt::Const(stmt) => rewrite_expr_private_paths(&mut stmt.value, private_names),
        Stmt::For(stmt) => {
            rewrite_expr_private_paths(&mut stmt.iterable, private_names);
            rewrite_block_private_paths(&mut stmt.body, private_names);
        }
        Stmt::While(stmt) => {
            rewrite_expr_private_paths(&mut stmt.condition, private_names);
            rewrite_block_private_paths(&mut stmt.body, private_names);
        }
        Stmt::Return(stmt) => {
            if let Some(value) = &mut stmt.value {
                rewrite_expr_private_paths(value, private_names);
            }
        }
        Stmt::Expr(expr) => rewrite_expr_private_paths(expr, private_names),
    }
}

fn rewrite_expr_private_paths(expr: &mut Expr, private_names: &HashMap<String, String>) {
    match &mut expr.kind {
        ExprKind::Path(path) => {
            if path.segments.len() == 1 {
                if let Some(hidden) = private_names.get(&path.segments[0]) {
                    path.segments[0] = hidden.clone();
                }
            }
        }
        ExprKind::Array(items) => {
            for item in items {
                rewrite_expr_private_paths(item, private_names);
            }
        }
        ExprKind::RepeatArray { value, .. } => rewrite_expr_private_paths(value, private_names),
        ExprKind::Cast { expr: inner, .. } => rewrite_expr_private_paths(inner, private_names),
        ExprKind::Unary { expr: inner, .. } => rewrite_expr_private_paths(inner, private_names),
        ExprKind::Binary { left, right, .. } => {
            rewrite_expr_private_paths(left, private_names);
            rewrite_expr_private_paths(right, private_names);
        }
        ExprKind::Call { callee, args } => {
            rewrite_expr_private_paths(callee, private_names);
            for arg in args {
                rewrite_expr_private_paths(arg, private_names);
            }
        }
        ExprKind::Field { base, .. } => rewrite_expr_private_paths(base, private_names),
        ExprKind::Index { target, index } => {
            rewrite_expr_private_paths(target, private_names);
            rewrite_expr_private_paths(index, private_names);
        }
        ExprKind::StructLiteral { fields, .. } => {
            for field in fields {
                rewrite_expr_private_paths(&mut field.value, private_names);
            }
        }
        ExprKind::If {
            condition,
            then_block,
            else_branch,
        } => {
            rewrite_expr_private_paths(condition, private_names);
            rewrite_block_private_paths(then_block, private_names);
            if let Some(branch) = else_branch {
                rewrite_expr_private_paths(branch, private_names);
            }
        }
        ExprKind::Match { value, arms } => {
            rewrite_expr_private_paths(value, private_names);
            for arm in arms {
                rewrite_expr_private_paths(&mut arm.value, private_names);
            }
        }
        ExprKind::Block(block) => rewrite_block_private_paths(block, private_names),
        ExprKind::Try(inner) => rewrite_expr_private_paths(inner, private_names),
        ExprKind::Literal(_) => {}
    }
}

fn hidden_private_name(source_path: &Path, name: &str) -> String {
    format!(
        "__priv_{:016x}_{}",
        stable_symbol_hash(&source_path.display().to_string()),
        sanitize_symbol(name)
    )
}

fn sanitize_symbol(value: &str) -> String {
    value
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect()
}

fn stable_symbol_hash(value: &str) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in value.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn resolve_import_path(
    current_file: &Path,
    segments: &[String],
    options: &ModuleLoadOptions,
) -> Result<PathBuf, SessionError> {
    if segments.is_empty() {
        return Err(SessionError::new("include", "import path cannot be empty"));
    }

    let std_segments = if segments.first().is_some_and(|segment| segment == "std") {
        &segments[1..]
    } else {
        segments
    };

    if !std_segments.is_empty() {
        let candidate = std_segments
            .iter()
            .fold(options.stdlib_root.clone(), |path, segment| {
                path.join(segment)
            })
            .with_extension("mtl");
        if candidate.exists() {
            return candidate.canonicalize().map_err(|error| {
                SessionError::new(
                    "include",
                    format!(
                        "failed to resolve stdlib import `{}`: {error}",
                        candidate.display()
                    ),
                )
            });
        }
    }

    let base_dir = current_file.parent().ok_or_else(|| {
        SessionError::new(
            "include",
            format!(
                "cannot resolve imports relative to `{}`",
                current_file.display()
            ),
        )
    })?;
    let candidate = segments
        .iter()
        .fold(base_dir.to_path_buf(), |path, segment| path.join(segment))
        .with_extension("mtl");
    candidate.canonicalize().map_err(|error| {
        SessionError::new(
            "include",
            format!(
                "failed to resolve import `{}` from `{}`: {error}",
                segments.join("."),
                current_file.display()
            ),
        )
    })
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("resolve crate should live under the workspace")
        .to_path_buf()
}

fn rebase_module_spans(module: &mut Module, base_offset: usize) {
    shift_span(&mut module.span, base_offset);
    for item in &mut module.items {
        rebase_item_spans(item, base_offset);
    }
}

fn rebase_item_spans(item: &mut Item, base_offset: usize) {
    match item {
        Item::Import(import) => rebase_import_spans(import, base_offset),
        Item::Struct(decl) => rebase_struct_spans(decl, base_offset),
        Item::Enum(decl) => rebase_enum_spans(decl, base_offset),
        Item::Function(function) => rebase_function_spans(function, base_offset),
    }
}

fn rebase_import_spans(import: &mut Import, base_offset: usize) {
    shift_span(&mut import.span, base_offset);
    rebase_path_spans(&mut import.path, base_offset);
}

fn rebase_struct_spans(decl: &mut StructDecl, base_offset: usize) {
    shift_span(&mut decl.name_span, base_offset);
    shift_span(&mut decl.span, base_offset);
    for field in &mut decl.fields {
        rebase_struct_field_spans(field, base_offset);
    }
}

fn rebase_struct_field_spans(field: &mut StructField, base_offset: usize) {
    shift_span(&mut field.name_span, base_offset);
    shift_span(&mut field.span, base_offset);
    rebase_type_ref_spans(&mut field.ty, base_offset);
}

fn rebase_enum_spans(decl: &mut EnumDecl, base_offset: usize) {
    shift_span(&mut decl.name_span, base_offset);
    shift_span(&mut decl.span, base_offset);
    for variant in &mut decl.variants {
        rebase_enum_variant_spans(variant, base_offset);
    }
}

fn rebase_enum_variant_spans(variant: &mut EnumVariant, base_offset: usize) {
    shift_span(&mut variant.name_span, base_offset);
    shift_span(&mut variant.span, base_offset);
}

fn rebase_function_spans(function: &mut FunctionDecl, base_offset: usize) {
    shift_span(&mut function.name_span, base_offset);
    shift_span(&mut function.span, base_offset);
    if let Some(receiver) = &mut function.receiver {
        rebase_path_spans(receiver, base_offset);
    }
    for param in &mut function.params {
        rebase_param_spans(param, base_offset);
    }
    if let Some(return_type) = &mut function.return_type {
        rebase_type_ref_spans(return_type, base_offset);
    }
    if let Some(body) = &mut function.body {
        rebase_block_spans(body, base_offset);
    }
}

fn rebase_param_spans(param: &mut Param, base_offset: usize) {
    shift_span(&mut param.name_span, base_offset);
    shift_span(&mut param.span, base_offset);
    if let Some(ty) = &mut param.ty {
        rebase_type_ref_spans(ty, base_offset);
    }
}

fn rebase_block_spans(block: &mut Block, base_offset: usize) {
    shift_span(&mut block.span, base_offset);
    for statement in &mut block.statements {
        rebase_statement_spans(statement, base_offset);
    }
}

fn rebase_statement_spans(statement: &mut Stmt, base_offset: usize) {
    match statement {
        Stmt::Let(stmt) => {
            shift_span(&mut stmt.name_span, base_offset);
            shift_span(&mut stmt.span, base_offset);
            if let Some(ty) = &mut stmt.ty {
                rebase_type_ref_spans(ty, base_offset);
            }
            rebase_expr_spans(&mut stmt.value, base_offset);
        }
        Stmt::Const(stmt) => {
            shift_span(&mut stmt.name_span, base_offset);
            shift_span(&mut stmt.span, base_offset);
            if let Some(ty) = &mut stmt.ty {
                rebase_type_ref_spans(ty, base_offset);
            }
            rebase_expr_spans(&mut stmt.value, base_offset);
        }
        Stmt::For(stmt) => {
            shift_span(&mut stmt.span, base_offset);
            rebase_pattern_spans(&mut stmt.pattern, base_offset);
            rebase_expr_spans(&mut stmt.iterable, base_offset);
            rebase_block_spans(&mut stmt.body, base_offset);
        }
        Stmt::While(stmt) => {
            shift_span(&mut stmt.span, base_offset);
            rebase_expr_spans(&mut stmt.condition, base_offset);
            rebase_block_spans(&mut stmt.body, base_offset);
        }
        Stmt::Return(stmt) => {
            shift_span(&mut stmt.span, base_offset);
            if let Some(value) = &mut stmt.value {
                rebase_expr_spans(value, base_offset);
            }
        }
        Stmt::Expr(expr) => rebase_expr_spans(expr, base_offset),
    }
}

fn rebase_expr_spans(expr: &mut Expr, base_offset: usize) {
    shift_span(&mut expr.span, base_offset);
    match &mut expr.kind {
        ExprKind::Literal(_) => {}
        ExprKind::Path(path) => rebase_path_spans(path, base_offset),
        ExprKind::Array(items) => {
            for item in items {
                rebase_expr_spans(item, base_offset);
            }
        }
        ExprKind::RepeatArray { value, .. } => rebase_expr_spans(value, base_offset),
        ExprKind::Cast { expr: inner, ty } => {
            rebase_expr_spans(inner, base_offset);
            rebase_type_ref_spans(ty, base_offset);
        }
        ExprKind::Unary { expr: inner, .. } => rebase_expr_spans(inner, base_offset),
        ExprKind::Binary { left, right, .. } => {
            rebase_expr_spans(left, base_offset);
            rebase_expr_spans(right, base_offset);
        }
        ExprKind::Call { callee, args } => {
            rebase_expr_spans(callee, base_offset);
            for arg in args {
                rebase_expr_spans(arg, base_offset);
            }
        }
        ExprKind::Field { base, .. } => rebase_expr_spans(base, base_offset),
        ExprKind::Index { target, index } => {
            rebase_expr_spans(target, base_offset);
            rebase_expr_spans(index, base_offset);
        }
        ExprKind::StructLiteral { path, fields } => {
            rebase_path_spans(path, base_offset);
            for field in fields {
                rebase_struct_literal_field_spans(field, base_offset);
            }
        }
        ExprKind::If {
            condition,
            then_block,
            else_branch,
        } => {
            rebase_expr_spans(condition, base_offset);
            rebase_block_spans(then_block, base_offset);
            if let Some(branch) = else_branch {
                rebase_expr_spans(branch, base_offset);
            }
        }
        ExprKind::Match { value, arms } => {
            rebase_expr_spans(value, base_offset);
            for arm in arms {
                rebase_match_arm_spans(arm, base_offset);
            }
        }
        ExprKind::Block(block) => rebase_block_spans(block, base_offset),
        ExprKind::Try(inner) => rebase_expr_spans(inner, base_offset),
    }
}

fn rebase_struct_literal_field_spans(field: &mut StructLiteralField, base_offset: usize) {
    shift_span(&mut field.name_span, base_offset);
    shift_span(&mut field.span, base_offset);
    rebase_expr_spans(&mut field.value, base_offset);
}

fn rebase_match_arm_spans(arm: &mut MatchArm, base_offset: usize) {
    shift_span(&mut arm.span, base_offset);
    rebase_pattern_spans(&mut arm.pattern, base_offset);
    rebase_expr_spans(&mut arm.value, base_offset);
}

fn rebase_pattern_spans(pattern: &mut Pattern, base_offset: usize) {
    shift_span(&mut pattern.span, base_offset);
    match &mut pattern.kind {
        PatternKind::Wildcard | PatternKind::Binding(_) | PatternKind::Literal(_) => {}
        PatternKind::Path(path) => rebase_path_spans(path, base_offset),
        PatternKind::Constructor { path, arguments } => {
            rebase_path_spans(path, base_offset);
            for argument in arguments {
                rebase_pattern_spans(argument, base_offset);
            }
        }
    }
}

fn rebase_type_ref_spans(ty: &mut TypeRef, base_offset: usize) {
    shift_span(&mut ty.span, base_offset);
    match &mut ty.kind {
        TypeRefKind::Path { path, arguments } => {
            rebase_path_spans(path, base_offset);
            for argument in arguments {
                rebase_type_ref_spans(argument, base_offset);
            }
        }
        TypeRefKind::Array { element, .. } => rebase_type_ref_spans(element, base_offset),
    }
}

fn rebase_path_spans(path: &mut AstPath, base_offset: usize) {
    shift_span(&mut path.span, base_offset);
    for segment_span in &mut path.segment_spans {
        shift_span(segment_span, base_offset);
    }
}

fn shift_span(span: &mut Span, base_offset: usize) {
    shift_position(&mut span.start, base_offset);
    shift_position(&mut span.end, base_offset);
}

fn shift_position(position: &mut Position, base_offset: usize) {
    position.offset += base_offset;
}
