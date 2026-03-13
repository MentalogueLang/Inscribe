pub mod commands;
pub mod session;

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    match commands::dispatch(&args) {
        Ok(()) => {}
        Err(message) => {
            eprintln!("{message}");
            std::process::exit(1);
        }
    }
}
