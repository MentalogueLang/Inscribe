use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use inscribe_ast::nodes::{Item, Module};
use inscribe_ast::span::Span;
use inscribe_codegen::Target;
use inscribe_hir::lower_module;
use inscribe_mir::{lower_program, MirProgram};
use inscribe_parser::parse_module;
use inscribe_session::{Session, SessionError};
use inscribe_typeck::analyze_module;

pub fn host_session() -> Session {
    Session::default()
}

pub fn compile_file_to_mir(input: &Path) -> Result<MirProgram, SessionError> {
    let module = load_module_closure(input)?;
    let (resolved, typed) = analyze_module(&module)
        .map_err(|errors| join_errors("typeck", errors.into_iter().map(|e| e.to_string())))?;
    let hir = lower_module(&module, &resolved, &typed);
    Ok(lower_program(&hir))
}

fn load_module_closure(input: &Path) -> Result<Module, SessionError> {
    let canonical = input.canonicalize().map_err(|error| {
        SessionError::new(
            "io",
            format!("failed to resolve `{}`: {error}", input.display()),
        )
    })?;
    let mut loaded = HashMap::new();
    let mut stack = HashSet::new();
    let root = load_module_recursive(&canonical, &mut loaded, &mut stack)?;

    let mut items = Vec::new();
    for path in root.closure {
        if let Some(module) = loaded.get(&path) {
            items.extend(
                module
                    .module
                    .items
                    .iter()
                    .filter(|item| !matches!(item, Item::Import(_)))
                    .cloned(),
            );
        }
    }

    Ok(Module {
        items,
        span: root.span,
    })
}

#[derive(Debug, Clone)]
struct LoadedModule {
    module: Module,
    closure: Vec<PathBuf>,
    span: Span,
}

fn load_module_recursive(
    path: &Path,
    loaded: &mut HashMap<PathBuf, LoadedModule>,
    stack: &mut HashSet<PathBuf>,
) -> Result<LoadedModule, SessionError> {
    if let Some(module) = loaded.get(path) {
        return Ok(module.clone());
    }

    if !stack.insert(path.to_path_buf()) {
        return Err(SessionError::new(
            "include",
            format!("import cycle detected while loading `{}`", path.display()),
        ));
    }

    let source = fs::read_to_string(path).map_err(|error| {
        SessionError::new(
            "io",
            format!("failed to read `{}`: {error}", path.display()),
        )
    })?;
    let tokens = inscribe_lexer::lex(&source)
        .map_err(|error| SessionError::new("lex", error.to_string()))?;
    let module =
        parse_module(tokens).map_err(|error| SessionError::new("parse", error.to_string()))?;

    let mut closure = Vec::new();
    for item in &module.items {
        let Item::Import(import) = item else {
            continue;
        };
        let import_path = resolve_import_path(path, &import.path.segments)?;
        let child = load_module_recursive(&import_path, loaded, stack)?;
        for entry in child.closure {
            if !closure.contains(&entry) {
                closure.push(entry);
            }
        }
    }
    if !closure.contains(&path.to_path_buf()) {
        closure.push(path.to_path_buf());
    }

    let loaded_module = LoadedModule {
        span: module.span,
        module,
        closure,
    };
    stack.remove(path);
    loaded.insert(path.to_path_buf(), loaded_module.clone());
    Ok(loaded_module)
}

fn resolve_import_path(current_file: &Path, segments: &[String]) -> Result<PathBuf, SessionError> {
    if segments.is_empty() {
        return Err(SessionError::new("include", "import path cannot be empty"));
    }

    let workspace_root = workspace_root();
    let std_segments = if segments.first().is_some_and(|segment| segment == "std") {
        &segments[1..]
    } else if matches!(
        segments.first().map(String::as_str),
        Some("core" | "runtime")
    ) {
        segments
    } else {
        &[] as &[String]
    };

    if !std_segments.is_empty() {
        let candidate = std_segments
            .iter()
            .fold(workspace_root.join("stdlib"), |path, segment| {
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
        .expect("cli crate should live under the workspace")
        .to_path_buf()
}

fn join_errors<I>(stage: &'static str, errors: I) -> SessionError
where
    I: IntoIterator<Item = String>,
{
    let message = errors.into_iter().collect::<Vec<_>>().join("\n");
    SessionError::new(stage, message)
}

pub fn host_target() -> Target {
    #[cfg(target_os = "windows")]
    {
        Target::windows_x86_64()
    }

    #[cfg(not(target_os = "windows"))]
    {
        Target::linux_x86_64()
    }
}

pub fn parse_target_arg(maybe_target: Option<&str>) -> Result<Target, SessionError> {
    match maybe_target {
        None => Ok(host_target()),
        Some("linux-x86_64") | Some("x86_64-linux") => Ok(Target::linux_x86_64()),
        Some("windows-x86_64") | Some("x86_64-windows") | Some("windows") => {
            Ok(Target::windows_x86_64())
        }
        Some(value) => Err(SessionError::new(
            "cli",
            format!("unknown target `{value}`; expected `linux-x86_64` or `windows-x86_64`"),
        )),
    }
}

pub fn default_assembly_output(input: &Path) -> PathBuf {
    input.with_extension("asm")
}

pub fn default_executable_output(input: &Path, target: Target) -> PathBuf {
    input.with_extension(target.executable_extension())
}

pub fn write_output(path: &Path, bytes: &[u8]) -> Result<(), SessionError> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|error| {
                SessionError::new(
                    "io",
                    format!("failed to create `{}`: {error}", parent.display()),
                )
            })?;
        }
    }

    fs::write(path, bytes).map_err(|error| {
        SessionError::new(
            "io",
            format!("failed to write `{}`: {error}", path.display()),
        )
    })
}

pub fn parse_path_arg(value: &str) -> PathBuf {
    PathBuf::from(value)
}

pub fn run_host_executable(path: &Path) -> Result<i32, SessionError> {
    let status = Command::new(path).status().map_err(|error| {
        SessionError::new(
            "run",
            format!("failed to run `{}`: {error}", path.display()),
        )
    })?;

    Ok(status.code().unwrap_or(1))
}

pub fn temp_output_path(extension: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should move forward")
        .as_nanos();
    std::env::temp_dir().join(format!("inscribe_{stamp}.{extension}"))
}

#[cfg(test)]
mod tests {
    use super::compile_file_to_mir;
    use super::workspace_root;

    #[test]
    fn compiles_local_import_fixture() {
        let path = workspace_root().join("tests/compile_pass/import_local.mtl");
        let program = compile_file_to_mir(&path).expect("local imports should compile");
        assert!(!program.functions.is_empty());
    }

    #[test]
    fn compiles_stdlib_import_fixture() {
        let path = workspace_root().join("tests/compile_pass/import_stdlib.mtl");
        let program = compile_file_to_mir(&path).expect("stdlib imports should compile");
        assert!(!program.functions.is_empty());
    }
}
