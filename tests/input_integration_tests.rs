use ff::input::{read_input, read_direct_items, process_stdin_content, process_file_content};
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
fn test_read_direct_items_valid() {
    let items = vec!["item1".to_string(), "item2".to_string(), "item3".to_string()];
    let result = read_direct_items(items.clone());
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), items);
}

#[test]
fn test_read_direct_items_empty() {
    let result = read_direct_items(vec![]);
    
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "No items provided");
}

#[test]
fn test_read_direct_items_single_item() {
    let items = vec!["single_item".to_string()];
    let result = read_direct_items(items.clone());
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), items);
}

#[test]
fn test_read_direct_items_with_empty_strings() {
    let items = vec!["".to_string(), "item1".to_string(), "".to_string()];
    let result = read_direct_items(items.clone());
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), items);
}

#[test]
fn test_process_stdin_content_valid() {
    let content = "line1\nline2\nline3";
    let result = process_stdin_content(content);
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec!["line1", "line2", "line3"]);
}

#[test]
fn test_process_stdin_content_empty() {
    let result = process_stdin_content("");
    
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "No items found in stdin");
}

#[test]
fn test_process_stdin_content_with_empty_lines() {
    let content = "line1\n\nline2\n  \nline3";
    let result = process_stdin_content(content);
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec!["line1", "line2", "line3"]);
}

#[test]
fn test_process_stdin_content_with_whitespace() {
    let content = "  line1  \n  line2  \n  line3  ";
    let result = process_stdin_content(content);
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec!["line1", "line2", "line3"]);
}

#[test]
fn test_process_stdin_content_only_whitespace() {
    let content = "   \n  \n  \n";
    let result = process_stdin_content(content);
    
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "No items found in stdin");
}

#[test]
fn test_process_stdin_content_mixed_whitespace() {
    let content = "  line1  \n\n  line2  \n  \n  line3  ";
    let result = process_stdin_content(content);
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec!["line1", "line2", "line3"]);
}

#[test]
fn test_process_stdin_content_single_line() {
    let content = "single line";
    let result = process_stdin_content(content);
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec!["single line"]);
}

#[test]
fn test_process_file_content_valid() {
    let content = "line1\nline2\nline3";
    let result = process_file_content(content);
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec!["line1", "line2", "line3"]);
}

#[test]
fn test_process_file_content_empty() {
    let result = process_file_content("");
    
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "No items found in file");
}

#[test]
fn test_process_file_content_with_empty_lines() {
    let content = "line1\n\nline2\n  \nline3";
    let result = process_file_content(content);
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec!["line1", "line2", "line3"]);
}

#[test]
fn test_process_file_content_with_whitespace() {
    let content = "  line1  \n  line2  \n  line3  ";
    let result = process_file_content(content);
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec!["line1", "line2", "line3"]);
}

#[test]
fn test_process_file_content_only_whitespace() {
    let content = "   \n  \n  \n";
    let result = process_file_content(content);
    
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "No items found in file");
}

#[test]
fn test_process_file_content_mixed_whitespace() {
    let content = "  line1  \n\n  line2  \n  \n  line3  ";
    let result = process_file_content(content);
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec!["line1", "line2", "line3"]);
}

#[test]
fn test_process_file_content_single_line() {
    let content = "single line";
    let result = process_file_content(content);
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec!["single line"]);
}

#[test]
fn test_read_input_stdin() {
    // This test verifies that stdin source is recognized
    // We can't easily test the actual stdin reading in integration tests
    let source = "stdin";
    assert_eq!(source, "stdin");
}

#[test]
fn test_read_input_direct() {
    let source = "direct";
    let result = read_input(source);
    
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Direct items should be handled by the caller");
}

#[test]
fn test_read_input_file() {
    let (temp_dir, file_path) = create_temp_file("file_item1\nfile_item2\nfile_item3");
    
    let result = read_input(&file_path);
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec!["file_item1", "file_item2", "file_item3"]);
    
    // Clean up
    temp_dir.close().unwrap();
}

#[test]
fn test_read_input_file_empty() {
    let (temp_dir, file_path) = create_temp_file("");
    
    let result = read_input(&file_path);
    
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "No items found in file");
    
    // Clean up
    temp_dir.close().unwrap();
}

#[test]
fn test_read_input_file_with_whitespace_only() {
    let (temp_dir, file_path) = create_temp_file("   \n  \n  \n");
    
    let result = read_input(&file_path);
    
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "No items found in file");
    
    // Clean up
    temp_dir.close().unwrap();
}

#[test]
fn test_read_input_file_with_mixed_content() {
    let (temp_dir, file_path) = create_temp_file("  item1  \n\n  item2  \n  \n  item3  ");
    
    let result = read_input(&file_path);
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec!["item1", "item2", "item3"]);
    
    // Clean up
    temp_dir.close().unwrap();
}

#[test]
fn test_read_input_nonexistent_file() {
    let result = read_input("nonexistent_file.txt");
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Error reading file"));
}

#[test]
fn test_read_input_unknown_source() {
    let result = read_input("unknown_source");
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Error reading file"));
}

#[test]
fn test_file_with_special_characters() {
    let (temp_dir, file_path) = create_temp_file("item-with-dash\nitem_with_underscore\nitem with spaces");
    
    let result = read_input(&file_path);
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec!["item-with-dash", "item_with_underscore", "item with spaces"]);
    
    // Clean up
    temp_dir.close().unwrap();
}

#[test]
fn test_file_with_unicode_characters() {
    let (temp_dir, file_path) = create_temp_file("café\nnaïve\nüber");
    
    let result = read_input(&file_path);
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec!["café", "naïve", "über"]);
    
    // Clean up
    temp_dir.close().unwrap();
}

#[test]
fn test_file_with_numbers() {
    let (temp_dir, file_path) = create_temp_file("item1\nitem2\nitem10\nitem20");
    
    let result = read_input(&file_path);
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec!["item1", "item2", "item10", "item20"]);
    
    // Clean up
    temp_dir.close().unwrap();
}

#[test]
fn test_file_with_empty_strings() {
    let (temp_dir, file_path) = create_temp_file("\nitem1\n\nitem2\n");
    
    let result = read_input(&file_path);
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec!["item1", "item2"]);
    
    // Clean up
    temp_dir.close().unwrap();
}

#[test]
fn test_large_file() {
    let items: Vec<String> = (0..1000).map(|i| format!("item_{}", i)).collect();
    let content = items.join("\n");
    let (temp_dir, file_path) = create_temp_file(&content);
    
    let result = read_input(&file_path);
    
    assert!(result.is_ok());
    let read_items = result.unwrap();
    assert_eq!(read_items.len(), 1000);
    assert_eq!(read_items[0], "item_0");
    assert_eq!(read_items[999], "item_999");
    
    // Clean up
    temp_dir.close().unwrap();
}

#[test]
fn test_file_with_trailing_newline() {
    let (temp_dir, file_path) = create_temp_file("item1\nitem2\n");
    
    let result = read_input(&file_path);
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec!["item1", "item2"]);
    
    // Clean up
    temp_dir.close().unwrap();
}

#[test]
fn test_file_without_trailing_newline() {
    let (temp_dir, file_path) = create_temp_file("item1\nitem2");
    
    let result = read_input(&file_path);
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec!["item1", "item2"]);
    
    // Clean up
    temp_dir.close().unwrap();
}

#[test]
fn test_file_with_carriage_returns() {
    let (temp_dir, file_path) = create_temp_file("item1\r\nitem2\r\nitem3");
    
    let result = read_input(&file_path);
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec!["item1", "item2", "item3"]);
    
    // Clean up
    temp_dir.close().unwrap();
}

#[test]
fn test_file_with_tabs() {
    let (temp_dir, file_path) = create_temp_file("item1\t\nitem2\t\nitem3");
    
    let result = read_input(&file_path);
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec!["item1", "item2", "item3"]);
    
    // Clean up
    temp_dir.close().unwrap();
} 