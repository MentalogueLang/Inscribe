use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use inscribe_codegen::Target;
use inscribe_hir::lower_module;
use inscribe_mir::{lower_program, optimize_program, MirProgram};
use inscribe_resolve::{load_module_graph, resolve_module_graph};
use inscribe_session::{Session, SessionError};
use inscribe_typeck::check_module;

pub fn host_session() -> Session {
    Session::default()
}

pub fn compile_file_to_mir(input: &Path) -> Result<MirProgram, SessionError> {
    let graph = load_module_graph(input)?;
    let resolved = resolve_module_graph(&graph)
        .map_err(|errors| join_errors("resolve", errors.into_iter().map(|e| e.to_string())))?;
    let typed = check_module(&graph.merged, &resolved)
        .map_err(|errors| join_errors("typeck", errors.into_iter().map(|e| e.to_string())))?;
    let hir = lower_module(&graph.merged, &resolved, &typed);
    let mut mir = lower_program(&hir);
    optimize_program(&mut mir);
    Ok(mir)
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
    use std::path::Path;

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

    fn workspace_root() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("cli crate should live under the workspace")
            .to_path_buf()
    }
}
