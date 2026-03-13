use crate::session::{compile_file_to_mir, parse_path_arg};

pub fn run(args: &[String]) -> Result<(), String> {
    if args.len() != 1 {
        return Err("usage: inscribe check <input.mtl>".to_string());
    }

    let input = parse_path_arg(&args[0]);
    let _ = compile_file_to_mir(&input).map_err(|error| error.to_string())?;
    println!("check passed: {}", input.display());
    Ok(())
}
