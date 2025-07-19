// This is the CLI entry point for ff
fn main() {
    if let Err(e) = ff::cli_main() {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
