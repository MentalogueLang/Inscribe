use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use inscribe_codegen::Target;
use inscribe_hir::lower_module;
use inscribe_mir::{lower_program, MirProgram};
use inscribe_parser::parse_module;
use inscribe_resolve::resolve_module;
use inscribe_session::{Session, SessionError};
use inscribe_typeck::check_module;

pub fn host_session() -> Session {
    Session::default()
}

pub fn compile_file_to_mir(input: &Path) -> Result<MirProgram, SessionError> {
    let source = fs::read_to_string(input).map_err(|error| {
        SessionError::new(
            "io",
            format!("failed to read `{}`: {error}", input.display()),
        )
    })?;

    let tokens = inscribe_lexer::lex(&source)
        .map_err(|error| SessionError::new("lex", error.to_string()))?;
    let module =
        parse_module(tokens).map_err(|error| SessionError::new("parse", error.to_string()))?;
    let resolved = resolve_module(&module)
        .map_err(|errors| join_errors("resolve", errors.into_iter().map(|e| e.to_string())))?;
    let typed = check_module(&module, &resolved)
        .map_err(|errors| join_errors("typeck", errors.into_iter().map(|e| e.to_string())))?;
    let hir = lower_module(&module, &resolved, &typed);
    Ok(lower_program(&hir))
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
