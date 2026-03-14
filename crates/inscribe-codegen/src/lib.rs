use inscribe_abi as _;
use inscribe_sandbox as _;

use inscribe_mir::MirProgram;

pub mod llvm;
mod native;
pub mod targets;
pub mod wasm;

pub use native::{emit_assembly, emit_executable};
pub use targets::{Architecture, ExecutableFormat, OperatingSystem, Target};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodegenError {
    pub message: String,
}

impl CodegenError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for CodegenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for CodegenError {}

pub fn emit_native_assembly(program: &MirProgram, target: Target) -> Result<String, CodegenError> {
    emit_assembly(program, target)
}

pub fn emit_native_executable(
    program: &MirProgram,
    target: Target,
) -> Result<Vec<u8>, CodegenError> {
    emit_executable(program, target)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::process::Command;
    use std::time::{SystemTime, UNIX_EPOCH};

    use inscribe_hir::lower_module;
    use inscribe_lexer::lex;
    use inscribe_mir::lower_program;
    use inscribe_parser::parse_module;
    use inscribe_resolve::resolve_module;
    use inscribe_typeck::check_module;

    use crate::{emit_native_assembly, emit_native_executable, Target};

    fn compile_source(source: &str) -> inscribe_mir::MirProgram {
        let tokens = lex(source).expect("lexing should succeed");
        let module = parse_module(tokens).expect("parsing should succeed");
        let resolved = resolve_module(&module).expect("resolution should succeed");
        let typed = check_module(&module, &resolved).expect("type checking should succeed");
        let hir = lower_module(&module, &resolved, &typed);
        lower_program(&hir)
    }

    #[test]
    fn emits_linux_x64_assembly() {
        let mir = compile_source(
            r#"
fn main() -> int {
    let base = 40

    if true {
        base + 2
    } else {
        0
    }
}
"#,
        );

        let assembly =
            emit_native_assembly(&mir, Target::linux_x86_64()).expect("assembly emission");

        assert!(assembly.contains(".intel_syntax noprefix"));
        assert!(assembly.contains("_start:"));
        assert!(assembly.contains("syscall"));
        assert!(assembly.contains(".Lbb"));
    }

    #[test]
    fn emits_raw_elf_bytes() {
        let mir = compile_source(
            r#"
fn main() -> int {
    7
}
"#,
        );

        let bytes =
            emit_native_executable(&mir, Target::linux_x86_64()).expect("elf emission should work");

        assert_eq!(&bytes[..4], b"\x7FELF");
        assert_eq!(bytes[4], 2);
        assert_eq!(bytes[5], 1);
        assert_eq!(u16::from_le_bytes([bytes[18], bytes[19]]), 62);
    }

    #[test]
    fn emits_raw_pe_bytes() {
        let mir = compile_source(
            r#"
fn main() -> int {
    9
}
"#,
        );

        let bytes = emit_native_executable(&mir, Target::windows_x86_64())
            .expect("pe emission should work");

        assert_eq!(&bytes[..2], b"MZ");
        let pe_offset =
            u32::from_le_bytes([bytes[0x3c], bytes[0x3d], bytes[0x3e], bytes[0x3f]]) as usize;
        assert_eq!(&bytes[pe_offset..pe_offset + 4], b"PE\0\0");
    }

    #[test]
    fn emits_direct_function_calls() {
        let mir = compile_source(
            r#"
fn add(left: int, right: int) -> int {
    left + right
}

fn main() -> int {
    add(4, 3)
}
"#,
        );

        let assembly =
            emit_native_assembly(&mir, Target::linux_x86_64()).expect("assembly emission");

        assert!(assembly.contains("call __ml_fn_add"));
        assert!(assembly.contains("__ml_fn_main"));
        assert!(assembly.contains("__ml_fn_add"));
    }

    #[test]
    fn emits_many_argument_calls() {
        let mir = compile_source(
            r#"
fn sum8(a: int, b: int, c: int, d: int, e: int, f: int, g: int, h: int) -> int {
    a + b + c + d + e + f + g + h
}

fn main() -> int {
    sum8(1, 2, 3, 4, 5, 6, 7, 8)
}
"#,
        );

        let assembly =
            emit_native_assembly(&mir, Target::linux_x86_64()).expect("assembly emission");

        assert!(assembly.contains("call __ml_fn_sum8"));
        assert!(assembly.contains("r8"));
        assert!(assembly.contains("r9"));
        assert!(assembly.contains("qword ptr [rsp + 0]"));
        assert!(assembly.contains("qword ptr [rsp + 8]"));
    }

    #[cfg(windows)]
    #[test]
    fn generated_pe_executable_runs() {
        let mir = compile_source(
            r#"
fn main() -> int {
    let counter = 4
    counter + 3
}
"#,
        );

        let bytes = emit_native_executable(&mir, Target::windows_x86_64())
            .expect("pe emission should work");
        let path = temp_output("inscribe_codegen_smoke.exe");
        fs::write(&path, bytes).expect("should write executable");

        let status = Command::new(&path)
            .status()
            .expect("generated executable should run");

        let _ = fs::remove_file(&path);
        assert_eq!(status.code(), Some(7));
    }

    #[cfg(windows)]
    #[test]
    fn generated_pe_executable_runs_with_many_arguments() {
        let mir = compile_source(
            r#"
fn sum6(a: int, b: int, c: int, d: int, e: int, f: int) -> int {
    a + b + c + d + e + f
}

fn main() -> int {
    sum6(1, 2, 3, 4, 5, 6)
}
"#,
        );

        let bytes = emit_native_executable(&mir, Target::windows_x86_64())
            .expect("pe emission should work");
        let path = temp_output("inscribe_codegen_many_args.exe");
        fs::write(&path, bytes).expect("should write executable");

        let status = Command::new(&path)
            .status()
            .expect("generated executable should run");

        let _ = fs::remove_file(&path);
        assert_eq!(status.code(), Some(21));
    }

    #[cfg(windows)]
    fn temp_output(name: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should move forward")
            .as_nanos();
        std::env::temp_dir().join(format!("{stamp}_{name}"))
    }

    #[test]
    fn emits_pe_with_helper_call() {
        let mir = compile_source(
            r#"
fn add(left: int, right: int) -> int {
    left + right
}

fn main() -> int {
    add(2, 5)
}
"#,
        );

        let bytes = emit_native_executable(&mir, Target::windows_x86_64())
            .expect("pe emission should work");
        assert_eq!(&bytes[..2], b"MZ");
    }

    #[test]
    fn rejects_unknown_declared_runtime_functions() {
        let mir = compile_source(
            r#"
fn host_magic(value: int)

fn main() -> int {
    host_magic(7)
    0
}
"#,
        );

        let error = emit_native_executable(&mir, Target::linux_x86_64())
            .expect_err("declared runtime calls should not compile as no-op stubs");
        assert!(error
            .message
            .contains("does not yet implement declared runtime function `host_magic`"));
    }

    #[cfg(windows)]
    #[test]
    fn generated_pe_runtime_prints_int() {
        let mir = compile_source(
            r#"
fn print_int(value: int)

fn main() -> int {
    print_int(81)
    0
}
"#,
        );

        let bytes = emit_native_executable(&mir, Target::windows_x86_64())
            .expect("pe emission should work");
        let path = temp_output("inscribe_codegen_print_int.exe");
        fs::write(&path, bytes).expect("should write executable");

        let output = Command::new(&path)
            .output()
            .expect("generated executable should run");

        let _ = fs::remove_file(&path);
        assert_eq!(output.status.code(), Some(0));
        assert_eq!(String::from_utf8_lossy(&output.stdout), "81");
    }
}
