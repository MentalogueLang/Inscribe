use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use inscribe_abi::{MlibExportKind, MlibFile};
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
            stdlib_root: detect_stdlib_root(),
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
        SessionError::new("io", format!("failed to read `{}`: {error}", path.display()))
    });
    let module = if path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("mlib"))
    {
        load_mlib_module(path, next_span_base)?
    } else {
        let source_text = source_text?;
        let tokens = inscribe_lexer::lex(&source_text)
            .map_err(|error| SessionError::new("lex", error.to_string()))?;
        let mut module =
            parse_module(tokens).map_err(|error| SessionError::new("parse", error.to_string()))?;
        rebase_module_spans(&mut module, *next_span_base);
        *next_span_base += source_text.len().max(1) + 1;
        module
    };

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
    if candidate.exists() {
        return candidate.canonicalize().map_err(|error| {
            SessionError::new(
                "include",
                format!(
                    "failed to resolve import `{}` from `{}`: {error}",
                    segments.join("."),
                    current_file.display()
                ),
            )
        });
    }

    if let Some(candidate) = resolve_suture_import_path(current_file, segments)? {
        return Ok(candidate);
    }

    Err(SessionError::new(
        "include",
        format!(
            "failed to resolve import `{}` from `{}`",
            segments.join("."),
            current_file.display()
        ),
    ))
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("resolve crate should live under the workspace")
        .to_path_buf()
}

fn detect_stdlib_root() -> PathBuf {
    if let Ok(value) = std::env::var("INSCRIBE_STDLIB_DIR") {
        let path = PathBuf::from(value);
        if path.exists() {
            return path;
        }
    }

    for candidate in installed_stdlib_candidates() {
        if candidate.exists() {
            return candidate;
        }
    }

    workspace_root().join("stdlib")
}

fn installed_stdlib_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            candidates.push(parent.join("stdlib"));
            if let Some(grandparent) = parent.parent() {
                candidates.push(grandparent.join("stdlib"));
            }
        }
    }

    if let Some(home) = home_dir() {
        candidates.push(home.join(".mentalogue").join("inscribe").join("stdlib"));
    }

    if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        candidates.push(
            PathBuf::from(local_app_data)
                .join("Mentalogue")
                .join("Inscribe")
                .join("stdlib"),
        );
    }

    candidates
}

fn home_dir() -> Option<PathBuf> {
    if let Ok(value) = std::env::var("HOME") {
        if !value.is_empty() {
            return Some(PathBuf::from(value));
        }
    }
    if let Ok(value) = std::env::var("USERPROFILE") {
        if !value.is_empty() {
            return Some(PathBuf::from(value));
        }
    }
    None
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

fn resolve_suture_import_path(
    current_file: &Path,
    segments: &[String],
) -> Result<Option<PathBuf>, SessionError> {
    let package = segments.join(".");
    let Some(mut current) = current_file.parent().map(Path::to_path_buf) else {
        return Ok(None);
    };

    loop {
        let package_root = current.join(".suture").join("mlib").join(&package);
        if package_root.is_dir() {
            let direct = package_root.join(format!("{package}.mlib"));
            if direct.exists() {
                return direct.canonicalize().map(Some).map_err(|error| {
                    SessionError::new(
                        "include",
                        format!("failed to resolve `{}`: {error}", direct.display()),
                    )
                });
            }

            let mut versions = fs::read_dir(&package_root)
                .map_err(|error| {
                    SessionError::new(
                        "include",
                        format!("failed to read `{}`: {error}", package_root.display()),
                    )
                })?
                .filter_map(|entry| entry.ok())
                .map(|entry| entry.path())
                .filter(|path| path.is_dir())
                .collect::<Vec<_>>();
            versions.sort();
            versions.reverse();
            for version_dir in versions {
                let candidate = version_dir.join(format!("{package}.mlib"));
                if candidate.exists() {
                    return candidate.canonicalize().map(Some).map_err(|error| {
                        SessionError::new(
                            "include",
                            format!("failed to resolve `{}`: {error}", candidate.display()),
                        )
                    });
                }
            }
        }

        if !current.pop() {
            break;
        }
    }

    Ok(None)
}

fn load_mlib_module(path: &Path, next_span_base: &mut usize) -> Result<Module, SessionError> {
    let bytes = fs::read(path).map_err(|error| {
        SessionError::new(
            "io",
            format!("failed to read `{}`: {error}", path.display()),
        )
    })?;
    let file = MlibFile::from_bytes(&bytes).ok_or_else(|| {
        SessionError::new(
            "include",
            format!("failed to decode MLIB `{}`", path.display()),
        )
    })?;

    let mut custom_types = HashSet::new();
    let mut declared_types = HashSet::new();
    let mut items = Vec::new();
    for export in &file.exports {
        if export.kind != MlibExportKind::Type {
            continue;
        }
        let signature = export.signature.as_deref().ok_or_else(|| {
            SessionError::new(
                "include",
                format!("MLIB type export `{}` is missing metadata", export.name),
            )
        })?;
        let signature = std::str::from_utf8(signature).map_err(|error| {
            SessionError::new(
                "include",
                format!("MLIB type export `{}` has invalid metadata: {error}", export.name),
            )
        })?;
        items.push(synthetic_type_item_from_export(
            export.name.as_str(),
            signature,
            next_span_base,
            &mut custom_types,
        )?);
        declared_types.insert(export.name.clone());
    }

    let mut functions = Vec::new();
    for export in &file.exports {
        if export.kind != MlibExportKind::Function {
            continue;
        }
        let signature = export.signature.as_deref().ok_or_else(|| {
            SessionError::new(
                "include",
                format!("MLIB export `{}` is missing a function signature", export.name),
            )
        })?;
        let signature = std::str::from_utf8(signature).map_err(|error| {
            SessionError::new(
                "include",
                format!("MLIB export `{}` has an invalid signature: {error}", export.name),
            )
        })?;
        functions.push(synthetic_function_from_export(
            export.name.as_str(),
            signature,
            next_span_base,
            &mut custom_types,
        )?);
    }

    let mut type_names = custom_types.into_iter().collect::<Vec<_>>();
    type_names.sort();
    items.extend(type_names
        .into_iter()
        .filter(|name| !declared_types.contains(name))
        .map(|name| Item::Struct(synthetic_struct_decl(name, next_span_base)))
    );
    items.extend(functions.into_iter().map(Item::Function));
    let span = fresh_span(next_span_base);
    Ok(Module { items, span })
}

fn synthetic_type_item_from_export(
    export_name: &str,
    signature: &str,
    next_span_base: &mut usize,
    custom_types: &mut HashSet<String>,
) -> Result<Item, SessionError> {
    let source = signature.trim();
    if let Some(rest) = source.strip_prefix("struct") {
        Ok(Item::Struct(synthetic_struct_decl_from_export(
            export_name,
            rest,
            next_span_base,
            custom_types,
        )?))
    } else if let Some(rest) = source.strip_prefix("enum") {
        Ok(Item::Enum(synthetic_enum_decl_from_export(
            export_name,
            rest,
            next_span_base,
        )?))
    } else {
        Err(SessionError::new(
            "include",
            format!("unsupported MLIB type signature `{source}`"),
        ))
    }
}

fn synthetic_struct_decl_from_export(
    export_name: &str,
    body: &str,
    next_span_base: &mut usize,
    custom_types: &mut HashSet<String>,
) -> Result<StructDecl, SessionError> {
    let body = parse_braced_body(body.trim(), "struct")?;
    let name_span = fresh_span(next_span_base);
    let mut fields = Vec::new();

    if !body.trim().is_empty() {
        for field in split_top_level(body, ',') {
            let Some((name, ty_text)) = field.split_once(':') else {
                return Err(SessionError::new(
                    "include",
                    format!("unsupported MLIB struct field `{field}`"),
                ));
            };
            let ty = parse_type_ref_text(ty_text.trim())?;
            collect_custom_types_from_type_ref(&ty, custom_types);
            let span = fresh_span(next_span_base);
            let field_name = name.trim().to_string();
            fields.push(StructField {
                name: field_name,
                name_span: span,
                ty,
                span,
            });
        }
    }

    let span = fresh_span(next_span_base);
    Ok(StructDecl {
        name: export_name.to_string(),
        name_span,
        fields,
        span,
    })
}

fn synthetic_enum_decl_from_export(
    export_name: &str,
    body: &str,
    next_span_base: &mut usize,
) -> Result<EnumDecl, SessionError> {
    let body = parse_braced_body(body.trim(), "enum")?;
    let name_span = fresh_span(next_span_base);
    let mut variants = Vec::new();

    if !body.trim().is_empty() {
        for variant in split_top_level(body, ',') {
            let Some((name, discriminant)) = variant.split_once('=') else {
                return Err(SessionError::new(
                    "include",
                    format!("unsupported MLIB enum variant `{variant}`"),
                ));
            };
            let discriminant = discriminant.trim().parse::<usize>().map_err(|error| {
                SessionError::new(
                    "include",
                    format!(
                        "invalid MLIB enum discriminant `{}` for `{}`: {error}",
                        discriminant.trim(),
                        name.trim()
                    ),
                )
            })?;
            let span = fresh_span(next_span_base);
            variants.push(EnumVariant {
                name: name.trim().to_string(),
                name_span: span,
                discriminant: Some(discriminant),
                span,
            });
        }
    }

    let span = fresh_span(next_span_base);
    Ok(EnumDecl {
        name: export_name.to_string(),
        name_span,
        variants,
        span,
    })
}

fn synthetic_function_from_export(
    export_name: &str,
    signature: &str,
    next_span_base: &mut usize,
    custom_types: &mut HashSet<String>,
) -> Result<FunctionDecl, SessionError> {
    let (receiver, name) = if let Some((receiver, name)) = export_name.rsplit_once('.') {
        (Some(receiver.to_string()), name.to_string())
    } else {
        (None, export_name.to_string())
    };
    let parsed = parse_signature_text(signature)?;
    for ty in &parsed.params {
        collect_custom_types_from_type_ref(ty, custom_types);
    }
    if let Some(return_type) = &parsed.return_type {
        collect_custom_types_from_type_ref(return_type, custom_types);
    }
    if let Some(receiver) = &receiver {
        custom_types.insert(receiver.clone());
    }

    let receiver_path = receiver
        .as_ref()
        .map(|receiver| synthetic_path(vec![receiver.clone()], fresh_span(next_span_base)));
    let name_span = fresh_span(next_span_base);
    let span = fresh_span(next_span_base);
    let mut params = Vec::new();
    for (index, ty) in parsed.params.into_iter().enumerate() {
        let is_self = receiver.is_some()
            && index == 0
            && matches!(
                &ty.kind,
                TypeRefKind::Path { path, .. }
                    if path.segments.last() == receiver.as_ref()
            );
        let param_name = if is_self {
            "self".to_string()
        } else {
            format!("arg{index}")
        };
        let ty = if is_self { None } else { Some(ty) };
        let param_span = fresh_span(next_span_base);
        params.push(Param {
            name: param_name,
            name_span: param_span,
            ty,
            span: param_span,
        });
    }

    Ok(FunctionDecl {
        visibility: Visibility::Public,
        receiver: receiver_path,
        name,
        name_span,
        params,
        return_type: parsed.return_type,
        body: None,
        span,
    })
}

fn synthetic_struct_decl(name: String, next_span_base: &mut usize) -> StructDecl {
    let span = fresh_span(next_span_base);
    StructDecl {
        name,
        name_span: span,
        fields: Vec::new(),
        span,
    }
}

fn parse_braced_body<'a>(source: &'a str, kind: &str) -> Result<&'a str, SessionError> {
    let source = source.trim();
    let Some(rest) = source.strip_prefix('{') else {
        return Err(SessionError::new(
            "include",
            format!("unsupported MLIB {kind} signature `{source}`"),
        ));
    };
    let Some(rest) = rest.strip_suffix('}') else {
        return Err(SessionError::new(
            "include",
            format!("unsupported MLIB {kind} signature `{source}`"),
        ));
    };
    Ok(rest.trim())
}

fn collect_custom_types_from_type_ref(ty: &TypeRef, custom_types: &mut HashSet<String>) {
    match &ty.kind {
        TypeRefKind::Path { path, arguments } => {
            if let Some(name) = path.segments.last() {
                if !is_builtin_type_name(name) {
                    custom_types.insert(name.clone());
                }
            }
            for argument in arguments {
                collect_custom_types_from_type_ref(argument, custom_types);
            }
        }
        TypeRefKind::Array { element, .. } => collect_custom_types_from_type_ref(element, custom_types),
    }
}

fn is_builtin_type_name(name: &str) -> bool {
    matches!(name, "int" | "byte" | "float" | "string" | "bool" | "Error" | "Result" | "Range")
}

struct ParsedSignature {
    params: Vec<TypeRef>,
    return_type: Option<TypeRef>,
}

fn parse_signature_text(source: &str) -> Result<ParsedSignature, SessionError> {
    let source = source.trim();
    let Some(rest) = source.strip_prefix("fn(") else {
        return Err(SessionError::new(
            "include",
            format!("unsupported MLIB signature `{source}`"),
        ));
    };
    let Some((params_text, return_text)) = rest.split_once(") -> ") else {
        return Err(SessionError::new(
            "include",
            format!("unsupported MLIB signature `{source}`"),
        ));
    };

    let params = if params_text.trim().is_empty() {
        Vec::new()
    } else {
        split_top_level(params_text, ',')
            .into_iter()
            .map(|entry| parse_type_ref_text(entry.trim()))
            .collect::<Result<Vec<_>, _>>()?
    };
    let return_type = if return_text.trim() == "()" {
        None
    } else {
        Some(parse_type_ref_text(return_text.trim())?)
    };

    Ok(ParsedSignature { params, return_type })
}

fn parse_type_ref_text(source: &str) -> Result<TypeRef, SessionError> {
    let span = Span::default();
    if source == "_" {
        return Err(SessionError::new(
            "include",
            "MLIB signatures cannot contain inferred `_` types",
        ));
    }

    if let Some(inner) = source.strip_prefix('[').and_then(|rest| rest.strip_suffix(']')) {
        let Some((element, length)) = inner.rsplit_once(';') else {
            return Err(SessionError::new(
                "include",
                format!("unsupported array type `{source}`"),
            ));
        };
        let element = parse_type_ref_text(element.trim())?;
        let length = length.trim().parse::<usize>().map_err(|error| {
            SessionError::new(
                "include",
                format!("invalid array length in `{source}`: {error}"),
            )
        })?;
        return Ok(TypeRef {
            kind: TypeRefKind::Array {
                element: Box::new(element),
                length,
            },
            span,
        });
    }

    if let Some(inner) = source
        .strip_prefix("Result<")
        .and_then(|rest| rest.strip_suffix('>'))
    {
        let parts = split_top_level(inner, ',');
        if parts.len() != 2 {
            return Err(SessionError::new(
                "include",
                format!("unsupported Result type `{source}`"),
            ));
        }
        return Ok(TypeRef {
            kind: TypeRefKind::Path {
                path: synthetic_path(vec!["Result".to_string()], span),
                arguments: vec![
                    parse_type_ref_text(parts[0].trim())?,
                    parse_type_ref_text(parts[1].trim())?,
                ],
            },
            span,
        });
    }

    if let Some(inner) = source
        .strip_prefix("Range<")
        .and_then(|rest| rest.strip_suffix('>'))
    {
        return Ok(TypeRef {
            kind: TypeRefKind::Path {
                path: synthetic_path(vec!["Range".to_string()], span),
                arguments: vec![parse_type_ref_text(inner.trim())?],
            },
            span,
        });
    }

    if source.starts_with("fn(") {
        return Err(SessionError::new(
            "include",
            format!("function-typed MLIB signatures are not supported yet: `{source}`"),
        ));
    }

    Ok(TypeRef {
        kind: TypeRefKind::Path {
            path: synthetic_path(
                source
                    .split('.')
                    .map(|segment| segment.trim().to_string())
                    .collect(),
                span,
            ),
            arguments: Vec::new(),
        },
        span,
    })
}

fn split_top_level(source: &str, delimiter: char) -> Vec<String> {
    let mut depth_angle = 0usize;
    let mut depth_bracket = 0usize;
    let mut current = String::new();
    let mut parts = Vec::new();

    for ch in source.chars() {
        match ch {
            '<' => depth_angle += 1,
            '>' => depth_angle = depth_angle.saturating_sub(1),
            '[' => depth_bracket += 1,
            ']' => depth_bracket = depth_bracket.saturating_sub(1),
            _ => {}
        }

        if ch == delimiter && depth_angle == 0 && depth_bracket == 0 {
            parts.push(current.trim().to_string());
            current.clear();
        } else {
            current.push(ch);
        }
    }

    if !current.trim().is_empty() {
        parts.push(current.trim().to_string());
    }

    parts
}

fn synthetic_path(segments: Vec<String>, span: Span) -> AstPath {
    AstPath {
        segment_spans: vec![span; segments.len()],
        segments,
        span,
    }
}

fn fresh_span(next_span_base: &mut usize) -> Span {
    let offset = *next_span_base;
    *next_span_base += 1;
    let position = Position::new(offset, 1, offset + 1);
    Span::new(position, position)
}
