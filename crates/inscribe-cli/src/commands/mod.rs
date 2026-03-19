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
        "  inscribe check <input.mtl>",
        "  inscribe emit <asm|hir|mir|mlib|dwarf|debug> <input.mtl> [--target <linux-x86_64|windows-x86_64>] [-o <output>]",
        "  inscribe emit abi [--target <linux-x86_64|windows-x86_64>] [--stability <stable|experimental|internal>] [-o <output.mabi>]",
        "  inscribe build <input.mtl> [--sandbox] [--target <linux-x86_64|windows-x86_64>] [-o <output>]",
        "  inscribe run <input.mtl|input.smtl> [--sandbox]",
    ]
    .join("\n")
}
