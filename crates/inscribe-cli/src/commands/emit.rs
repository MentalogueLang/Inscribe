use std::path::{Path, PathBuf};

use inscribe_abi::{current_header, AbiTarget, Stability};
use inscribe_codegen::{emit_native_assembly, OperatingSystem, Target};
use inscribe_debug::{build_program_debug_info, emit_program_dwarf};
use inscribe_hir::render as render_hir;

use crate::session::{
    compile_file_to_hir, compile_file_to_mir, default_assembly_output, default_text_output,
    parse_path_arg, parse_target_arg, write_output,
};

pub fn run(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Err(usage().to_string());
    }

    let format = EmitFormat::parse(&args[0])?;
    let parsed = parse_emit_args(format, &args[1..])?;
    let output = parsed
        .output
        .unwrap_or_else(|| default_output_path(format, parsed.input.as_deref(), parsed.target));

    let bytes = match format {
        EmitFormat::Asm => {
            let input = parsed.input.as_ref().expect("asm emit requires an input");
            let mir = compile_file_to_mir(input).map_err(|error| error.to_string())?;
            emit_native_assembly(&mir, parsed.target)
                .map(|assembly| assembly.into_bytes())
                .map_err(|error| error.to_string())?
        }
        EmitFormat::Hir => {
            let input = parsed.input.as_ref().expect("hir emit requires an input");
            let hir = compile_file_to_hir(input).map_err(|error| error.to_string())?;
            let mut output = render_hir(&hir);
            if !output.ends_with('\n') {
                output.push('\n');
            }
            output.into_bytes()
        }
        EmitFormat::Mir => {
            let input = parsed.input.as_ref().expect("mir emit requires an input");
            let mir = compile_file_to_mir(input).map_err(|error| error.to_string())?;
            format!("{mir:#?}\n").into_bytes()
        }
        EmitFormat::Dwarf => {
            let input = parsed.input.as_ref().expect("dwarf emit requires an input");
            let mir = compile_file_to_mir(input).map_err(|error| error.to_string())?;
            format!("{:#?}\n", emit_program_dwarf(&mir)).into_bytes()
        }
        EmitFormat::Debug => {
            let input = parsed.input.as_ref().expect("debug emit requires an input");
            let mir = compile_file_to_mir(input).map_err(|error| error.to_string())?;
            format!("{:#?}\n", build_program_debug_info(&mir)).into_bytes()
        }
        EmitFormat::Abi => current_header(abi_target(parsed.target), parsed.stability)
            .to_bytes()
            .to_vec(),
    };

    write_output(&output, &bytes).map_err(|error| error.to_string())?;
    println!("wrote {}: {}", format.label(), output.display());
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EmitFormat {
    Asm,
    Hir,
    Mir,
    Dwarf,
    Debug,
    Abi,
}

impl EmitFormat {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "asm" => Ok(Self::Asm),
            "hir" => Ok(Self::Hir),
            "mir" => Ok(Self::Mir),
            "dwarf" => Ok(Self::Dwarf),
            "debug" => Ok(Self::Debug),
            "abi" => Ok(Self::Abi),
            _ => Err(format!(
                "unsupported emit format `{value}`; available formats: asm, hir, mir, dwarf, debug, abi"
            )),
        }
    }

    fn extension(self) -> &'static str {
        match self {
            Self::Asm => "asm",
            Self::Hir => "hir",
            Self::Mir => "mir",
            Self::Dwarf => "dwarf",
            Self::Debug => "debug",
            Self::Abi => "mabi",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Asm => "assembly",
            Self::Hir => "HIR",
            Self::Mir => "MIR",
            Self::Dwarf => "DWARF summary",
            Self::Debug => "debug report",
            Self::Abi => "ABI header",
        }
    }

    fn requires_input(self) -> bool {
        !matches!(self, Self::Abi)
    }
}

#[derive(Debug)]
pub(crate) struct ParsedArgs {
    pub(crate) input: Option<PathBuf>,
    pub(crate) output: Option<PathBuf>,
    pub(crate) target: Target,
    pub(crate) stability: Stability,
}

#[derive(Debug)]
pub(crate) struct CommonArgs {
    pub(crate) input: PathBuf,
    pub(crate) output: Option<PathBuf>,
    pub(crate) target: Target,
}

pub(crate) fn parse_common_args(args: &[String]) -> Result<CommonArgs, String> {
    let parsed = parse_emit_args(EmitFormat::Asm, args)?;
    Ok(CommonArgs {
        input: parsed
            .input
            .expect("common emit arguments should always include an input"),
        output: parsed.output,
        target: parsed.target,
    })
}

fn parse_emit_args(format: EmitFormat, args: &[String]) -> Result<ParsedArgs, String> {
    let mut input = None;
    let mut output = None;
    let mut target = None;
    let mut stability = Stability::Stable;
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
            "--stability" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err("missing value after `--stability`".to_string());
                };
                stability = parse_stability_arg(value)?;
            }
            value if value.starts_with('-') => return Err(format!("unknown flag `{value}`")),
            value => {
                if input.is_some() {
                    return Err("expected at most one input file".to_string());
                }
                input = Some(parse_path_arg(value));
            }
        }
        index += 1;
    }

    if format.requires_input() && input.is_none() {
        return Err("missing input file".to_string());
    }

    if !format.requires_input() && input.is_some() {
        return Err("`emit abi` does not accept an input file".to_string());
    }

    Ok(ParsedArgs {
        input,
        output,
        target: target.unwrap_or_else(crate::session::host_target),
        stability,
    })
}

fn parse_stability_arg(value: &str) -> Result<Stability, String> {
    match value {
        "stable" => Ok(Stability::Stable),
        "experimental" => Ok(Stability::Experimental),
        "internal" => Ok(Stability::Internal),
        _ => Err(format!(
            "unknown stability `{value}`; expected `stable`, `experimental`, or `internal`"
        )),
    }
}

fn abi_target(target: Target) -> AbiTarget {
    match target.os {
        OperatingSystem::Linux => AbiTarget::LinuxX86_64,
        OperatingSystem::Windows => AbiTarget::WindowsX86_64,
    }
}

fn default_output_path(format: EmitFormat, input: Option<&Path>, target: Target) -> PathBuf {
    match format {
        EmitFormat::Asm => default_assembly_output(
            input.expect("assembly output should only be requested for file-backed emits"),
        ),
        EmitFormat::Abi => PathBuf::from(format!("inscribe.{}", format.extension())),
        _ => {
            let input = input.expect("text output should only be requested for file-backed emits");
            let _ = target;
            default_text_output(input, format.extension())
        }
    }
}

fn usage() -> &'static str {
    "usage: inscribe emit <asm|hir|mir|dwarf|debug> <input.mtl> [--target <linux-x86_64|windows-x86_64>] [-o <output>]\n       inscribe emit abi [--target <linux-x86_64|windows-x86_64>] [--stability <stable|experimental|internal>] [-o <output.mabi>]"
}

#[cfg(test)]
mod tests {
    use super::{parse_emit_args, EmitFormat};
    use inscribe_abi::Stability;
    use inscribe_codegen::Target;

    #[test]
    fn parses_hir_emit_with_input() {
        let args = vec!["program.mtl".to_string()];
        let parsed = parse_emit_args(EmitFormat::Hir, &args).expect("hir args should parse");

        assert_eq!(parsed.input.as_deref(), Some(std::path::Path::new("program.mtl")));
        assert_eq!(parsed.stability, Stability::Stable);
    }

    #[test]
    fn parses_abi_emit_without_input() {
        let args = vec![
            "--target".to_string(),
            "windows-x86_64".to_string(),
            "--stability".to_string(),
            "experimental".to_string(),
        ];
        let parsed = parse_emit_args(EmitFormat::Abi, &args).expect("abi args should parse");

        assert!(parsed.input.is_none());
        assert_eq!(parsed.target, Target::windows_x86_64());
        assert_eq!(parsed.stability, Stability::Experimental);
    }

    #[test]
    fn rejects_input_for_abi_emit() {
        let args = vec!["program.mtl".to_string()];
        let error = parse_emit_args(EmitFormat::Abi, &args).expect_err("abi emit should reject input");

        assert!(error.contains("does not accept an input file"));
    }
}
