/// Print usage information for the command line tool.
pub fn print_usage() {
    eprintln!("ff - fast fuzzy finder");
    eprintln!();
    eprintln!("Usage: ff [OPTIONS] [INPUT]");
    eprintln!("       <command> | ff [OPTIONS]");
    eprintln!();
    eprintln!("Arguments:");
    eprintln!("  [INPUT]  File, directory, URL, items to search through, or - for stdin");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -m, --multi-select             Enable multi-select mode");
    eprintln!("  -n, --line-number              Output line numbers (file input: 'file:line')");
    eprintln!("  -f, --filter <QUERY>           Print matches and exit, without the TUI");
    eprintln!("      --height <N>               Set TUI height in lines (non-fullscreen)");
    eprintln!("      --height-percentage <N>    Set TUI height as % of terminal (non-fullscreen)");
    eprintln!(
        "  -p, --preview <cmd>            Preview command (repeatable, {{ext1,ext2}} for filters)"
    );
    eprintln!("      --preview-auto             Auto-show preview on cursor move");
    eprintln!("      --help-text                Show the keyboard help inside the TUI");
    eprintln!("  -h, --help                     Show this help message");
    eprintln!("  -V, --version                  Show version information");
    eprintln!();
    eprintln!("Flags may appear before or after the input. Use -- to end flag parsing,");
    eprintln!("for example: ff -- --name-starting-with-dashes");
    eprintln!();
    eprintln!("Examples:");
    eprintln!("  ff file.txt                    Select from file");
    eprintln!("  ff --height 10 file.txt        Flags can come first");
    eprintln!("  ff file.txt -m                 Multi-select from file");
    eprintln!("  ff ./src/                      Select from directory listing");
    eprintln!("  ff apple banana cherry         Select from inline items");
    eprintln!("  ls | ff                        Select from piped input");
    eprintln!("  git ls-files | ff -f src/      Print matches, no TUI (exit 1 if none)");
    eprintln!("  ff -f help ./src/              Filter a directory listing");
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
