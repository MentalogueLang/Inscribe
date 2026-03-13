pub mod build;
pub mod check;
pub mod emit;
pub mod run;

pub fn dispatch(args: &[String]) -> Result<(), String> {
    let Some((command, rest)) = args.split_first() else {
        return Err(usage());
    };

    match command.as_str() {
        "check" => check::run(rest),
        "emit" => emit::run(rest),
        "build" => build::run(rest),
        "run" => run::run(rest),
        "help" | "--help" | "-h" => Err(usage()),
        other => Err(format!("unknown command `{other}`\n\n{}", usage())),
    }
}

pub fn usage() -> String {
    [
        "usage:",
        "  inscribe check <input.ins>",
        "  inscribe emit asm <input.ins> [--target <linux-x86_64|windows-x86_64>] [-o <output.asm>]",
        "  inscribe build <input.ins> [--target <linux-x86_64|windows-x86_64>] [-o <output>]",
        "  inscribe run <input.ins>",
    ]
    .join("\n")
}
