/// Actions that can be performed by the CLI application.
#[derive(Debug, PartialEq)]
pub enum CliAction {
    /// Show version information
    ShowVersion,
    /// Show help information
    ShowHelp,
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
    if input_source.starts_with('-') && input_source != "-" {
        return CliAction::Error(format!(
            "Invalid input source: '{input_source}'. Did you mean to use a flag?"
        ));
    }
    let multi_select = args
        .iter()
        .any(|arg| arg == "--multi-select" || arg == "-m");

    // Parse height options
    let mut height: Option<u16> = None;
    let mut height_percentage: Option<f32> = None;
    let mut show_help_text = false;

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
        } else if arg == "--help-text" {
            show_help_text = true;
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
            };
        } else {
            return CliAction::RunAsyncTui {
                items: vec![input_source],
                multi_select,
                height,
                height_percentage,
                show_help_text,
            };
        }
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

        if *arg == "--async" || *arg == "-a" {
            continue;
        }

        if *arg == "--height" || *arg == "--height-percentage" {
            skip_next = true;
            continue;
        }

        if arg.starts_with("--height=") || arg.starts_with("--height-percentage=") {
            continue;
        }

        if *arg == "--help-text" {
            continue;
        }

        direct_items.push(arg.clone());
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
}
