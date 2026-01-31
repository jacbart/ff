use ff::input::read_input;

#[tokio::test]
async fn test_read_input_from_file() {
    // Create a temporary file for testing
    let temp_file = "test_input_file.txt";
    std::fs::write(temp_file, "item1\nitem2\nitem3").unwrap();

    let result = read_input(temp_file).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec!["item1", "item2", "item3"]);

    // Clean up
    std::fs::remove_file(temp_file).unwrap();
}

#[tokio::test]
async fn test_read_input_from_file_with_empty_lines() {
    // Create a temporary file with empty lines
    let temp_file = "test_input_file_empty.txt";
    std::fs::write(temp_file, "item1\n\nitem2\n\nitem3").unwrap();

    let result = read_input(temp_file).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec!["item1", "", "item2", "", "item3"]);

    // Clean up
    std::fs::remove_file(temp_file).unwrap();
}

#[tokio::test]
async fn test_read_input_from_file_with_whitespace() {
    // Create a temporary file with whitespace
    let temp_file = "test_input_file_whitespace.txt";
    std::fs::write(temp_file, "  item1  \n  item2  \n  item3  ").unwrap();

    let result = read_input(temp_file).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec!["  item1  ", "  item2  ", "  item3  "]);

    // Clean up
    std::fs::remove_file(temp_file).unwrap();
}

#[tokio::test]
async fn test_read_input_from_nonexistent_file() {
    let result = read_input("nonexistent_file.txt").await;
    assert!(result.is_ok());
    let items = result.unwrap();
    assert_eq!(items, vec!["nonexistent_file.txt"]);
}

#[tokio::test]
async fn test_read_input_from_empty_file() {
    // Create a temporary empty file
    let temp_file = "test_empty_file.txt";
    std::fs::write(temp_file, "").unwrap();

    let result = read_input(temp_file).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Vec::<String>::new());

    // Clean up
    std::fs::remove_file(temp_file).unwrap();
}

#[tokio::test]
async fn test_read_input_from_file_with_whitespace_only() {
    // Create a temporary file with only whitespace
    let temp_file = "test_whitespace_file.txt";
    std::fs::write(temp_file, "   \n  \n  \n").unwrap();

    let result = read_input(temp_file).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec!["   ", "  ", "  "]);

    // Clean up
    std::fs::remove_file(temp_file).unwrap();
}

#[tokio::test]
async fn test_read_input_from_file_with_mixed_content() {
    // Create a temporary file with mixed content
    let temp_file = "test_mixed_file.txt";
    std::fs::write(temp_file, "  item1  \n\n  item2  \n  \n  item3  ").unwrap();

    let result = read_input(temp_file).await;
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        vec!["  item1  ", "", "  item2  ", "  ", "  item3  "]
    );

    // Clean up
    std::fs::remove_file(temp_file).unwrap();
}

#[tokio::test]
async fn test_read_input_from_file_with_unicode() {
    // Create a temporary file with unicode characters
    let temp_file = "test_unicode_file.txt";
    std::fs::write(temp_file, "café\nnaïve\nüber").unwrap();

    let result = read_input(temp_file).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec!["café", "naïve", "über"]);

    // Clean up
    std::fs::remove_file(temp_file).unwrap();
}

#[tokio::test]
async fn test_read_input_from_file_with_numbers() {
    // Create a temporary file with numbers
    let temp_file = "test_numbers_file.txt";
    std::fs::write(temp_file, "item1\nitem2\nitem10\nitem20").unwrap();

    let result = read_input(temp_file).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec!["item1", "item2", "item10", "item20"]);

    // Clean up
    std::fs::remove_file(temp_file).unwrap();
}

#[tokio::test]
async fn test_read_input_from_file_with_special_characters() {
    // Create a temporary file with special characters
    let temp_file = "test_special_file.txt";
    std::fs::write(temp_file, "item1\nitem2").unwrap();

    let result = read_input(temp_file).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec!["item1", "item2"]);

    // Clean up
    std::fs::remove_file(temp_file).unwrap();
}

#[tokio::test]
async fn test_read_input_from_file_with_tabs() {
    // Create a temporary file with tabs
    let temp_file = "test_tabs_file.txt";
    std::fs::write(temp_file, "item1\titem2").unwrap();

    let result = read_input(temp_file).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec!["item1\titem2"]);

    // Clean up
    std::fs::remove_file(temp_file).unwrap();
}

#[tokio::test]
async fn test_read_input_from_file_with_carriage_returns() {
    // Create a temporary file with carriage returns
    let temp_file = "test_cr_file.txt";
    std::fs::write(temp_file, "item1\r\nitem2\r\nitem3").unwrap();

    let result = read_input(temp_file).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec!["item1", "item2", "item3"]);

    // Clean up
    std::fs::remove_file(temp_file).unwrap();
}

#[tokio::test]
async fn test_read_input_from_file_with_mixed_line_endings() {
    // Create a temporary file with mixed line endings
    let temp_file = "test_mixed_endings_file.txt";
    std::fs::write(temp_file, "item1\nitem2\r\nitem3").unwrap();

    let result = read_input(temp_file).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec!["item1", "item2", "item3"]);

    // Clean up
    std::fs::remove_file(temp_file).unwrap();
}

#[tokio::test]
async fn test_read_input_from_file_with_large_content() {
    // Create a temporary file with large content
    let temp_file = "test_large_file.txt";
    let content: Vec<String> = (1..=1000).map(|i| format!("item{}", i)).collect();
    std::fs::write(temp_file, content.join("\n")).unwrap();

    let result = read_input(temp_file).await;
    assert!(result.is_ok());
    let read_items = result.unwrap();
    assert_eq!(read_items.len(), 1000);
    assert_eq!(read_items[0], "item1");
    assert_eq!(read_items[999], "item1000");

    // Clean up
    std::fs::remove_file(temp_file).unwrap();
}

#[tokio::test]
async fn test_read_input_from_file_with_very_long_lines() {
    // Create a temporary file with very long lines
    let temp_file = "test_long_lines_file.txt";
    let long_line = "a".repeat(10000);
    std::fs::write(temp_file, format!("{}\n{}", long_line, long_line)).unwrap();

    let result = read_input(temp_file).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec![long_line.clone(), long_line]);

    // Clean up
    std::fs::remove_file(temp_file).unwrap();
}

#[tokio::test]
async fn test_read_input_from_file_with_binary_content() {
    // Create a temporary file with binary content that is invalid UTF-8
    let temp_file = "test_binary_file.txt";
    let binary_content = vec![0xFF, 0xFE, 0x00, 0x01]; // Invalid UTF-8 sequence
    std::fs::write(temp_file, binary_content).unwrap();

    let result = read_input(temp_file).await;
    // This should fail due to invalid UTF-8
    assert!(result.is_err());

    // Clean up
    std::fs::remove_file(temp_file).unwrap();
}

#[tokio::test]
async fn test_read_input_from_file_with_utf8_bom() {
    // Create a temporary file with UTF-8 BOM
    let temp_file = "test_utf8_bom_file.txt";
    let content_with_bom = "\u{FEFF}item1\nitem2\nitem3";
    std::fs::write(temp_file, content_with_bom).unwrap();

    let result = read_input(temp_file).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec!["\u{FEFF}item1", "item2", "item3"]);

    // Clean up
    std::fs::remove_file(temp_file).unwrap();
}

#[tokio::test]
async fn test_read_input_from_file_with_control_characters() {
    // Create a temporary file with control characters
    let temp_file = "test_control_file.txt";
    std::fs::write(temp_file, "item1\nitem2\nitem3").unwrap();

    let result = read_input(temp_file).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec!["item1", "item2", "item3"]);

    // Clean up
    std::fs::remove_file(temp_file).unwrap();
}

#[tokio::test]
async fn test_read_input_from_file_with_emoji() {
    // Create a temporary file with emoji
    let temp_file = "test_emoji_file.txt";
    std::fs::write(temp_file, "item1\nitem2\nitem3").unwrap();

    let result = read_input(temp_file).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec!["item1", "item2", "item3"]);

    // Clean up
    std::fs::remove_file(temp_file).unwrap();
}
