use std::env;
use std::io::Write;
use std::path::Path;

use crate::cli::planner::{CliAction, InputSpec, RunOptions, plan_cli_action};
use crate::cli::tty::check_tty_requirements;
use crate::fuzzy::FuzzyFinder;
use crate::get_build_info;
use crate::help;
use crate::input::{read_input, read_piped_stdin, reopen_stdin_from_tty, send_input_to_channel};
use crate::tui::TuiConfig;
use crate::tui::ui::{create_items_channel, run_tui_with_config};

/// Validate that TTY requirements are met for interactive mode.
pub fn validate_tty_requirements() -> Result<(), String> {
    if !check_tty_requirements() {
        return Err("Interactive selection requires a TTY.".to_string());
    }
    Ok(())
}

/// Handle TUI results.
pub fn handle_tui_results(selected: Vec<(usize, String)>) -> Vec<String> {
    selected.into_iter().map(|(_, item)| item).collect()
}

/// Build the TUI configuration from parsed options.
fn build_tui_config(options: &RunOptions) -> TuiConfig {
    TuiConfig {
        fullscreen: options.height.is_none() && options.height_percentage.is_none(),
        height: options.height,
        height_percentage: options.height_percentage,
        show_help_text: options.show_help_text,
        show_loading_indicator: true,
        loading_message: None,
        ready_message: None,
        preview_rules: options.preview_rules.clone(),
        preview_auto: options.preview_auto,
    }
}

/// The file a selection came from, when the input was a single local file.
///
/// Used to print `file:line` instead of a bare line number.
fn single_file_source(input: &InputSpec) -> Option<String> {
    match input {
        InputSpec::Source(path)
            if !path.starts_with("unix://")
                && !path.starts_with("http://")
                && !path.starts_with("https://") =>
        {
            let path = Path::new(path);
            path.is_file().then(|| path.to_string_lossy().into_owned())
        }
        _ => None,
    }
}

fn boxed_error(message: impl Into<String>) -> Box<dyn std::error::Error> {
    Box::new(std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        message.into(),
    ))
}

/// Write results through a buffered stdout, treating a closed pipe as success.
///
/// `ff --filter … | head` closes the read end early; `println!` would panic with
/// "failed printing to stdout" instead of exiting quietly.
fn with_stdout<F>(write: F) -> Result<(), Box<dyn std::error::Error>>
where
    F: FnOnce(&mut dyn std::io::Write) -> std::io::Result<()>,
{
    let stdout = std::io::stdout();
    let mut out = std::io::BufWriter::new(stdout.lock());
    match write(&mut out).and_then(|()| out.flush()) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::BrokenPipe => Ok(()),
        Err(err) => Err(Box::new(err)),
    }
}

/// Print one result, honouring `--line-number`.
fn print_item(
    out: &mut dyn std::io::Write,
    item: Option<&str>,
    original_index: usize,
    line_number: bool,
    source_file: Option<&str>,
) -> std::io::Result<()> {
    if !line_number {
        if let Some(item) = item {
            writeln!(out, "{item}")?;
        }
        return Ok(());
    }
    match source_file {
        Some(file) => writeln!(out, "{}:{}", file, original_index + 1),
        None => writeln!(out, "{}", original_index + 1),
    }
}

/// Run the interactive TUI.
fn run_tui(input: InputSpec, options: RunOptions) -> Result<(), Box<dyn std::error::Error>> {
    validate_tty_requirements()?;

    // Piped input is consumed before the TUI starts; stdin is then reopened from
    // /dev/tty so key events can be read.
    let stdin_items = if matches!(input, InputSpec::Stdin) {
        let items = read_piped_stdin().map_err(boxed_error)?;
        if items.is_empty() {
            return Err(boxed_error("No items provided via stdin"));
        }
        reopen_stdin_from_tty().map_err(boxed_error)?;
        Some(items)
    } else {
        None
    };

    let source_file = single_file_source(&input);
    let config = build_tui_config(&options);
    let multi_select = options.multi_select;
    let line_number = options.line_number;

    let rt = tokio::runtime::Runtime::new()?;
    let selected = rt.block_on(async {
        let (sender, receiver) = create_items_channel();

        match input {
            InputSpec::Stdin => {
                let items = stdin_items.unwrap_or_default();
                tokio::spawn(async move {
                    for item in items {
                        let _ = sender.send(item).await;
                    }
                });
            }
            InputSpec::Source(source) => {
                tokio::spawn(async move {
                    let _ = send_input_to_channel(&source, sender).await;
                });
            }
            InputSpec::Items(items) => {
                tokio::spawn(async move {
                    for item in items {
                        let _ = sender.send(item).await;
                    }
                });
            }
        }

        run_tui_with_config(receiver, multi_select, config)
            .await
            .map_err(|e| e as Box<dyn std::error::Error>)
    })?;

    with_stdout(|out| {
        for (index, item) in selected {
            print_item(out, Some(&item), index, line_number, source_file.as_deref())?;
        }
        Ok(())
    })
}

/// Non-interactive filter mode: print every match, exit 1 when nothing matches.
fn run_filter(
    input: InputSpec,
    query: &str,
    line_number: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let source_file = single_file_source(&input);
    let rt = tokio::runtime::Runtime::new()?;

    let items = match input {
        InputSpec::Stdin => read_piped_stdin().map_err(boxed_error)?,
        InputSpec::Source(source) => rt
            .block_on(read_input(&source))
            .map_err(|e| boxed_error(e.to_string()))?,
        InputSpec::Items(items) => items,
    };

    let mut finder = rt.block_on(FuzzyFinder::with_items_async(items, false));
    rt.block_on(finder.set_query(query.to_string()));

    let matches = finder.get_filtered_items();
    if matches.is_empty() {
        // fzf-compatible: no matches is a non-zero exit for scripts.
        std::process::exit(1);
    }

    with_stdout(|out| {
        for (position, item) in matches.iter().enumerate() {
            let original_index = finder.get_original_index(position).unwrap_or(position);
            print_item(
                out,
                Some(item),
                original_index,
                line_number,
                source_file.as_deref(),
            )?;
        }
        Ok(())
    })
}

/// Run the CLI application.
pub fn cli_main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    match plan_cli_action(&args) {
        CliAction::ShowVersion => with_stdout(|out| writeln!(out, "{}", get_build_info())),
        CliAction::ShowHelp => {
            help::print_usage();
            Ok(())
        }
        CliAction::Run { input, options } => run_tui(input, options),
        CliAction::Filter {
            input,
            query,
            line_number,
        } => run_filter(input, &query, line_number),
        CliAction::Error(message) => {
            eprintln!("Error: {message}");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_tui_results() {
        let selected = vec![(0, "result1".to_string()), (1, "result2".to_string())];
        let result = handle_tui_results(selected);
        assert_eq!(result, vec!["result1", "result2"]);
    }

    #[test]
    fn test_handle_tui_results_empty() {
        let selected = vec![];
        let result = handle_tui_results(selected);
        assert_eq!(result, Vec::<String>::new());
    }

    #[test]
    fn test_validate_tty_requirements() {
        // This test depends on the actual TTY check implementation
        // We can't easily mock this in a unit test, so we just test that it doesn't panic
        let _result = validate_tty_requirements();
        // If we get here, it didn't panic
    }
}
