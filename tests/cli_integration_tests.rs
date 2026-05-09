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

#[tokio::test]
async fn test_file_input() {
    let (temp_dir, file_path) = create_temp_file("file_item1\nfile_item2\nfile_item3");

    let read_result = read_input(&file_path).await;
    assert!(read_result.is_ok());
    assert_eq!(
        read_result.unwrap(),
        vec!["file_item1", "file_item2", "file_item3"]
    );

    temp_dir.close().unwrap();
}

#[tokio::test]
async fn test_nonexistent_file() {
    let read_result = read_input("nonexistent_file.txt").await;
    assert!(read_result.is_ok());
    assert_eq!(read_result.unwrap(), vec!["nonexistent_file.txt"]);
}

#[tokio::test]
async fn test_empty_file() {
    let (temp_dir, file_path) = create_temp_file("");

    let read_result = read_input(&file_path).await;
    assert!(read_result.is_ok());
    assert_eq!(read_result.unwrap(), Vec::<String>::new());

    temp_dir.close().unwrap();
}

#[tokio::test]
async fn test_file_with_whitespace_only() {
    let (temp_dir, file_path) = create_temp_file("   \n  \n  \n");

    let read_result = read_input(&file_path).await;
    assert!(read_result.is_ok());
    assert_eq!(read_result.unwrap(), vec!["   ", "  ", "  "]);

    temp_dir.close().unwrap();
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
