use std::fs;

use inscribe_codegen::emit_native_executable;
use inscribe_comptime::ComptimeValue;
use inscribe_sandbox::{run_main as run_sandbox_main, SandboxPolicy};

use crate::session::{
    compile_file_to_mir, decode_sandbox_module, host_target, parse_path_arg, run_host_executable,
    temp_output_path, write_output,
};

pub fn run(args: &[String]) -> Result<(), String> {
    let mut input = None;
    let mut sandbox = false;

    for arg in args {
        match arg.as_str() {
            "--sandbox" => sandbox = true,
            value if value.starts_with('-') => return Err(format!("unknown flag `{value}`")),
            value => {
                if input.is_some() {
                    return Err("expected a single input file".to_string());
                }
                input = Some(parse_path_arg(value));
            }
        }
    }

    let Some(input) = input else {
        return Err("usage: inscribe run <input.mtl|input.smtl> [--sandbox]".to_string());
    };

    if is_sandbox_module_path(&input) {
        let bytes = std::fs::read(&input)
            .map_err(|error| format!("failed to read `{}`: {error}", input.display()))?;
        let mir = decode_sandbox_module(&bytes).map_err(|error| error.to_string())?;
        return run_with_sandbox(&mir);
    }

    let mir = compile_file_to_mir(&input).map_err(|error| error.to_string())?;

    if sandbox {
        return run_with_sandbox(&mir);
    }

    let target = host_target();
    let temp = temp_output_path(target.executable_extension());
    let bytes = match emit_native_executable(&mir, target) {
        Ok(bytes) => bytes,
        Err(error) if should_fallback_to_sandbox(&error.message) => {
            eprintln!(
                "native backend unavailable: {}. Falling back to sandbox execution.",
                error.message
            );
            return run_with_sandbox(&mir);
        }
        Err(error) => return Err(error.to_string()),
    };

    write_output(&temp, &bytes).map_err(|error| error.to_string())?;

    let exit_code = run_host_executable(&temp).map_err(|error| error.to_string())?;
    let _ = fs::remove_file(&temp);
    println!("program exited with code {exit_code}");
    Ok(())
}

fn is_sandbox_module_path(path: &std::path::Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.eq_ignore_ascii_case("smtl"))
        .unwrap_or(false)
}

fn run_with_sandbox(mir: &inscribe_mir::MirProgram) -> Result<(), String> {
    let policy = SandboxPolicy {
        allow_stdout: true,
        allow_stdin: false,
        allow_network: false,
        deterministic_only: true,
    };
    let result = run_sandbox_main(mir, policy).map_err(|error| error.message)?;
    if !matches!(result, ComptimeValue::Unit) {
        match result {
            ComptimeValue::String(value) => println!("{value}"),
            other => println!("{}", other.display()),
        }
    }
    Ok(())
}

fn should_fallback_to_sandbox(message: &str) -> bool {
    message.contains("native codegen does not yet support")
        || message.contains("native codegen does not yet implement")
        || message.contains("native codegen currently only supports")
}
