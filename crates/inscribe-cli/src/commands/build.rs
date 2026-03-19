use inscribe_codegen::emit_native_executable;

use crate::session::{
    compile_file_to_mir, default_executable_output, default_sandbox_output, encode_sandbox_module,
    parse_path_arg, parse_target_arg, write_output,
};

pub fn run(args: &[String]) -> Result<(), String> {
    let parsed = parse_build_args(args)?;

    let mir = compile_file_to_mir(&parsed.input).map_err(|error| error.to_string())?;

    if parsed.sandbox {
        let output = parsed
            .output
            .unwrap_or_else(|| default_sandbox_output(&parsed.input));
        let bytes = encode_sandbox_module(&mir).map_err(|error| error.to_string())?;
        write_output(&output, &bytes).map_err(|error| error.to_string())?;
        println!("wrote sandbox artifact: {}", output.display());
        return Ok(());
    }

    let output = parsed
        .output
        .unwrap_or_else(|| default_executable_output(&parsed.input, parsed.target));
    let bytes = emit_native_executable(&mir, parsed.target)
        .map_err(|error| format_build_codegen_error(&error.message))?;
    write_output(&output, &bytes).map_err(|error| error.to_string())?;
    println!("wrote executable: {}", output.display());
    Ok(())
}

#[derive(Debug)]
struct BuildArgs {
    input: std::path::PathBuf,
    output: Option<std::path::PathBuf>,
    target: inscribe_codegen::Target,
    sandbox: bool,
}

fn parse_build_args(args: &[String]) -> Result<BuildArgs, String> {
    let mut input = None;
    let mut output = None;
    let mut target = None;
    let mut sandbox = false;
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--sandbox" => sandbox = true,
            "-o" | "--output" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err("missing value after `-o`".to_string());
                };
                output = Some(parse_path_arg(value));
            }
            "--target" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err("missing value after `--target`".to_string());
                };
                target = Some(parse_target_arg(Some(value)).map_err(|error| error.to_string())?);
            }
            value if value.starts_with('-') => return Err(format!("unknown flag `{value}`")),
            value => {
                if input.is_some() {
                    return Err("expected a single input file".to_string());
                }
                input = Some(parse_path_arg(value));
            }
        }
        index += 1;
    }

    let Some(input) = input else {
        return Err(
            "usage: inscribe build <input.mtl> [--sandbox] [--target <linux-x86_64|windows-x86_64>] [-o <output>]"
                .to_string(),
        );
    };

    Ok(BuildArgs {
        input,
        output,
        target: target.unwrap_or_else(crate::session::host_target),
        sandbox,
    })
}

fn format_build_codegen_error(message: &str) -> String {
    if message.contains("native codegen does not yet support")
        || message.contains("native codegen does not yet implement")
        || message.contains("native codegen currently only supports")
    {
        format!(
            "{message}\nHint: use `inscribe run --sandbox <input.mtl>` to execute immediately, or `inscribe build --sandbox <input.mtl>` to write a portable `.smtl` artifact."
        )
    } else {
        message.to_string()
    }
}
