use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use inscribe_codegen::Target;
use inscribe_hir::{lower_module, HirProgram};
use inscribe_incremental::{
    Cache, DiskCache, Fingerprint, FingerprintBuilder, QueryEngine, QueryError,
};
use inscribe_mir::{lower_program, optimize_program, MirProgram};
use inscribe_resolve::{
    load_module_graph, resolve_module_graph, LoadedModuleGraph, ResolvedProgram,
};
use inscribe_session::{Session, SessionError};
use inscribe_typeck::{check_module, TypeCheckResult};

pub fn host_session() -> Session {
    Session::default()
}

pub struct CompiledArtifacts {
    pub hir: HirProgram,
    pub mir: MirProgram,
}

const CACHE_SCHEMA_VERSION: &str = "inscribe-cache-v2";
const SMTL_MAGIC: &[u8] = b"SMTL1";

#[derive(Debug, Default)]
pub struct IncrementalSession {
    engine: QueryEngine,
    graph_cache: Cache<PathBuf, LoadedModuleGraph>,
    resolved_cache: Cache<PathBuf, ResolvedProgram>,
    typed_cache: Cache<PathBuf, TypeCheckResult>,
    hir_cache: Cache<PathBuf, HirProgram>,
    mir_cache: Cache<PathBuf, MirProgram>,
}

impl IncrementalSession {
    pub fn new() -> Self {
        Self::default()
    }
}

static INCREMENTAL: OnceLock<Mutex<IncrementalSession>> = OnceLock::new();

fn with_incremental_session<F, R>(f: F) -> R
where
    F: FnOnce(&mut IncrementalSession) -> R,
{
    let session = INCREMENTAL.get_or_init(|| Mutex::new(IncrementalSession::new()));
    let mut guard = session.lock().expect("incremental session lock poisoned");
    f(&mut guard)
}

pub fn compile_file(input: &Path) -> Result<CompiledArtifacts, SessionError> {
    with_incremental_session(|incremental| compile_file_with_incremental(incremental, input))
}

pub fn compile_file_to_hir(input: &Path) -> Result<HirProgram, SessionError> {
    compile_file(input).map(|artifacts| artifacts.hir)
}

pub fn compile_file_to_mir(input: &Path) -> Result<MirProgram, SessionError> {
    compile_file(input).map(|artifacts| artifacts.mir)
}

fn compile_file_with_incremental(
    incremental: &mut IncrementalSession,
    input: &Path,
) -> Result<CompiledArtifacts, SessionError> {
    let entry = canonicalize_path(input)?;
    let cache_root = cache_root_for_entry(&entry);
    let graph_disk = DiskCache::new(&cache_root, "graph");

    let mut source_fingerprint = None;
    let mut graph = None;

    if let Some(entry_cache) = graph_disk
        .load::<PathBuf, LoadedModuleGraph>(&entry)
        .map_err(|error| cache_error("graph", &error))?
    {
        let fingerprint = fingerprint_graph_sources(&entry_cache.value)?;
        if fingerprint == entry_cache.fingerprint {
            source_fingerprint = Some(entry_cache.fingerprint);
            incremental.graph_cache.insert(
                entry.clone(),
                entry_cache.fingerprint,
                entry_cache.value.clone(),
            );
            graph = Some(entry_cache.value);
        }
    }

    let graph = match graph {
        Some(graph) => graph,
        None => {
            let graph = load_module_graph(&entry)
                .map_err(|error| SessionError::new("load", error.to_string()))?;
            let fingerprint = fingerprint_graph_sources(&graph)?;
            graph_disk
                .store(&entry, fingerprint, &graph)
                .map_err(|error| cache_error("graph", &error))?;
            incremental
                .graph_cache
                .insert(entry.clone(), fingerprint, graph.clone());
            source_fingerprint = Some(fingerprint);
            graph
        }
    };

    let source_fingerprint = source_fingerprint.unwrap_or_else(|| Fingerprint::of(&entry));
    let resolve_disk = DiskCache::new(&cache_root, "resolve");

    let resolved = incremental
        .engine
        .execute_with_disk(
            &mut incremental.resolved_cache,
            Some(&resolve_disk),
            "resolve",
            entry.clone(),
            source_fingerprint,
            |_| {
                resolve_module_graph(&graph).map_err(|errors| {
                    QueryError::new(join_error_messages(
                        errors.into_iter().map(|error| error.to_string()),
                    ))
                })
            },
        )
        .map_err(|error| SessionError::new("resolve", error.message))?;

    let typed = incremental
        .engine
        .execute_with_disk(
            &mut incremental.typed_cache,
            Some(&DiskCache::new(&cache_root, "typeck")),
            "typeck",
            entry.clone(),
            source_fingerprint,
            |_| {
                check_module(&graph.merged, &resolved).map_err(|errors| {
                    QueryError::new(join_error_messages(
                        errors.into_iter().map(|error| error.to_string()),
                    ))
                })
            },
        )
        .map_err(|error| SessionError::new("typeck", error.message))?;

    let hir = incremental
        .engine
        .execute_with_disk(
            &mut incremental.hir_cache,
            Some(&DiskCache::new(&cache_root, "hir")),
            "lower_hir",
            entry.clone(),
            source_fingerprint,
            |_| Ok(lower_module(&graph.merged, &resolved, &typed)),
        )
        .map_err(|error| SessionError::new("hir", error.message))?;

    let mir = incremental
        .engine
        .execute_with_disk(
            &mut incremental.mir_cache,
            Some(&DiskCache::new(&cache_root, "mir")),
            "lower_mir",
            entry.clone(),
            source_fingerprint,
            |_| {
                let mut mir = lower_program(&hir);
                optimize_program(&mut mir);
                Ok(mir)
            },
        )
        .map_err(|error| SessionError::new("mir", error.message))?;

    Ok(CompiledArtifacts { hir, mir })
}

fn join_error_messages<I>(errors: I) -> String
where
    I: IntoIterator<Item = String>,
{
    errors.into_iter().collect::<Vec<_>>().join("\n")
}

fn fingerprint_graph_sources(graph: &LoadedModuleGraph) -> Result<Fingerprint, SessionError> {
    let mut paths = graph
        .modules
        .iter()
        .map(|module| module.path.clone())
        .collect::<Vec<_>>();
    paths.sort();
    let fingerprint = fingerprint_paths(&paths)?;
    Ok(fingerprint.combine(compiler_cache_fingerprint()?))
}

fn fingerprint_paths(paths: &[PathBuf]) -> Result<Fingerprint, SessionError> {
    let mut builder = FingerprintBuilder::new();
    for path in paths {
        let bytes = fs::read(path).map_err(|error| {
            SessionError::new(
                "io",
                format!("failed to read `{}`: {error}", path.display()),
            )
        })?;
        builder.update_str(&path.to_string_lossy());
        builder.update_bytes(&bytes);
    }
    Ok(builder.finish())
}

fn cache_root_for_entry(entry: &Path) -> PathBuf {
    let base = entry.parent().unwrap_or_else(|| Path::new("."));
    base.join(".inscribe").join("cache")
}

fn compiler_cache_fingerprint() -> Result<Fingerprint, SessionError> {
    let mut builder = FingerprintBuilder::new();
    builder.update_str(CACHE_SCHEMA_VERSION);
    builder.update_str(env!("CARGO_PKG_VERSION"));

    let exe = std::env::current_exe().map_err(|error| {
        SessionError::new(
            "io",
            format!("failed to resolve current executable: {error}"),
        )
    })?;
    builder.update_str(&exe.to_string_lossy());

    let metadata = fs::metadata(&exe).map_err(|error| {
        SessionError::new(
            "io",
            format!(
                "failed to read compiler metadata for `{}`: {error}",
                exe.display()
            ),
        )
    })?;
    builder.update_u64(metadata.len());

    if let Ok(modified) = metadata.modified() {
        if let Ok(duration) = modified.duration_since(UNIX_EPOCH) {
            builder.update_u64(duration.as_secs());
            builder.update_u64(u64::from(duration.subsec_nanos()));
        }
    }

    Ok(builder.finish())
}

fn cache_error(stage: &str, error: &std::io::Error) -> SessionError {
    SessionError::new("cache", format!("failed to access {stage} cache: {error}"))
}

fn canonicalize_path(path: &Path) -> Result<PathBuf, SessionError> {
    path.canonicalize().map_err(|error| {
        SessionError::new(
            "io",
            format!("failed to resolve `{}`: {error}", path.display()),
        )
    })
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

pub fn default_text_output(input: &Path, extension: &str) -> PathBuf {
    input.with_extension(extension)
}

pub fn default_executable_output(input: &Path, target: Target) -> PathBuf {
    input.with_extension(target.executable_extension())
}

pub fn default_sandbox_output(input: &Path) -> PathBuf {
    input.with_extension("smtl")
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

pub fn encode_sandbox_module(mir: &MirProgram) -> Result<Vec<u8>, SessionError> {
    let encoded = bincode::serialize(mir).map_err(|error| {
        SessionError::new(
            "io",
            format!("failed to serialize sandbox artifact: {error}"),
        )
    })?;
    let mut bytes = Vec::with_capacity(SMTL_MAGIC.len() + encoded.len());
    bytes.extend_from_slice(SMTL_MAGIC);
    bytes.extend_from_slice(&encoded);
    Ok(bytes)
}

pub fn decode_sandbox_module(bytes: &[u8]) -> Result<MirProgram, SessionError> {
    if bytes.len() < SMTL_MAGIC.len() || &bytes[..SMTL_MAGIC.len()] != SMTL_MAGIC {
        return Err(SessionError::new(
            "io",
            "invalid .smtl file: missing SMTL1 header",
        ));
    }

    bincode::deserialize::<MirProgram>(&bytes[SMTL_MAGIC.len()..]).map_err(|error| {
        SessionError::new(
            "io",
            format!("invalid .smtl file: failed to decode MIR payload: {error}"),
        )
    })
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
    use super::{compile_file_to_mir, decode_sandbox_module, encode_sandbox_module};
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

    #[test]
    fn compiles_io_stdlib_import_fixture() {
        let path = workspace_root().join("tests/compile_pass/import_io_console.mtl");
        let program = compile_file_to_mir(&path).expect("io stdlib imports should compile");
        assert!(!program.functions.is_empty());
    }

    #[test]
    fn sandbox_module_roundtrip() {
        let path = workspace_root().join("tests/compile_pass/import_local.mtl");
        let program = compile_file_to_mir(&path).expect("local imports should compile");
        let bytes = encode_sandbox_module(&program).expect("sandbox artifact should encode");
        let decoded = decode_sandbox_module(&bytes).expect("sandbox artifact should decode");
        assert_eq!(program, decoded);
    }

    fn workspace_root() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("cli crate should live under the workspace")
            .to_path_buf()
    }
}
