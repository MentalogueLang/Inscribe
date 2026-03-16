use std::fs;

use inscribe_codegen::emit_native_executable;
use inscribe_comptime::ComptimeValue;
use inscribe_sandbox::{run_main as run_sandbox_main, SandboxPolicy};

use crate::session::{
    compile_file_to_mir, host_target, parse_path_arg, run_host_executable, temp_output_path,
    write_output,
};

pub fn run(args: &[String]) -> Result<(), String> {
    let mut input = None;
    let mut sandbox = false;

    for arg in args {
        match arg.as_str() {
            "--sandbox" => sandbox = true,
            value if value.starts_with('-') => {
                return Err(format!("unknown flag `{value}`"))
            }
            value => {
                if input.is_some() {
                    return Err("expected a single input file".to_string());
                }
                input = Some(parse_path_arg(value));
            }
        }
    }

    let Some(input) = input else {
        return Err("usage: inscribe run <input.mtl> [--sandbox]".to_string());
    };

    if sandbox {
        let mir = compile_file_to_mir(&input).map_err(|error| error.to_string())?;
        let policy = SandboxPolicy {
            allow_stdout: true,
            allow_stdin: false,
            allow_network: false,
            deterministic_only: true,
        };
        let result = run_sandbox_main(&mir, policy).map_err(|error| error.message)?;
        if !matches!(result, ComptimeValue::Unit) {
            match result {
                ComptimeValue::String(value) => println!("{value}"),
                other => println!("{}", other.display()),
            }
        }
        return Ok(());
    }

    let target = host_target();
    let temp = temp_output_path(target.executable_extension());
    let mir = compile_file_to_mir(&input).map_err(|error| error.to_string())?;
    let bytes = emit_native_executable(&mir, target).map_err(|error| error.to_string())?;

    write_output(&temp, &bytes).map_err(|error| error.to_string())?;

    let exit_code = run_host_executable(&temp).map_err(|error| error.to_string())?;
    let _ = fs::remove_file(&temp);
    println!("program exited with code {exit_code}");
    Ok(())
}
