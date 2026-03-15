use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use inscribe_ast::nodes::{Block, Expr, ExprKind, Item, Module, Stmt, Visibility};
use inscribe_parser::parse_module;
use inscribe_session::SessionError;

use crate::module_tree::{ImportNode, ItemNode, ModuleNode, ModuleTree};
use crate::resolver::FunctionKey;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceModule {
    pub path: PathBuf,
    pub module: Module,
    pub imports: Vec<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedModuleGraph {
    pub entry: PathBuf,
    pub modules: Vec<SourceModule>,
    pub merged: Module,
    pub tree: ModuleTree,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
    load_recursive(&entry, options, &mut loaded, &mut stack)?;

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
    let module =
        parse_module(tokens).map_err(|error| SessionError::new("parse", error.to_string()))?;

    let mut imports = Vec::new();
    for item in &module.items {
        let Item::Import(import) = item else {
            continue;
        };

        let import_path = resolve_import_path(path, &import.path.segments, options)?;
        imports.push(import_path.clone());
        load_recursive(&import_path, options, loaded, stack)?;
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
        "__priv_{}_{}",
        sanitize_symbol(&source_path.display().to_string()),
        sanitize_symbol(name)
    )
}

fn sanitize_symbol(value: &str) -> String {
    value
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect()
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
