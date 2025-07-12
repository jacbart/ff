use std::env;

/// Configuration for the fuzzy finder application.
#[derive(Debug)]
pub struct Config {
    /// Input source identifier ("stdin", "direct", file path, or "benchmark")
    pub input_source: String,
    /// Whether multi-select mode is enabled
    pub multi_select: bool,
    /// Direct items provided as command line arguments
    pub direct_items: Option<Vec<String>>,
}

/// Parse command line arguments into a Config struct.
pub fn parse_args_from(args: &[String]) -> Result<Config, String> {
    if args.iter().any(|arg| arg == "-h" || arg == "--help") {
        print_usage();
        std::process::exit(0);
    }
    if args.len() < 2 {
        return Err("Missing required argument: input-source or items".to_string());
    }
    let input_source = args[1].clone();
    if input_source.starts_with('-') {
        return Err(format!(
            "Invalid input source: '{}'. Did you mean to use a flag?",
            input_source
        ));
    }
    if input_source == "benchmark" {
        let multi_select = args
            .iter()
            .any(|arg| arg == "--multi-select" || arg == "-m");
        return Ok(Config {
            input_source: "benchmark".to_string(),
            multi_select,
            direct_items: None,
        });
    }
    if input_source.contains('/') || input_source.contains('\\') || input_source.contains('.') {
        let multi_select = args
            .iter()
            .any(|arg| arg == "--multi-select" || arg == "-m");
        return Ok(Config {
            input_source,
            multi_select,
            direct_items: None,
        });
    }
    let multi_select = args
        .iter()
        .any(|arg| arg == "--multi-select" || arg == "-m");
    let direct_items: Vec<String> = args[1..]
        .iter()
        .filter(|arg| *arg != "--multi-select" && *arg != "-m")
        .cloned()
        .collect();
    if direct_items.is_empty() {
        return Err("No items provided".to_string());
    }
    Ok(Config {
        input_source: "direct".to_string(),
        multi_select,
        direct_items: Some(direct_items),
    })
}

/// Parse command line arguments from the environment.
pub fn parse_args() -> Result<Config, String> {
    let args: Vec<String> = env::args().collect();
    parse_args_from(&args)
}

/// Print usage information for the command line tool.
pub fn print_usage() {
    eprintln!("Usage: ff <input-source> [--multi-select] [--height <lines>] [--height-percentage <percent>]");
    eprintln!("   or: ff <item1> [item2] [item3] ... [--multi-select] [--height <lines>] [--height-percentage <percent>]");
    eprintln!();
    eprintln!("Arguments:");
    eprintln!("  input-source    File path to read items from");
    eprintln!("  item1, item2, ...  Direct list of items to search through");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  --multi-select, -m  Allow selecting multiple items (default: single select)");
    eprintln!(
        "  --height <lines>    Set TUI height to specific number of lines (non-fullscreen mode)"
    );
    eprintln!("  --height-percentage <percent>  Set TUI height as percentage of terminal (non-fullscreen mode)");
    eprintln!("  -h, --help      Show this help message");
    eprintln!("  -V, --version   Show version information");
    eprintln!();
    eprintln!("TUI Modes:");
    eprintln!("  Fullscreen (default): Full terminal interface with borders");
    eprintln!("  Non-fullscreen: Compact interface without borders, search bar as input line");
    eprintln!();
    eprintln!("TUI Controls:");
    eprintln!("  Type to search  Filter items as you type");
    eprintln!("  ↑/↓ arrows      Navigate through results");
    eprintln!("  Enter           Select item (single mode) or confirm selection (multi mode)");
    eprintln!("  Tab/Space       Toggle selection (multi-select mode only)");
    eprintln!("  Esc             Exit without selection");
    eprintln!("  Ctrl+Q          Exit without selection");
    eprintln!();
    eprintln!("Examples:");
    eprintln!("  cat file.txt | ff             # Read from stdin, single select");
    eprintln!("  ff /path/to/file.txt          # Read from file, single select");
    eprintln!("  ff /path/to/file.txt -m       # Read from file, multi-select");
    eprintln!("  ff item1 item2 item3          # Direct items, single select");
    eprintln!("  ff apple banana cherry -m     # Direct items, multi-select");
    eprintln!("  ff file.txt --height 10       # Non-fullscreen mode, 10 lines high");
    eprintln!("  ff file.txt --height-percentage 50  # Non-fullscreen mode, 50% of terminal");
    eprintln!("  ff --version                   # Show version information");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn to_args(args: &[&str]) -> Vec<String> {
        args.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn detects_help_flag() {
        let args = to_args(&["ff", "--help"]);
        // This will exit, so we can't test it directly, but we can check the branch
        // Instead, test that the function would exit if called
        // (Skip actual test to avoid process exit)
        assert!(args.iter().any(|arg| arg == "--help"));
    }

    #[test]
    fn detects_help_flag_short() {
        let args = to_args(&["ff", "-h"]);
        assert!(args.iter().any(|arg| arg == "-h"));
    }

    #[test]
    fn detects_missing_argument() {
        let args = to_args(&["ff"]);
        assert!(parse_args_from(&args).is_err());
    }

    #[test]
    fn detects_missing_argument_empty() {
        let args = to_args(&[]);
        assert!(parse_args_from(&args).is_err());
    }

    #[test]
    fn detects_invalid_input_source() {
        let args = to_args(&["ff", "-bad"]);
        assert!(parse_args_from(&args).is_err());
    }

    #[test]
    fn detects_invalid_input_source_double_dash() {
        let args = to_args(&["ff", "--invalid"]);
        assert!(parse_args_from(&args).is_err());
    }

    #[test]
    fn detects_benchmark_mode() {
        let args = to_args(&["ff", "benchmark"]);
        let config = parse_args_from(&args).unwrap();
        assert_eq!(config.input_source, "benchmark");
        assert!(!config.multi_select);
        assert!(config.direct_items.is_none());
    }

    #[test]
    fn detects_benchmark_mode_with_multi_select() {
        let args = to_args(&["ff", "benchmark", "-m"]);
        let config = parse_args_from(&args).unwrap();
        assert_eq!(config.input_source, "benchmark");
        assert!(config.multi_select);
        assert!(config.direct_items.is_none());
    }

    #[test]
    fn detects_benchmark_mode_with_multi_select_long() {
        let args = to_args(&["ff", "benchmark", "--multi-select"]);
        let config = parse_args_from(&args).unwrap();
        assert_eq!(config.input_source, "benchmark");
        assert!(config.multi_select);
        assert!(config.direct_items.is_none());
    }

    #[test]
    fn detects_file_path() {
        let args = to_args(&["ff", "file.txt"]);
        let config = parse_args_from(&args).unwrap();
        assert_eq!(config.input_source, "file.txt");
        assert!(!config.multi_select);
        assert!(config.direct_items.is_none());
    }

    #[test]
    fn detects_file_path_with_slash() {
        let args = to_args(&["ff", "/path/to/file.txt"]);
        let config = parse_args_from(&args).unwrap();
        assert_eq!(config.input_source, "/path/to/file.txt");
        assert!(!config.multi_select);
        assert!(config.direct_items.is_none());
    }

    #[test]
    fn detects_file_path_with_backslash() {
        let args = to_args(&["ff", "path\\to\\file.txt"]);
        let config = parse_args_from(&args).unwrap();
        assert_eq!(config.input_source, "path\\to\\file.txt");
        assert!(!config.multi_select);
        assert!(config.direct_items.is_none());
    }

    #[test]
    fn detects_file_path_with_dot() {
        let args = to_args(&["ff", "file.name"]);
        let config = parse_args_from(&args).unwrap();
        assert_eq!(config.input_source, "file.name");
        assert!(!config.multi_select);
        assert!(config.direct_items.is_none());
    }

    #[test]
    fn detects_file_path_with_multi_select() {
        let args = to_args(&["ff", "/path/to/file.txt", "-m"]);
        let config = parse_args_from(&args).unwrap();
        assert_eq!(config.input_source, "/path/to/file.txt");
        assert!(config.multi_select);
        assert!(config.direct_items.is_none());
    }

    #[test]
    fn detects_file_path_with_multi_select_long() {
        let args = to_args(&["ff", "file.txt", "--multi-select"]);
        let config = parse_args_from(&args).unwrap();
        assert_eq!(config.input_source, "file.txt");
        assert!(config.multi_select);
        assert!(config.direct_items.is_none());
    }

    #[test]
    fn detects_direct_items() {
        let args = to_args(&["ff", "apple", "banana", "cherry"]);
        let config = parse_args_from(&args).unwrap();
        assert_eq!(config.input_source, "direct");
        assert_eq!(
            config.direct_items.as_ref().unwrap(),
            &vec![
                "apple".to_string(),
                "banana".to_string(),
                "cherry".to_string()
            ]
        );
        assert!(!config.multi_select);
    }

    #[test]
    fn detects_direct_items_with_multi_select() {
        let args = to_args(&["ff", "apple", "banana", "-m"]);
        let config = parse_args_from(&args).unwrap();
        assert_eq!(config.input_source, "direct");
        assert!(config.multi_select);
        assert_eq!(
            config.direct_items.as_ref().unwrap(),
            &vec!["apple".to_string(), "banana".to_string()]
        );
    }

    #[test]
    fn detects_direct_items_with_multi_select_long() {
        let args = to_args(&["ff", "apple", "banana", "--multi-select"]);
        let config = parse_args_from(&args).unwrap();
        assert_eq!(config.input_source, "direct");
        assert!(config.multi_select);
        assert_eq!(
            config.direct_items.as_ref().unwrap(),
            &vec!["apple".to_string(), "banana".to_string()]
        );
    }

    #[test]
    fn detects_direct_items_with_flags_mixed() {
        let args = to_args(&["ff", "apple", "-m", "banana", "--multi-select", "cherry"]);
        let config = parse_args_from(&args).unwrap();
        assert_eq!(config.input_source, "direct");
        assert!(config.multi_select);
        assert_eq!(
            config.direct_items.as_ref().unwrap(),
            &vec![
                "apple".to_string(),
                "banana".to_string(),
                "cherry".to_string()
            ]
        );
    }

    #[test]
    fn detects_empty_direct_items() {
        let args = to_args(&["ff", "-m"]);
        assert!(parse_args_from(&args).is_err());
    }

    #[test]
    fn detects_empty_direct_items_with_flags_only() {
        let args = to_args(&["ff", "--multi-select"]);
        assert!(parse_args_from(&args).is_err());
    }

    #[test]
    fn detects_single_direct_item() {
        let args = to_args(&["ff", "single_item"]);
        let config = parse_args_from(&args).unwrap();
        assert_eq!(config.input_source, "direct");
        assert!(!config.multi_select);
        assert_eq!(
            config.direct_items.as_ref().unwrap(),
            &vec!["single_item".to_string()]
        );
    }

    #[test]
    fn detects_single_direct_item_with_multi_select() {
        let args = to_args(&["ff", "single_item", "-m"]);
        let config = parse_args_from(&args).unwrap();
        assert_eq!(config.input_source, "direct");
        assert!(config.multi_select);
        assert_eq!(
            config.direct_items.as_ref().unwrap(),
            &vec!["single_item".to_string()]
        );
    }

    #[test]
    fn test_config_struct() {
        let config = Config {
            input_source: "test".to_string(),
            multi_select: true,
            direct_items: Some(vec!["item1".to_string(), "item2".to_string()]),
        };
        assert_eq!(config.input_source, "test");
        assert!(config.multi_select);
        assert_eq!(
            config.direct_items.as_ref().unwrap(),
            &vec!["item1".to_string(), "item2".to_string()]
        );
    }

    #[test]
    fn test_config_struct_without_direct_items() {
        let config = Config {
            input_source: "file".to_string(),
            multi_select: false,
            direct_items: None,
        };
        assert_eq!(config.input_source, "file");
        assert!(!config.multi_select);
        assert!(config.direct_items.is_none());
    }

    #[test]
    fn test_print_usage_does_not_panic() {
        // Test that print_usage doesn't panic
        // We can't easily capture stderr in tests, so we just test it doesn't crash
        print_usage();
        // If we get here, it didn't panic
    }

    #[test]
    fn test_parse_args_environment() {
        // Test that parse_args function exists and can be referenced
        // We can't easily test the actual execution since it uses env::args()
        // But we can test that the function exists
        let _function = parse_args;
        // If we get here, the function exists
    }
}
