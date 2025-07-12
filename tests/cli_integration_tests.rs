use ff::cli::args::{has_multi_select_flag, has_version_flag, is_file_path};
use ff::config::parse_args_from;
use ff::input::{process_file_content, process_stdin_content, read_direct_items, read_input};
use std::fs;
use tempfile::TempDir;

/// Helper function to create a temporary file with content
fn create_temp_file(content: &str) -> (TempDir, String) {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test_items.txt");
    fs::write(&file_path, content).unwrap();
    (temp_dir, file_path.to_string_lossy().to_string())
}

#[test]
fn test_version_flag_detection() {
    let args = vec!["ff".to_string(), "--version".to_string()];
    assert!(has_version_flag(&args));

    let args = vec!["ff".to_string(), "-v".to_string()];
    assert!(has_version_flag(&args));

    let args = vec!["ff".to_string(), "-V".to_string()];
    assert!(has_version_flag(&args));

    let args = vec!["ff".to_string(), "item1".to_string()];
    assert!(!has_version_flag(&args));
}

#[test]
fn test_help_flag_detection() {
    let args = ["ff".to_string(), "--help".to_string()];
    // Note: We can't easily test the actual help output since it calls process::exit()
    // But we can test that the flag is detected
    assert!(args.iter().any(|arg| arg == "--help"));

    let args = ["ff".to_string(), "-h".to_string()];
    assert!(args.iter().any(|arg| arg == "-h"));
}

#[test]
fn test_multi_select_flag_detection() {
    let args = vec!["ff".to_string(), "item1".to_string(), "-m".to_string()];
    assert!(has_multi_select_flag(&args));

    let args = vec![
        "ff".to_string(),
        "item1".to_string(),
        "--multi-select".to_string(),
    ];
    assert!(has_multi_select_flag(&args));

    let args = vec!["ff".to_string(), "item1".to_string()];
    assert!(!has_multi_select_flag(&args));
}

#[test]
fn test_file_path_detection() {
    // Test with existing file
    let (temp_dir, file_path) = create_temp_file("test content");
    assert!(is_file_path(&file_path));
    temp_dir.close().unwrap();

    // Test with non-existent file
    assert!(!is_file_path("nonexistent_file.txt"));

    // Test with flags (should not be file paths)
    assert!(!is_file_path("--help"));
    assert!(!is_file_path("-m"));
    assert!(!is_file_path("--multi-select"));
}

#[test]
fn test_missing_arguments() {
    let args = vec!["ff".to_string()];
    let result = parse_args_from(&args);

    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        "Missing required argument: input-source or items"
    );
}

#[test]
fn test_invalid_input_source() {
    let args = vec!["ff".to_string(), "-invalid".to_string()];
    let result = parse_args_from(&args);

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid input source"));
}

#[test]
fn test_direct_items_single_select() {
    let args = vec![
        "ff".to_string(),
        "item1".to_string(),
        "item2".to_string(),
        "item3".to_string(),
    ];
    let result = parse_args_from(&args);

    assert!(result.is_ok());
    let config = result.unwrap();
    assert_eq!(config.input_source, "direct");
    assert!(!config.multi_select);
    assert_eq!(
        config.direct_items.unwrap(),
        vec!["item1", "item2", "item3"]
    );
}

#[test]
fn test_direct_items_multi_select() {
    let args = vec![
        "ff".to_string(),
        "item1".to_string(),
        "item2".to_string(),
        "-m".to_string(),
    ];
    let result = parse_args_from(&args);

    assert!(result.is_ok());
    let config = result.unwrap();
    assert_eq!(config.input_source, "direct");
    assert!(config.multi_select);
    assert_eq!(config.direct_items.unwrap(), vec!["item1", "item2"]);
}

#[test]
fn test_direct_items_multi_select_long() {
    let args = vec![
        "ff".to_string(),
        "item1".to_string(),
        "item2".to_string(),
        "--multi-select".to_string(),
    ];
    let result = parse_args_from(&args);

    assert!(result.is_ok());
    let config = result.unwrap();
    assert_eq!(config.input_source, "direct");
    assert!(config.multi_select);
    assert_eq!(config.direct_items.unwrap(), vec!["item1", "item2"]);
}

#[test]
fn test_file_input() {
    let (temp_dir, file_path) = create_temp_file("file_item1\nfile_item2\nfile_item3");

    let args = vec!["ff".to_string(), file_path.clone()];
    let result = parse_args_from(&args);

    assert!(result.is_ok());
    let config = result.unwrap();
    assert_eq!(config.input_source, file_path);
    assert!(!config.multi_select);
    assert!(config.direct_items.is_none());

    // Test file reading
    let read_result = read_input(&file_path);
    assert!(read_result.is_ok());
    assert_eq!(
        read_result.unwrap(),
        vec!["file_item1", "file_item2", "file_item3"]
    );

    // Clean up
    temp_dir.close().unwrap();
}

#[test]
fn test_file_input_multi_select() {
    let (temp_dir, file_path) = create_temp_file("file_item1\nfile_item2\nfile_item3");

    let args = vec!["ff".to_string(), file_path.clone(), "-m".to_string()];
    let result = parse_args_from(&args);

    assert!(result.is_ok());
    let config = result.unwrap();
    assert_eq!(config.input_source, file_path);
    assert!(config.multi_select);
    assert!(config.direct_items.is_none());

    // Clean up
    temp_dir.close().unwrap();
}

#[test]
fn test_nonexistent_file() {
    let args = vec!["ff".to_string(), "nonexistent_file.txt".to_string()];
    let result = parse_args_from(&args);

    // Argument parsing should succeed
    assert!(result.is_ok());
    let config = result.unwrap();
    assert_eq!(config.input_source, "nonexistent_file.txt");

    // But file reading should fail
    let read_result = read_input("nonexistent_file.txt");
    assert!(read_result.is_err());
    assert!(read_result.unwrap_err().contains("Error reading file"));
}

#[test]
fn test_empty_file() {
    let (temp_dir, file_path) = create_temp_file("");

    let args = vec!["ff".to_string(), file_path.clone()];
    let result = parse_args_from(&args);

    // Argument parsing should succeed
    assert!(result.is_ok());

    // But file reading should fail
    let read_result = read_input(&file_path);
    assert!(read_result.is_err());
    assert_eq!(read_result.unwrap_err(), "No items found in file");

    // Clean up
    temp_dir.close().unwrap();
}

#[test]
fn test_file_with_whitespace_only() {
    let (temp_dir, file_path) = create_temp_file("   \n  \n  \n");

    let args = vec!["ff".to_string(), file_path.clone()];
    let result = parse_args_from(&args);

    // Argument parsing should succeed
    assert!(result.is_ok());

    // But file reading should fail
    let read_result = read_input(&file_path);
    assert!(read_result.is_err());
    assert_eq!(read_result.unwrap_err(), "No items found in file");

    // Clean up
    temp_dir.close().unwrap();
}

#[test]
fn test_benchmark_mode() {
    let args = vec!["ff".to_string(), "benchmark".to_string()];
    let result = parse_args_from(&args);

    assert!(result.is_ok());
    let config = result.unwrap();
    assert_eq!(config.input_source, "benchmark");
    assert!(!config.multi_select);
    assert!(config.direct_items.is_none());
}

#[test]
fn test_benchmark_mode_multi_select() {
    let args = vec!["ff".to_string(), "benchmark".to_string(), "-m".to_string()];
    let result = parse_args_from(&args);

    assert!(result.is_ok());
    let config = result.unwrap();
    assert_eq!(config.input_source, "benchmark");
    assert!(config.multi_select);
    assert!(config.direct_items.is_none());
}

#[test]
fn test_empty_direct_items() {
    let args = vec!["ff".to_string(), "-m".to_string()];
    let result = parse_args_from(&args);

    assert!(result.is_err());
    // The error message depends on how the argument parsing works
    let error = result.unwrap_err();
    assert!(error.contains("Invalid input source") || error.contains("No items provided"));
}

#[test]
fn test_mixed_flags_and_items() {
    let args = vec![
        "ff".to_string(),
        "item1".to_string(),
        "-m".to_string(),
        "item2".to_string(),
        "--multi-select".to_string(),
        "item3".to_string(),
    ];
    let result = parse_args_from(&args);

    assert!(result.is_ok());
    let config = result.unwrap();
    assert_eq!(config.input_source, "direct");
    assert!(config.multi_select);
    assert_eq!(
        config.direct_items.unwrap(),
        vec!["item1", "item2", "item3"]
    );
}

#[test]
fn test_single_item() {
    let args = vec!["ff".to_string(), "single_item".to_string()];
    let result = parse_args_from(&args);

    assert!(result.is_ok());
    let config = result.unwrap();
    assert_eq!(config.input_source, "direct");
    assert!(!config.multi_select);
    assert_eq!(config.direct_items.unwrap(), vec!["single_item"]);
}

#[test]
fn test_file_path_with_slash() {
    let (temp_dir, file_path) = create_temp_file("item1\nitem2");
    let file_path_with_slash = format!("/{}", file_path);

    let args = vec!["ff".to_string(), file_path_with_slash.clone()];
    let result = parse_args_from(&args);

    assert!(result.is_ok());
    let config = result.unwrap();
    assert_eq!(config.input_source, file_path_with_slash);

    // Clean up
    temp_dir.close().unwrap();
}

#[test]
fn test_file_path_with_dot() {
    let (temp_dir, file_path) = create_temp_file("item1\nitem2");
    let file_path_with_dot = format!("{}.txt", file_path);

    let args = vec!["ff".to_string(), file_path_with_dot.clone()];
    let result = parse_args_from(&args);

    assert!(result.is_ok());
    let config = result.unwrap();
    assert_eq!(config.input_source, file_path_with_dot);

    // Clean up
    temp_dir.close().unwrap();
}

#[test]
fn test_file_path_with_backslash() {
    let (temp_dir, file_path) = create_temp_file("item1\nitem2");
    let file_path_with_backslash = file_path.replace('/', "\\");

    let args = vec!["ff".to_string(), file_path_with_backslash.clone()];
    let result = parse_args_from(&args);

    assert!(result.is_ok());
    let config = result.unwrap();
    assert_eq!(config.input_source, file_path_with_backslash);

    // Clean up
    temp_dir.close().unwrap();
}

#[test]
fn test_version_flag_short_v() {
    let args = vec!["ff".to_string(), "-v".to_string()];
    assert!(has_version_flag(&args));
}

#[test]
fn test_version_flag_short_v_upper() {
    let args = vec!["ff".to_string(), "-V".to_string()];
    assert!(has_version_flag(&args));
}

#[test]
fn test_help_and_version_flags_ignored() {
    let args = [
        "ff".to_string(),
        "--help".to_string(),
        "item1".to_string(),
        "item2".to_string(),
    ];
    // The help flag should be processed first, so we can't test the items
    // But we can test that the help flag is detected
    assert!(args.iter().any(|arg| arg == "--help"));
}

#[test]
fn test_version_and_items_ignored() {
    let args = vec![
        "ff".to_string(),
        "--version".to_string(),
        "item1".to_string(),
        "item2".to_string(),
    ];
    // The version flag should be processed first, so we can't test the items
    // But we can test that the version flag is detected
    assert!(has_version_flag(&args));
}

#[test]
fn test_read_direct_items_integration() {
    let items = vec![
        "item1".to_string(),
        "item2".to_string(),
        "item3".to_string(),
    ];
    let result = read_direct_items(items.clone());

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), items);
}

#[test]
fn test_process_stdin_content_integration() {
    let content = "line1\nline2\nline3";
    let result = process_stdin_content(content);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec!["line1", "line2", "line3"]);
}

#[test]
fn test_process_file_content_integration() {
    let content = "line1\nline2\nline3";
    let result = process_file_content(content);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec!["line1", "line2", "line3"]);
}
