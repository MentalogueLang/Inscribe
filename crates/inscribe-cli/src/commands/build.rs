use inscribe_codegen::emit_native_executable;

use crate::commands::emit::parse_common_args;
use crate::session::{compile_file_to_mir, default_executable_output, write_output};

pub fn run(args: &[String]) -> Result<(), String> {
    let parsed = parse_common_args(args)?;
    let output = parsed
        .output
        .unwrap_or_else(|| default_executable_output(&parsed.input, parsed.target));

    let mir = compile_file_to_mir(&parsed.input).map_err(|error| error.to_string())?;
    let bytes = emit_native_executable(&mir, parsed.target).map_err(|error| error.to_string())?;

    write_output(&output, &bytes).map_err(|error| error.to_string())?;
    println!("wrote executable: {}", output.display());
    Ok(())
}
