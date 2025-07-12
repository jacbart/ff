

#[derive(Debug, PartialEq)]
pub enum CliAction {
    ShowVersion,
    ShowHelp,
    RunBenchmark { multi_select: bool },
    RunTui { items: Vec<String>, multi_select: bool },
    Error(String),
}

pub fn plan_cli_action(args: &[String]) -> CliAction {
    if args.iter().any(|arg| arg == "--version" || arg == "-V" || arg == "-v") {
        return CliAction::ShowVersion;
    }
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        return CliAction::ShowHelp;
    }
    if args.len() < 2 {
        return CliAction::Error("Missing required argument: input-source or items".to_string());
    }
    let input_source = args[1].clone();
    if input_source.starts_with('-') {
        return CliAction::Error(format!("Invalid input source: '{}'. Did you mean to use a flag?", input_source));
    }
    let multi_select = args.iter().any(|arg| arg == "--multi-select" || arg == "-m");
    if input_source == "benchmark" {
        return CliAction::RunBenchmark { multi_select };
    }
    if input_source.contains('/') || input_source.contains('\\') || input_source.contains('.') {
        // File path
        // For testability, just return the file path as a single-item Vec
        return CliAction::RunTui { items: vec![input_source], multi_select };
    }
    // Direct items
    let direct_items: Vec<String> = args[1..]
        .iter()
        .filter(|arg| *arg != "--multi-select" && *arg != "-m")
        .cloned()
        .collect();
    if direct_items.is_empty() {
        return CliAction::Error("No items provided".to_string());
    }
    CliAction::RunTui { items: direct_items, multi_select }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn to_args(args: &[&str]) -> Vec<String> {
        args.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn detects_version_flag() {
        let args = to_args(&["ff", "--version"]);
        assert_eq!(plan_cli_action(&args), CliAction::ShowVersion);
        let args = to_args(&["ff", "-V"]);
        assert_eq!(plan_cli_action(&args), CliAction::ShowVersion);
    }

    #[test]
    fn detects_help_flag() {
        let args = to_args(&["ff", "--help"]);
        assert_eq!(plan_cli_action(&args), CliAction::ShowHelp);
        let args = to_args(&["ff", "-h"]);
        assert_eq!(plan_cli_action(&args), CliAction::ShowHelp);
    }

    #[test]
    fn detects_missing_argument() {
        let args = to_args(&["ff"]);
        assert!(matches!(plan_cli_action(&args), CliAction::Error(_)));
    }

    #[test]
    fn detects_invalid_input_source() {
        let args = to_args(&["ff", "-bad"]);
        assert!(matches!(plan_cli_action(&args), CliAction::Error(_)));
    }

    #[test]
    fn detects_benchmark_mode() {
        let args = to_args(&["ff", "benchmark"]);
        assert_eq!(plan_cli_action(&args), CliAction::RunBenchmark { multi_select: false });
        let args = to_args(&["ff", "benchmark", "-m"]);
        assert_eq!(plan_cli_action(&args), CliAction::RunBenchmark { multi_select: true });
    }

    #[test]
    fn detects_file_path() {
        let args = to_args(&["ff", "file.txt"]);
        assert_eq!(plan_cli_action(&args), CliAction::RunTui { items: vec!["file.txt".to_string()], multi_select: false });
        let args = to_args(&["ff", "/path/to/file.txt", "-m"]);
        assert_eq!(plan_cli_action(&args), CliAction::RunTui { items: vec!["/path/to/file.txt".to_string()], multi_select: true });
    }

    #[test]
    fn detects_direct_items() {
        let args = to_args(&["ff", "apple", "banana", "cherry"]);
        assert_eq!(plan_cli_action(&args), CliAction::RunTui { items: vec!["apple".to_string(), "banana".to_string(), "cherry".to_string()], multi_select: false });
        let args = to_args(&["ff", "apple", "banana", "-m"]);
        assert_eq!(plan_cli_action(&args), CliAction::RunTui { items: vec!["apple".to_string(), "banana".to_string()], multi_select: true });
    }

    #[test]
    fn detects_empty_direct_items() {
        let args = to_args(&["ff", "-m"]);
        assert!(matches!(plan_cli_action(&args), CliAction::Error(_)));
    }
} 