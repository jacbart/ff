//! Argument parsing and input resolution.
//!
//! Parsing is split in two steps so both halves stay testable:
//!
//! 1. [`parse_args`] turns `argv` into flags plus positional arguments. It never
//!    touches the filesystem or the environment.
//! 2. [`resolve_input`] decides where items come from, given the positionals and
//!    whether stdin is piped.
//!
//! [`plan_cli_action`] composes the two into the action the CLI executes.
//!
//! Flags may appear before, after, or between positional arguments. `--` ends
//! flag parsing; everything after it is treated as a positional argument.

use crate::tui::preview::PreviewRule;

/// Where the items to search through come from.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum InputSpec {
    /// Items were piped into stdin.
    Stdin,
    /// A single source: file, directory (or `dir:`), `unix://`, or `http(s)://`.
    Source(String),
    /// Items given directly on the command line.
    Items(Vec<String>),
}

/// Options for the interactive TUI.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct RunOptions {
    /// Whether multi-select mode is enabled
    pub multi_select: bool,
    /// Whether to output line numbers instead of content
    pub line_number: bool,
    /// Fixed height in lines
    pub height: Option<u16>,
    /// Height as percentage of terminal
    pub height_percentage: Option<f32>,
    /// Whether to show help text
    pub show_help_text: bool,
    /// Preview rules (scanned in order; empty exts = default)
    pub preview_rules: Vec<PreviewRule>,
    /// Auto-show preview on cursor move
    pub preview_auto: bool,
}

/// Actions that can be performed by the CLI application.
#[derive(Debug, PartialEq)]
pub enum CliAction {
    /// Show version information
    ShowVersion,
    /// Show help information
    ShowHelp,
    /// Run the interactive terminal user interface
    Run {
        /// Input source
        input: InputSpec,
        /// TUI options
        options: RunOptions,
    },
    /// Print matching items and exit without a TUI
    Filter {
        /// Input source
        input: InputSpec,
        /// Query to match against
        query: String,
        /// Whether to output line numbers instead of content
        line_number: bool,
    },
    /// Error with message
    Error(String),
}

/// Flags and positionals parsed from `argv`, before input resolution.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ParsedArgs {
    /// Parsed options
    pub options: RunOptions,
    /// Positional arguments, in order
    pub positionals: Vec<String>,
    /// Query from `--filter`, if given
    pub filter: Option<String>,
}

/// Parse `argv` (including the program name) into flags and positionals.
///
/// Pure: no filesystem or environment access. Returns `Err` for unknown flags,
/// missing flag values, and malformed numbers.
pub fn parse_args(args: &[String]) -> Result<ParsedArgs, String> {
    let mut parsed = ParsedArgs::default();
    let mut idx = 1; // skip the program name
    let mut only_positional = false;
    let mut has_default_preview = false;

    while let Some(arg) = args.get(idx) {
        if only_positional {
            parsed.positionals.push(arg.clone());
            idx += 1;
            continue;
        }

        if arg == "--" {
            only_positional = true;
            idx += 1;
            continue;
        }

        if let Some(long) = arg.strip_prefix("--") {
            let (name, inline) = match long.split_once('=') {
                Some((name, value)) => (name, Some(value.to_string())),
                None => (long, None),
            };

            match name {
                "version" | "help" | "multi-select" | "line-number" | "help-text"
                | "preview-auto" => {
                    // Boolean flags take no value. --help and --version are acted
                    // on by plan_cli_action before parsing even starts.
                    if let Some(value) = inline {
                        return Err(format!("--{name} does not take a value (got '{value}')"));
                    }
                    match name {
                        "multi-select" => parsed.options.multi_select = true,
                        "line-number" => parsed.options.line_number = true,
                        "help-text" => parsed.options.show_help_text = true,
                        "preview-auto" => parsed.options.preview_auto = true,
                        _ => {}
                    }
                }
                "height" => {
                    let value = take_value(
                        args,
                        &mut idx,
                        inline,
                        "Missing height value after --height",
                    )?;
                    parsed.options.height = Some(value.parse::<u16>().map_err(|_| {
                        "Invalid height value. Must be a positive integer.".to_string()
                    })?);
                }
                "height-percentage" => {
                    let value = take_value(
                        args,
                        &mut idx,
                        inline,
                        "Missing height percentage value after --height-percentage",
                    )?;
                    let percentage = value.parse::<f32>().map_err(|_| {
                        "Invalid height percentage value. Must be a number between 0 and 100."
                            .to_string()
                    })?;
                    if percentage <= 0.0 || percentage > 100.0 {
                        return Err("Height percentage must be between 0 and 100.".to_string());
                    }
                    parsed.options.height_percentage = Some(percentage);
                }
                "preview" => {
                    let value = take_value(
                        args,
                        &mut idx,
                        inline,
                        "Missing preview command after --preview",
                    )?;
                    add_preview_rule(
                        &mut parsed.options.preview_rules,
                        &value,
                        &mut has_default_preview,
                    )?;
                }
                "filter" => {
                    let value = take_value(args, &mut idx, inline, "Missing query after --filter")?;
                    parsed.filter = Some(value);
                }
                _ => return Err(format!("Unknown flag: --{name}")),
            }
        } else if arg.len() > 1 && arg.starts_with('-') {
            let flags: Vec<char> = arg.chars().skip(1).collect();
            let mut pos = 0;
            while pos < flags.len() {
                match flags[pos] {
                    // Acted on by plan_cli_action before parsing.
                    'V' | 'h' => {}
                    'm' => parsed.options.multi_select = true,
                    'n' => parsed.options.line_number = true,
                    'p' => {
                        // Everything after -p in the same token is its value.
                        let attached: String = flags[pos + 1..].iter().collect();
                        let value = if attached.is_empty() {
                            take_value(
                                args,
                                &mut idx,
                                None,
                                "Missing preview command after --preview",
                            )?
                        } else {
                            attached
                        };
                        add_preview_rule(
                            &mut parsed.options.preview_rules,
                            &value,
                            &mut has_default_preview,
                        )?;
                        break;
                    }
                    'f' => {
                        let attached: String = flags[pos + 1..].iter().collect();
                        let value = if attached.is_empty() {
                            take_value(args, &mut idx, None, "Missing query after --filter")?
                        } else {
                            attached
                        };
                        parsed.filter = Some(value);
                        break;
                    }
                    unknown => return Err(format!("Unknown flag: -{unknown}")),
                }
                pos += 1;
            }
        } else {
            // Plain positional. A lone "-" means "read stdin".
            parsed.positionals.push(arg.clone());
        }

        idx += 1;
    }

    Ok(parsed)
}

/// Return the value of a flag, either inline (`--flag=value`) or from the next
/// argument, advancing `idx` past a consumed argument.
fn take_value(
    args: &[String],
    idx: &mut usize,
    inline: Option<String>,
    missing: &str,
) -> Result<String, String> {
    match inline {
        Some(value) => Ok(value),
        None => {
            *idx += 1;
            args.get(*idx).cloned().ok_or_else(|| missing.to_string())
        }
    }
}

/// Add a preview rule, rejecting a second default (no-extension) rule.
fn add_preview_rule(
    rules: &mut Vec<PreviewRule>,
    spec: &str,
    has_default: &mut bool,
) -> Result<(), String> {
    let rule = PreviewRule::parse(spec)?;
    if rule.exts.is_empty() {
        if *has_default {
            return Err(
                "Multiple default preview rules specified. Only one default rule is allowed."
                    .to_string(),
            );
        }
        *has_default = true;
    }
    rules.push(rule);
    Ok(())
}

/// Decide where items come from.
///
/// Explicit arguments win over piped stdin, so `ff file.txt` works inside a
/// script whose stdin happens to be a pipe. With no positionals, piped stdin is
/// used; a lone `-` always means stdin.
pub fn resolve_input(positionals: &[String], stdin_piped: bool) -> Result<InputSpec, String> {
    if positionals.is_empty() {
        if stdin_piped {
            return Ok(InputSpec::Stdin);
        }
        return Err("Missing required argument: input-source or items".to_string());
    }

    if positionals.len() == 1 {
        let source = &positionals[0];
        if source == "-" {
            return Ok(InputSpec::Stdin);
        }
        let is_special = source.starts_with("unix://")
            || source.starts_with("http://")
            || source.starts_with("https://")
            || source.starts_with("dir:");
        if is_special || std::path::Path::new(source).exists() {
            return Ok(InputSpec::Source(source.clone()));
        }
    }

    Ok(InputSpec::Items(positionals.to_vec()))
}

/// Plan the CLI action based on command line arguments.
pub fn plan_cli_action(args: &[String]) -> CliAction {
    // Help and version win over everything else, including invalid input.
    if let Some(flag) = args
        .iter()
        .skip(1)
        .take_while(|arg| arg.as_str() != "--")
        .find(|arg| matches!(arg.as_str(), "-h" | "--help" | "-V" | "--version"))
    {
        return if matches!(flag.as_str(), "-V" | "--version") {
            CliAction::ShowVersion
        } else {
            CliAction::ShowHelp
        };
    }

    let parsed = match parse_args(args) {
        Ok(parsed) => parsed,
        Err(message) => return CliAction::Error(message),
    };

    let input = match resolve_input(&parsed.positionals, super::tty::is_stdin_piped()) {
        Ok(input) => input,
        Err(message) => return CliAction::Error(message),
    };

    match parsed.filter {
        Some(query) => CliAction::Filter {
            input,
            query,
            line_number: parsed.options.line_number,
        },
        None => CliAction::Run {
            input,
            options: parsed.options,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn to_args(args: &[&str]) -> Vec<String> {
        args.iter().map(|s| s.to_string()).collect()
    }

    fn parse(args: &[&str]) -> ParsedArgs {
        parse_args(&to_args(args)).expect("arguments should parse")
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
    fn help_wins_over_other_flags_and_invalid_input() {
        let args = to_args(&["ff", "--height", "abc", "--help"]);
        assert_eq!(plan_cli_action(&args), CliAction::ShowHelp);
    }

    #[test]
    fn flags_may_precede_positionals() {
        let parsed = parse(&["ff", "--height", "10", "apple", "banana"]);
        assert_eq!(parsed.options.height, Some(10));
        assert_eq!(parsed.positionals, vec!["apple", "banana"]);
    }

    #[test]
    fn flags_may_be_interleaved_with_positionals() {
        let parsed = parse(&["ff", "apple", "-m", "banana", "--line-number"]);
        assert!(parsed.options.multi_select);
        assert!(parsed.options.line_number);
        assert_eq!(parsed.positionals, vec!["apple", "banana"]);
    }

    #[test]
    fn flags_accept_inline_values() {
        let parsed = parse(&["ff", "--height=12", "--filter=app", "apple"]);
        assert_eq!(parsed.options.height, Some(12));
        assert_eq!(parsed.filter.as_deref(), Some("app"));
        assert_eq!(parsed.positionals, vec!["apple"]);
    }

    #[test]
    fn double_dash_ends_flag_parsing() {
        let parsed = parse(&["ff", "--multi-select", "--", "--height", "-n"]);
        assert!(parsed.options.multi_select);
        assert_eq!(parsed.positionals, vec!["--height", "-n"]);
    }

    #[test]
    fn lone_dash_is_a_positional() {
        let parsed = parse(&["ff", "-"]);
        assert_eq!(parsed.positionals, vec!["-"]);
    }

    #[test]
    fn clusters_short_flags() {
        let parsed = parse(&["ff", "-mn", "apple"]);
        assert!(parsed.options.multi_select);
        assert!(parsed.options.line_number);
        assert_eq!(parsed.positionals, vec!["apple"]);
    }

    #[test]
    fn short_preview_flag_takes_attached_value() {
        let parsed = parse(&["ff", "-pbat", "apple"]);
        assert_eq!(parsed.options.preview_rules[0].cmd, "bat");
        assert_eq!(parsed.positionals, vec!["apple"]);
    }

    #[test]
    fn short_filter_flag_takes_value() {
        let parsed = parse(&["ff", "-f", "app", "apple"]);
        assert_eq!(parsed.filter.as_deref(), Some("app"));
    }

    #[test]
    fn rejects_unknown_long_flag() {
        let err = parse_args(&to_args(&["ff", "--bogus", "apple"])).unwrap_err();
        assert!(err.contains("Unknown flag: --bogus"), "got: {err}");
    }

    #[test]
    fn rejects_unknown_short_flag() {
        let err = parse_args(&to_args(&["ff", "-z", "apple"])).unwrap_err();
        assert!(err.contains("Unknown flag: -z"), "got: {err}");
    }

    #[test]
    fn rejects_value_on_boolean_flag() {
        let err = parse_args(&to_args(&["ff", "--multi-select=yes"])).unwrap_err();
        assert!(err.contains("does not take a value"), "got: {err}");
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
    fn detects_missing_preview_command() {
        let args = to_args(&["ff", "file.txt", "--preview"]);
        assert!(matches!(plan_cli_action(&args), CliAction::Error(_)));
    }

    #[test]
    fn detects_missing_filter_query() {
        let args = to_args(&["ff", "file.txt", "--filter"]);
        assert!(matches!(plan_cli_action(&args), CliAction::Error(_)));
    }

    #[test]
    fn rejects_second_default_preview_rule() {
        let err = parse_args(&to_args(&["ff", "-p", "cat", "-p", "bat"])).unwrap_err();
        assert!(err.contains("Multiple default preview rules"), "got: {err}");
    }

    #[test]
    fn allows_multiple_filtered_preview_rules() {
        let parsed = parse(&["ff", "-p", "bat {rs}", "-p", "cat"]);
        assert_eq!(parsed.options.preview_rules.len(), 2);
    }

    #[test]
    fn resolve_input_prefers_explicit_arguments_over_piped_stdin() {
        let positionals = to_args(&["apple", "banana"]);
        assert_eq!(
            resolve_input(&positionals, true),
            Ok(InputSpec::Items(positionals.clone()))
        );
    }

    #[test]
    fn resolve_input_uses_stdin_without_positionals() {
        assert_eq!(resolve_input(&[], true), Ok(InputSpec::Stdin));
    }

    #[test]
    fn resolve_input_maps_lone_dash_to_stdin() {
        assert_eq!(resolve_input(&to_args(&["-"]), false), Ok(InputSpec::Stdin));
    }

    #[test]
    fn resolve_input_errors_without_positionals_or_pipe() {
        let err = resolve_input(&[], false).unwrap_err();
        assert!(err.contains("Missing required argument"), "got: {err}");
    }

    #[test]
    fn resolve_input_recognises_special_sources() {
        for source in [
            "unix:///tmp/ff.sock",
            "http://example.com/items",
            "https://example.com/items",
            "dir:/tmp",
        ] {
            assert_eq!(
                resolve_input(&to_args(&[source]), false),
                Ok(InputSpec::Source(source.to_string())),
                "source: {source}"
            );
        }
    }

    #[test]
    fn resolve_input_recognises_existing_paths() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let path = temp_dir.path().join("items.txt");
        std::fs::write(&path, "a\nb\n").unwrap();
        let positionals = vec![path.to_string_lossy().to_string()];

        assert_eq!(
            resolve_input(&positionals, false),
            Ok(InputSpec::Source(positionals[0].clone()))
        );
    }

    #[test]
    fn resolve_input_treats_multiple_positionals_as_items() {
        let positionals = to_args(&["one", "two"]);
        assert_eq!(
            resolve_input(&positionals, false),
            Ok(InputSpec::Items(positionals.clone()))
        );
    }

    #[test]
    fn resolve_input_treats_missing_path_as_literal_item() {
        let positionals = to_args(&["does-not-exist-anywhere.txt"]);
        assert_eq!(
            resolve_input(&positionals, false),
            Ok(InputSpec::Items(positionals.clone()))
        );
    }

    #[test]
    fn filter_flag_produces_filter_action() {
        let positionals = to_args(&["apple"]);
        let input = InputSpec::Items(positionals);
        assert_eq!(
            plan_cli_action(&to_args(&["ff", "--filter", "app", "apple"])),
            CliAction::Filter {
                input,
                query: "app".to_string(),
                line_number: false,
            }
        );
    }
}
