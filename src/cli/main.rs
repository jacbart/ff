use std::env;
use std::fs;

use crate::bench;
use crate::cli::planner::{plan_cli_action, CliAction};
use crate::cli::tty::check_tty_requirements;
use crate::config;
use crate::get_build_info;
use crate::tui::{run_tui, run_tui_with_config, TuiConfig};

/// Read items from a file.
pub fn read_items_from_file(file_path: &str) -> Result<Vec<String>, String> {
    match fs::read_to_string(file_path) {
        Ok(content) => {
            let items: Vec<String> = content
                .lines()
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty())
                .collect();
            Ok(items)
        }
        Err(e) => Err(format!("Failed to read file: {}", e)),
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
                    Err(e) => return Err(format!("Failed to read directory entry: {}", e)),
                }
            }
            files.sort();
            Ok(files)
        }
        Err(e) => Err(format!("Failed to read directory: {}", e)),
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
        if item.starts_with("dir:") {
            // Directory path
            let dir_path = &item[4..]; // Remove "dir:" prefix
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
pub fn handle_tui_results(selected: Vec<String>) -> Vec<String> {
    selected
}

/// Run TUI with validation and error handling.
pub fn run_tui_with_validation(
    items: Vec<String>,
    multi_select: bool,
) -> Result<Vec<String>, String> {
    let processed_items = process_items(items)?;

    validate_tty_requirements()?;

    match run_tui(processed_items, multi_select) {
        Ok(selected) => Ok(handle_tui_results(selected)),
        Err(err) => Err(format!("TUI error: {}", err)),
    }
}

/// Run TUI with height configuration and validation.
pub fn run_tui_with_height_validation(
    items: Vec<String>,
    multi_select: bool,
    height: Option<u16>,
    height_percentage: Option<f32>,
) -> Result<Vec<String>, String> {
    let processed_items = process_items(items)?;

    validate_tty_requirements()?;

    let config = if let Some(h) = height {
        TuiConfig::with_height(h)
    } else if let Some(p) = height_percentage {
        TuiConfig::with_height_percentage(p)
    } else {
        TuiConfig::fullscreen()
    };

    match run_tui_with_config(processed_items, multi_select, config) {
        Ok(selected) => Ok(handle_tui_results(selected)),
        Err(err) => Err(format!("TUI error: {}", err)),
    }
}

/// Run the CLI application.
pub fn cli_main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    match plan_cli_action(&args) {
        CliAction::ShowVersion => {
            println!("{}", get_build_info());
            Ok(())
        }
        CliAction::ShowHelp => {
            config::print_usage();
            Ok(())
        }
        CliAction::RunBenchmark { multi_select: _ } => {
            bench::run_all_benchmarks();
            Ok(())
        }
        CliAction::RunTui {
            items,
            multi_select,
            height,
            height_percentage,
            show_help_text,
        } => {
            let processed_items = process_items(items)?;
            let mut config = TuiConfig::default();
            config.fullscreen = height.is_none() && height_percentage.is_none();
            config.height = height;
            config.height_percentage = height_percentage;
            config.show_help_text = show_help_text;
            let selected = run_tui_with_config(processed_items, multi_select, config)?;
            for item in selected {
                println!("{}", item);
            }
            Ok(())
        }
        CliAction::Error(msg) => {
            eprintln!("Error: {}", msg);
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
        assert_eq!(items, vec!["item1", "item2", "item3"]);

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
        assert_eq!(items, vec!["item1", "item2"]);

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
        let temp_dir = PathBuf::from("test_dir");
        fs::create_dir(&temp_dir).unwrap();
        fs::write(&temp_dir.join("file1.txt"), "content1").unwrap();
        fs::write(&temp_dir.join("file2.txt"), "content2").unwrap();

        let result = list_files_in_directory(&temp_dir.to_str().unwrap());
        assert!(result.is_ok());
        let files = result.unwrap();
        assert_eq!(files, vec!["file1.txt", "file2.txt"]);

        // Clean up
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_list_files_in_directory_empty() {
        // Create a temporary empty directory
        let temp_dir = PathBuf::from("test_empty_dir");
        fs::create_dir(&temp_dir).unwrap();

        let result = list_files_in_directory(&temp_dir.to_str().unwrap());
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
        fs::create_dir(&temp_dir).unwrap();
        fs::write(&temp_dir.join("file1.txt"), "content1").unwrap();
        fs::write(&temp_dir.join("file2.txt"), "content2").unwrap();

        let items = vec!["dir:".to_string() + &temp_dir.to_str().unwrap()];
        let result = process_items(items);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec!["file1.txt", "file2.txt"]);

        // Clean up
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_handle_tui_results() {
        let selected = vec!["result1".to_string(), "result2".to_string()];
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

    #[test]
    fn test_run_tui_with_validation_success() {
        // This test would require mocking the TUI and TTY functions
        // For now, we'll test the error cases that we can control
        let items = vec![];
        let result = run_tui_with_validation(items, false);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No items to search through"));
    }

    #[test]
    fn test_run_tui_with_validation_with_items() {
        let items = vec!["item1".to_string(), "item2".to_string()];
        // Test only the validation logic, not the actual TUI execution
        // The TUI requires interactive input which we can't test in automated tests
        let processed_items = process_items(items).unwrap();
        assert_eq!(processed_items, vec!["item1", "item2"]);

        // Test that validate_tty_requirements doesn't panic
        let _result = validate_tty_requirements();
        // If we get here, it didn't panic
    }
}
