use std::path::PathBuf;

use inscribe_codegen::emit_native_assembly;

use crate::session::{
    compile_file_to_mir, default_assembly_output, parse_path_arg, parse_target_arg, write_output,
};

pub fn run(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Err(
            "usage: inscribe emit asm <input.ins> [--target <triple>] [-o <output.asm>]"
                .to_string(),
        );
    }

    let format = args[0].as_str();
    if format != "asm" {
        return Err(format!(
            "unsupported emit format `{format}`; currently only `asm` is available"
        ));
    }

    let parsed = parse_common_args(&args[1..])?;
    let output = parsed
        .output
        .unwrap_or_else(|| default_assembly_output(&parsed.input));

    let mir = compile_file_to_mir(&parsed.input).map_err(|error| error.to_string())?;
    let assembly = emit_native_assembly(&mir, parsed.target).map_err(|error| error.to_string())?;

    write_output(&output, assembly.as_bytes()).map_err(|error| error.to_string())?;
    println!("wrote assembly: {}", output.display());
    Ok(())
}

pub(crate) struct ParsedArgs {
    pub(crate) input: PathBuf,
    pub(crate) output: Option<PathBuf>,
    pub(crate) target: inscribe_codegen::Target,
}

pub(crate) fn parse_common_args(args: &[String]) -> Result<ParsedArgs, String> {
    let mut input = None;
    let mut output = None;
    let mut target = None;
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
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
            value if value.starts_with('-') => {
                return Err(format!("unknown flag `{value}`"));
            }
            value => {
                if input.is_some() {
                    return Err("expected exactly one input file".to_string());
                }
                input = Some(parse_path_arg(value));
            }
        }
        index += 1;
    }

    let Some(input) = input else {
        return Err("missing input file".to_string());
    };

    Ok(ParsedArgs {
        input,
        output,
        target: target.unwrap_or_else(crate::session::host_target),
    })
}
