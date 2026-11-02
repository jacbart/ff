use std::env;
use std::fs;
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

/// Read items from a file.
pub fn read_items_from_file(file_path: &str) -> Result<Vec<String>, String> {
    match fs::read_to_string(file_path) {
        Ok(content) => {
            let items: Vec<String> = content.lines().map(|l| l.trim().to_string()).collect();
            Ok(items)
        }
        Err(e) => Err(format!("Failed to read file: {e}")),
    }
}

/// List files in a directory.
pub fn list_files_in_directory(dir_path: &str) -> Result<Vec<String>, String> {
    match fs::read_dir(dir_path) {
        Ok(entries) => {
            let mut files = Vec::new();
            for entry in entries {
                match entry {
                    Ok(entry) => {
                        if let Some(file_name) = entry.file_name().to_str() {
                            files.push(file_name.to_string());
                        }
                    }
                    Err(e) => return Err(format!("Failed to read directory entry: {e}")),
                }
            }
            files.sort();
            Ok(files)
        }
        Err(e) => Err(format!("Failed to read directory: {e}")),
    }
}

/// Check if a path looks like a file path.
pub fn looks_like_file_path(path: &str) -> bool {
    path.contains('/') || path.contains('\\') || path.contains('.')
}

/// Process items from file or direct input.
pub fn process_items(items: Vec<String>) -> Result<Vec<String>, String> {
    // If items is a single file path, read from file
    let processed_items = if items.len() == 1 {
        let item = &items[0];
        if let Some(dir_path) = item.strip_prefix("dir:") {
            // Directory path
            list_files_in_directory(dir_path)?
        } else if looks_like_file_path(item) {
            read_items_from_file(item)?
        } else {
            items
        }
    } else {
        items
    };

    if processed_items.is_empty() {
        return Err("No items to search through".to_string());
    }

    Ok(processed_items)
}

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
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn test_looks_like_file_path() {
        assert!(looks_like_file_path("file.txt"));
        assert!(looks_like_file_path("path/file.txt"));
        assert!(looks_like_file_path("path\\file.txt"));
        assert!(looks_like_file_path("file.name"));
        assert!(!looks_like_file_path("justtext"));
        assert!(!looks_like_file_path(""));
    }

    #[test]
    fn test_read_items_from_file_success() {
        // Create a temporary file for testing
        let temp_file = PathBuf::from("test_items.txt");
        fs::write(&temp_file, "item1\nitem2\n\nitem3\n").unwrap();

        let result = read_items_from_file("test_items.txt");
        assert!(result.is_ok());
        let items = result.unwrap();
        // Updated expectation: empty lines are preserved
        assert_eq!(items, vec!["item1", "item2", "", "item3"]);

        // Clean up
        fs::remove_file(&temp_file).unwrap();
    }

    #[test]
    fn test_read_items_from_file_empty() {
        // Create a temporary empty file
        let temp_file = PathBuf::from("test_empty.txt");
        fs::write(&temp_file, "").unwrap();

        let result = read_items_from_file("test_empty.txt");
        assert!(result.is_ok());
        let items = result.unwrap();
        assert_eq!(items, Vec::<String>::new());

        // Clean up
        fs::remove_file(&temp_file).unwrap();
    }

    #[test]
    fn test_read_items_from_file_with_whitespace() {
        // Create a temporary file with whitespace
        let temp_file = PathBuf::from("test_whitespace.txt");
        fs::write(&temp_file, "  item1  \n\n  item2  \n  \n").unwrap();

        let result = read_items_from_file("test_whitespace.txt");
        assert!(result.is_ok());
        let items = result.unwrap();
        // Updated expectation: empty lines are preserved
        assert_eq!(items, vec!["item1", "", "item2", ""]);

        // Clean up
        fs::remove_file(&temp_file).unwrap();
    }

    #[test]
    fn test_read_items_from_file_nonexistent() {
        let result = read_items_from_file("nonexistent_file.txt");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to read file"));
    }

    #[test]
    fn test_list_files_in_directory_success() {
        // Create a temporary directory for testing
        let temp_dir = PathBuf::from("test_dir_success");
        // Remove directory if it already exists
        if temp_dir.exists() {
            let _ = fs::remove_dir_all(&temp_dir);
        }
        fs::create_dir(&temp_dir).unwrap();
        fs::write(temp_dir.join("file1.txt"), "content1").unwrap();
        fs::write(temp_dir.join("file2.txt"), "content2").unwrap();

        let result = list_files_in_directory(temp_dir.to_str().unwrap());
        assert!(result.is_ok());
        let files = result.unwrap();
        assert_eq!(files, vec!["file1.txt", "file2.txt"]);

        // Clean up
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_list_files_in_directory_empty() {
        // Create a temporary empty directory
        let temp_dir = PathBuf::from("test_empty_dir");
        fs::create_dir(&temp_dir).unwrap();

        let result = list_files_in_directory(temp_dir.to_str().unwrap());
        assert!(result.is_ok());
        let files = result.unwrap();
        assert_eq!(files, Vec::<String>::new());

        // Clean up
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_list_files_in_directory_nonexistent() {
        let result = list_files_in_directory("nonexistent_dir");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to read directory"));
    }

    #[test]
    fn test_process_items_direct() {
        let items = vec!["item1".to_string(), "item2".to_string()];
        let result = process_items(items);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec!["item1", "item2"]);
    }

    #[test]
    fn test_process_items_empty() {
        let items = vec![];
        let result = process_items(items);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "No items to search through");
    }

    #[test]
    fn test_process_items_single_file_path() {
        // Create a temporary file
        let temp_file = PathBuf::from("test_process.txt");
        fs::write(&temp_file, "file_item1\nfile_item2").unwrap();

        let items = vec!["test_process.txt".to_string()];
        let result = process_items(items);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec!["file_item1", "file_item2"]);

        // Clean up
        fs::remove_file(&temp_file).unwrap();
    }

    #[test]
    fn test_process_items_single_non_file_path() {
        let items = vec!["justtext".to_string()];
        let result = process_items(items);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec!["justtext"]);
    }

    #[test]
    fn test_process_items_single_directory_path() {
        // Create a temporary directory
        let temp_dir = PathBuf::from("test_dir");
        // Remove directory if it already exists
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).unwrap();
        }
        fs::create_dir(&temp_dir).unwrap();
        fs::write(temp_dir.join("file1.txt"), "content1").unwrap();
        fs::write(temp_dir.join("file2.txt"), "content2").unwrap();

        let items = vec!["dir:".to_string() + temp_dir.to_str().unwrap()];
        let result = process_items(items);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec!["file1.txt", "file2.txt"]);

        // Clean up
        fs::remove_dir_all(&temp_dir).unwrap();
    }

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
