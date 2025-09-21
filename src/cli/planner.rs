/// Actions that can be performed by the CLI application.
#[derive(Debug, PartialEq)]
pub enum CliAction {
    /// Show version information
    ShowVersion,
    /// Show help information
    ShowHelp,
    /// Generate shell integration script
    GenerateShellIntegration {
        /// Shell type (zsh, bash, fish)
        shell_type: String,
    },
    /// Run the async terminal user interface
    RunAsyncTui {
        /// Items to search through
        items: Vec<String>,
        /// Whether multi-select mode is enabled
        multi_select: bool,
        /// Fixed height in lines
        height: Option<u16>,
        /// Height as percentage of terminal
        height_percentage: Option<f32>,
        /// Whether to show help text
        show_help_text: bool,
        /// Initial query to set when TUI starts
        initial_query: Option<String>,
        /// Custom prompt to display
        prompt: Option<String>,
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

    // Check for shell integration flags
    if let Some(shell_type) = args.iter().find_map(|arg| match arg.as_str() {
        "--zsh" => Some("zsh"),
        "--bash" => Some("bash"),
        "--fish" => Some("fish"),
        _ => None,
    }) {
        return CliAction::GenerateShellIntegration {
            shell_type: shell_type.to_string(),
        };
    }

    // Parse flags first
    let multi_select = args
        .iter()
        .any(|arg| arg == "--multi-select" || arg == "-m");

    // Parse height options
    let mut height: Option<u16> = None;
    let mut height_percentage: Option<f32> = None;
    let mut show_help_text = false;
    let mut initial_query: Option<String> = None;
    let mut prompt: Option<String> = None;

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
        } else if arg == "--query" && i + 1 < args.len() {
            initial_query = Some(args[i + 1].clone());
        } else if arg.starts_with("--query=") {
            if let Some(value) = arg.strip_prefix("--query=") {
                initial_query = Some(value.to_string());
            }
        } else if arg == "--help-text" {
            show_help_text = true;
        } else if arg == "--prompt" && i + 1 < args.len() {
            prompt = Some(args[i + 1].clone());
        } else if arg.starts_with("--prompt=") {
            if let Some(value) = arg.strip_prefix("--prompt=") {
                prompt = Some(value.to_string());
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
        if arg == "--query" && i + 1 >= args.len() {
            return CliAction::Error("Missing query value after --query".to_string());
        }
        if arg == "--prompt" && i + 1 >= args.len() {
            return CliAction::Error("Missing prompt value after --prompt".to_string());
        }
    }

    // Check if we have arguments
    if args.len() < 2 {
        // No arguments provided - show error
        return CliAction::Error(
            "No input provided. Use --help for usage information.".to_string(),
        );
    }

    // Find the first non-flag argument as the input source
    let mut input_source = None;
    let mut i = 1;
    while i < args.len() {
        let arg = &args[i];
        if arg.starts_with('-') && *arg != "-" {
            // Skip flags and their values
            if *arg == "--height"
                || *arg == "--height-percentage"
                || *arg == "--query"
                || *arg == "--prompt"
            {
                i += 2; // Skip flag and its value
            } else if arg.starts_with("--height=")
                || arg.starts_with("--height-percentage=")
                || arg.starts_with("--query=")
                || arg.starts_with("--prompt=")
            {
                i += 1; // Skip flag with embedded value
            } else if *arg == "--multi-select"
                || *arg == "-m"
                || *arg == "--help-text"
                || *arg == "--zsh"
                || *arg == "--bash"
                || *arg == "--fish"
            {
                i += 1; // Skip flag without value
            } else {
                // Unknown flag
                return CliAction::Error(format!(
                    "Unknown flag: '{arg}'. Use --help for usage information."
                ));
            }
        } else {
            // Found a non-flag argument, this is our input source
            input_source = Some(arg.clone());
            break;
        }
    }

    let input_source = match input_source {
        Some(src) => src,
        None => {
            // No input source found, treat as stdin
            return CliAction::RunAsyncTui {
                items: vec!["stdin://".to_string()],
                multi_select,
                height,
                height_percentage,
                show_help_text,
                initial_query,
                prompt,
            };
        }
    };

    // Check for special input sources
    if input_source.starts_with("unix://")
        || input_source.starts_with("http://")
        || input_source.starts_with("https://")
    {
        return CliAction::RunAsyncTui {
            items: vec![input_source],
            multi_select,
            height,
            height_percentage,
            show_help_text,
            initial_query,
            prompt,
        };
    }

    // Check for file or directory paths
    if std::path::Path::new(&input_source).exists() {
        let path = std::path::Path::new(&input_source);
        if path.is_dir() {
            return CliAction::RunAsyncTui {
                items: vec![format!("dir:{}", input_source)],
                multi_select,
                height,
                height_percentage,
                show_help_text,
                initial_query,
                prompt,
            };
        } else {
            return CliAction::RunAsyncTui {
                items: vec![input_source],
                multi_select,
                height,
                height_percentage,
                show_help_text,
                initial_query,
                prompt,
            };
        }
    }

    // Direct items - collect all non-flag arguments after the input source
    let mut direct_items: Vec<String> = Vec::new();
    let mut i = 1;
    while i < args.len() {
        let arg = &args[i];
        if arg.starts_with('-') && *arg != "-" {
            // Skip flags and their values
            if *arg == "--height"
                || *arg == "--height-percentage"
                || *arg == "--query"
                || *arg == "--prompt"
            {
                i += 2; // Skip flag and its value
            } else if arg.starts_with("--height=")
                || arg.starts_with("--height-percentage=")
                || arg.starts_with("--query=")
                || arg.starts_with("--prompt=")
            {
                i += 1; // Skip flag with embedded value
            } else if *arg == "--multi-select" || *arg == "-m" || *arg == "--help-text" {
                i += 1; // Skip flag without value
            } else {
                i += 1; // Skip unknown flag
            }
        } else {
            // Found a non-flag argument
            direct_items.push(arg.clone());
            i += 1;
        }
    }

    if direct_items.is_empty() {
        return CliAction::Error("No items provided".to_string());
    }

    CliAction::RunAsyncTui {
        items: direct_items,
        multi_select,
        height,
        height_percentage,
        show_help_text,
        initial_query,
        prompt,
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

    #[test]
    fn detects_missing_query_value() {
        let args = to_args(&["ff", "file.txt", "--query"]);
        assert!(matches!(plan_cli_action(&args), CliAction::Error(_)));
    }
}
