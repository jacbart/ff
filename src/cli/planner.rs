/// Actions that can be performed by the CLI application.
#[derive(Debug, PartialEq)]
pub enum CliAction {
    /// Show version information
    ShowVersion,
    /// Show help information
    ShowHelp,
    /// Run benchmarks
    RunBenchmark {
        /// Whether multi-select mode is enabled
        multi_select: bool,
    },
    /// Run the terminal user interface
    RunTui {
        /// Items to search through
        items: Vec<String>,
        /// Whether multi-select mode is enabled
        multi_select: bool,
        /// Fixed height in lines
        height: Option<u16>,
        /// Height as percentage of terminal
        height_percentage: Option<f32>,
    },
    /// Error with message
    Error(String),
}

/// Plan the CLI action based on command line arguments.
pub fn plan_cli_action(args: &[String]) -> CliAction {
    if args
        .iter()
        .any(|arg| arg == "--version" || arg == "-V" || arg == "-v")
    {
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
        return CliAction::Error(format!(
            "Invalid input source: '{}'. Did you mean to use a flag?",
            input_source
        ));
    }
    let multi_select = args
        .iter()
        .any(|arg| arg == "--multi-select" || arg == "-m");

    // Parse height options
    let mut height: Option<u16> = None;
    let mut height_percentage: Option<f32> = None;

    for (i, arg) in args.iter().enumerate() {
        if arg == "--height" && i + 1 < args.len() {
            if let Ok(h) = args[i + 1].parse::<u16>() {
                height = Some(h);
            } else {
                return CliAction::Error(
                    "Invalid height value. Must be a positive integer.".to_string(),
                );
            }
        } else if arg.starts_with("--height=") {
            if let Some(value) = arg.strip_prefix("--height=") {
                if let Ok(h) = value.parse::<u16>() {
                    height = Some(h);
                } else {
                    return CliAction::Error(
                        "Invalid height value. Must be a positive integer.".to_string(),
                    );
                }
            }
        } else if arg == "--height-percentage" && i + 1 < args.len() {
            if let Ok(p) = args[i + 1].parse::<f32>() {
                if p > 0.0 && p <= 100.0 {
                    height_percentage = Some(p);
                } else {
                    return CliAction::Error(
                        "Height percentage must be between 0 and 100.".to_string(),
                    );
                }
            } else {
                return CliAction::Error(
                    "Invalid height percentage value. Must be a number between 0 and 100."
                        .to_string(),
                );
            }
        } else if arg.starts_with("--height-percentage=") {
            if let Some(value) = arg.strip_prefix("--height-percentage=") {
                if let Ok(p) = value.parse::<f32>() {
                    if p > 0.0 && p <= 100.0 {
                        height_percentage = Some(p);
                    } else {
                        return CliAction::Error(
                            "Height percentage must be between 0 and 100.".to_string(),
                        );
                    }
                } else {
                    return CliAction::Error(
                        "Invalid height percentage value. Must be a number between 0 and 100."
                            .to_string(),
                    );
                }
            }
        }
    }

    // Check for missing height values
    for (i, arg) in args.iter().enumerate() {
        if arg == "--height" && i + 1 >= args.len() {
            return CliAction::Error("Missing height value after --height".to_string());
        }
        if arg == "--height-percentage" && i + 1 >= args.len() {
            return CliAction::Error(
                "Missing height percentage value after --height-percentage".to_string(),
            );
        }
    }

    if input_source == "benchmark" {
        return CliAction::RunBenchmark { multi_select };
    }
    if input_source.contains('/') || input_source.contains('\\') || input_source.contains('.') {
        // File path
        // For testability, just return the file path as a single-item Vec
        return CliAction::RunTui {
            items: vec![input_source],
            multi_select,
            height,
            height_percentage,
        };
    }
    // Direct items
    let mut direct_items: Vec<String> = Vec::new();
    let mut skip_next = false;

    for arg in args[1..].iter() {
        if skip_next {
            skip_next = false;
            continue;
        }

        if *arg == "--multi-select" || *arg == "-m" {
            continue;
        }

        if *arg == "--height" || *arg == "--height-percentage" {
            skip_next = true;
            continue;
        }

        if arg.starts_with("--height=") || arg.starts_with("--height-percentage=") {
            continue;
        }

        direct_items.push(arg.clone());
    }
    if direct_items.is_empty() {
        return CliAction::Error("No items provided".to_string());
    }
    CliAction::RunTui {
        items: direct_items,
        multi_select,
        height,
        height_percentage,
    }
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
        assert_eq!(
            plan_cli_action(&args),
            CliAction::RunBenchmark {
                multi_select: false
            }
        );
        let args = to_args(&["ff", "benchmark", "-m"]);
        assert_eq!(
            plan_cli_action(&args),
            CliAction::RunBenchmark { multi_select: true }
        );
    }

    #[test]
    fn detects_file_path() {
        let args = to_args(&["ff", "file.txt"]);
        assert_eq!(
            plan_cli_action(&args),
            CliAction::RunTui {
                items: vec!["file.txt".to_string()],
                multi_select: false,
                height: None,
                height_percentage: None
            }
        );
        let args = to_args(&["ff", "/path/to/file.txt", "-m"]);
        assert_eq!(
            plan_cli_action(&args),
            CliAction::RunTui {
                items: vec!["/path/to/file.txt".to_string()],
                multi_select: true,
                height: None,
                height_percentage: None
            }
        );
    }

    #[test]
    fn detects_direct_items() {
        let args = to_args(&["ff", "apple", "banana", "cherry"]);
        assert_eq!(
            plan_cli_action(&args),
            CliAction::RunTui {
                items: vec![
                    "apple".to_string(),
                    "banana".to_string(),
                    "cherry".to_string()
                ],
                multi_select: false,
                height: None,
                height_percentage: None
            }
        );
        let args = to_args(&["ff", "apple", "banana", "-m"]);
        assert_eq!(
            plan_cli_action(&args),
            CliAction::RunTui {
                items: vec!["apple".to_string(), "banana".to_string()],
                multi_select: true,
                height: None,
                height_percentage: None
            }
        );
    }

    #[test]
    fn detects_empty_direct_items() {
        let args = to_args(&["ff", "-m"]);
        assert!(matches!(plan_cli_action(&args), CliAction::Error(_)));
    }

    #[test]
    fn detects_height_option() {
        let args = to_args(&["ff", "file.txt", "--height", "10"]);
        assert_eq!(
            plan_cli_action(&args),
            CliAction::RunTui {
                items: vec!["file.txt".to_string()],
                multi_select: false,
                height: Some(10),
                height_percentage: None
            }
        );
    }

    #[test]
    fn detects_height_option_with_equals() {
        let args = to_args(&["ff", "file.txt", "--height=15"]);
        assert_eq!(
            plan_cli_action(&args),
            CliAction::RunTui {
                items: vec!["file.txt".to_string()],
                multi_select: false,
                height: Some(15),
                height_percentage: None
            }
        );
    }

    #[test]
    fn detects_height_percentage_option() {
        let args = to_args(&["ff", "file.txt", "--height-percentage", "50"]);
        assert_eq!(
            plan_cli_action(&args),
            CliAction::RunTui {
                items: vec!["file.txt".to_string()],
                multi_select: false,
                height: None,
                height_percentage: Some(50.0)
            }
        );
    }

    #[test]
    fn detects_height_percentage_option_with_equals() {
        let args = to_args(&["ff", "file.txt", "--height-percentage=75"]);
        assert_eq!(
            plan_cli_action(&args),
            CliAction::RunTui {
                items: vec!["file.txt".to_string()],
                multi_select: false,
                height: None,
                height_percentage: Some(75.0)
            }
        );
    }

    #[test]
    fn detects_height_with_multi_select() {
        let args = to_args(&["ff", "file.txt", "--height", "10", "-m"]);
        assert_eq!(
            plan_cli_action(&args),
            CliAction::RunTui {
                items: vec!["file.txt".to_string()],
                multi_select: true,
                height: Some(10),
                height_percentage: None
            }
        );
    }

    #[test]
    fn detects_height_percentage_with_multi_select() {
        let args = to_args(&["ff", "file.txt", "--height-percentage", "50", "-m"]);
        assert_eq!(
            plan_cli_action(&args),
            CliAction::RunTui {
                items: vec!["file.txt".to_string()],
                multi_select: true,
                height: None,
                height_percentage: Some(50.0)
            }
        );
    }

    #[test]
    fn detects_height_with_direct_items() {
        let args = to_args(&["ff", "apple", "banana", "--height", "8"]);
        assert_eq!(
            plan_cli_action(&args),
            CliAction::RunTui {
                items: vec!["apple".to_string(), "banana".to_string()],
                multi_select: false,
                height: Some(8),
                height_percentage: None
            }
        );
    }

    #[test]
    fn detects_invalid_height_value() {
        let args = to_args(&["ff", "file.txt", "--height", "invalid"]);
        assert!(matches!(plan_cli_action(&args), CliAction::Error(_)));
    }

    #[test]
    fn detects_invalid_height_percentage_value() {
        let args = to_args(&["ff", "file.txt", "--height-percentage", "invalid"]);
        assert!(matches!(plan_cli_action(&args), CliAction::Error(_)));
    }

    #[test]
    fn detects_height_percentage_out_of_range() {
        let args = to_args(&["ff", "file.txt", "--height-percentage", "150"]);
        assert!(matches!(plan_cli_action(&args), CliAction::Error(_)));
    }

    #[test]
    fn detects_height_percentage_zero() {
        let args = to_args(&["ff", "file.txt", "--height-percentage", "0"]);
        assert!(matches!(plan_cli_action(&args), CliAction::Error(_)));
    }

    #[test]
    fn detects_missing_height_value() {
        let args = to_args(&["ff", "file.txt", "--height"]);
        assert!(matches!(plan_cli_action(&args), CliAction::Error(_)));
    }

    #[test]
    fn detects_missing_height_percentage_value() {
        let args = to_args(&["ff", "file.txt", "--height-percentage"]);
        assert!(matches!(plan_cli_action(&args), CliAction::Error(_)));
    }
}
