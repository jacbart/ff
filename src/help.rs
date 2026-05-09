/// Print usage information for the command line tool.
pub fn print_usage() {
    eprintln!("ff - fast fuzzy finder");
    eprintln!();
    eprintln!("Usage: ff [OPTIONS] [INPUT]");
    eprintln!("       <command> | ff [OPTIONS]");
    eprintln!();
    eprintln!("Arguments:");
    eprintln!("  [INPUT]  File, directory, URL, or items to search through");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -m, --multi-select             Enable multi-select mode");
    eprintln!("  -n, --line-number              Output line numbers (file input: 'file:line')");
    eprintln!("      --height <N>               Set TUI height in lines (non-fullscreen)");
    eprintln!("      --height-percentage <N>    Set TUI height as % of terminal (non-fullscreen)");
    eprintln!(
        "  -p, --preview <cmd>            Preview command (repeatable, {{ext1,ext2}} for filters)"
    );
    eprintln!("      --preview-auto             Auto-show preview on cursor move");
    eprintln!("  -h, --help                     Show this help message");
    eprintln!("  -V, --version                  Show version information");
    eprintln!();
    eprintln!("Examples:");
    eprintln!("  ff file.txt                    Select from file");
    eprintln!("  ff file.txt -m                 Multi-select from file");
    eprintln!("  ff ./src/                      Select from directory listing");
    eprintln!("  ff apple banana cherry         Select from inline items");
    eprintln!("  ls | ff                        Select from piped input");
    eprintln!("  ff file.txt --height 10        Non-fullscreen, 10 lines");
    eprintln!("  ls | ff -p 'cat'               Preview with cat (default rule)");
    eprintln!("  ls | ff -p 'bat {{rs,toml}}' -p 'glow {{md}}' -p 'cat'");
    eprintln!("  ls | ff -p 'bat' --preview-auto");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_usage_does_not_panic() {
        print_usage();
    }
}
