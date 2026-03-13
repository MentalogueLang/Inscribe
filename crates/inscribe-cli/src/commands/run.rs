use std::fs;

use inscribe_codegen::emit_native_executable;

use crate::session::{
    compile_file_to_mir, host_target, parse_path_arg, run_host_executable, temp_output_path,
    write_output,
};

pub fn run(args: &[String]) -> Result<(), String> {
    if args.len() != 1 {
        return Err("usage: inscribe run <input.mtl>".to_string());
    }

    let input = parse_path_arg(&args[0]);
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
