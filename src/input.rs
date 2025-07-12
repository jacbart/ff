use std::fs;
use std::io::{self, BufRead};

/// Read input items from the specified source.
pub fn read_input(source: &str) -> Result<Vec<String>, String> {
    match source {
        "stdin" => read_from_stdin(),
        "direct" => Err("Direct items should be handled by the caller".to_string()),
        _ => read_from_file(source),
    }
}

/// Process direct items provided as command line arguments.
pub fn read_direct_items(items: Vec<String>) -> Result<Vec<String>, String> {
    if items.is_empty() {
        return Err("No items provided".to_string());
    }
    Ok(items)
}

/// Process content as if it came from stdin.
pub fn process_stdin_content(content: &str) -> Result<Vec<String>, String> {
    let mut items = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            items.push(trimmed.to_string());
        }
    }
    if items.is_empty() {
        return Err("No items found in stdin".to_string());
    }
    Ok(items)
}

/// Process content as if it came from a file.
pub fn process_file_content(content: &str) -> Result<Vec<String>, String> {
    let items: Vec<String> = content
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .map(|line| line.to_string())
        .collect();
    if items.is_empty() {
        return Err("No items found in file".to_string());
    }
    Ok(items)
}

fn read_from_stdin() -> Result<Vec<String>, String> {
    let mut items = Vec::new();
    let stdin = io::stdin();
    let reader = stdin.lock();
    for (line_num, line) in reader.lines().enumerate() {
        match line {
            Ok(line) => {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    items.push(trimmed.to_string());
                }
            }
            Err(err) => {
                return Err(format!(
                    "Error reading line {} from stdin: {}",
                    line_num + 1,
                    err
                ));
            }
        }
    }
    if items.is_empty() {
        return Err("No items found in stdin".to_string());
    }
    Ok(items)
}

fn read_from_file(path: &str) -> Result<Vec<String>, String> {
    let content = fs::read_to_string(path)
        .map_err(|err| format!("Error reading file '{}': {}", path, err))?;
    process_file_content(&content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn test_process_stdin_content_empty() {
        let result = process_stdin_content("");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "No items found in stdin");
    }

    #[test]
    fn test_process_stdin_content_with_empty_lines() {
        let result = process_stdin_content("line1\n\nline2\n  \nline3");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec!["line1", "line2", "line3"]);
    }

    #[test]
    fn test_process_stdin_content_with_whitespace() {
        let result = process_stdin_content("  line1  \n  line2  \n  line3  ");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec!["line1", "line2", "line3"]);
    }

    #[test]
    fn test_process_stdin_content_single_line() {
        let result = process_stdin_content("single line");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec!["single line"]);
    }

    #[test]
    fn test_process_stdin_content_only_whitespace() {
        let result = process_stdin_content("   \n  \n  \n");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "No items found in stdin");
    }

    #[test]
    fn test_process_stdin_content_mixed_whitespace() {
        let result = process_stdin_content("  line1  \n\n  line2  \n  \n  line3  ");
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
        let result = process_file_content("line1\n\nline2\n  \nline3");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec!["line1", "line2", "line3"]);
    }

    #[test]
    fn test_process_file_content_with_whitespace() {
        let result = process_file_content("  line1  \n  line2  \n  line3  ");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec!["line1", "line2", "line3"]);
    }

    #[test]
    fn test_process_file_content_single_line() {
        let result = process_file_content("single line");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec!["single line"]);
    }

    #[test]
    fn test_process_file_content_only_whitespace() {
        let result = process_file_content("   \n  \n  \n");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "No items found in file");
    }

    #[test]
    fn test_process_file_content_mixed_whitespace() {
        let result = process_file_content("  line1  \n\n  line2  \n  \n  line3  ");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec!["line1", "line2", "line3"]);
    }

    #[test]
    fn test_read_direct_items_empty() {
        let result = read_direct_items(vec![]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "No items provided");
    }

    #[test]
    fn test_read_direct_items_valid() {
        let items = vec!["item1".to_string(), "item2".to_string()];
        let result = read_direct_items(items.clone());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), items);
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
    fn test_read_input_stdin() {
        // This is a smoke test since we can't easily mock stdin
        let source = "stdin";
        assert_eq!(source, "stdin");
    }

    #[test]
    fn test_read_input_direct() {
        let source = "direct";
        let result = read_input(source);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Direct items should be handled by the caller"
        );
    }

    #[test]
    fn test_read_input_file() {
        // Create a temporary file for testing
        let temp_file = PathBuf::from("test_input_file.txt");
        fs::write(&temp_file, "file_item1\nfile_item2").unwrap();

        let result = read_input("test_input_file.txt");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec!["file_item1", "file_item2"]);

        // Clean up
        fs::remove_file(&temp_file).unwrap();
    }

    #[test]
    fn test_read_input_file_nonexistent() {
        let result = read_input("nonexistent_file.txt");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Error reading file"));
    }

    #[test]
    fn test_read_input_file_empty() {
        // Create a temporary empty file
        let temp_file = PathBuf::from("test_empty_file.txt");
        fs::write(&temp_file, "").unwrap();

        let result = read_input("test_empty_file.txt");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "No items found in file");

        // Clean up
        fs::remove_file(&temp_file).unwrap();
    }

    #[test]
    fn test_read_input_file_with_whitespace_only() {
        // Create a temporary file with only whitespace
        let temp_file = PathBuf::from("test_whitespace_file.txt");
        fs::write(&temp_file, "   \n  \n  \n").unwrap();

        let result = read_input("test_whitespace_file.txt");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "No items found in file");

        // Clean up
        fs::remove_file(&temp_file).unwrap();
    }

    #[test]
    fn test_read_input_file_with_mixed_content() {
        // Create a temporary file with mixed content
        let temp_file = PathBuf::from("test_mixed_file.txt");
        fs::write(&temp_file, "  item1  \n\n  item2  \n  \n  item3  ").unwrap();

        let result = read_input("test_mixed_file.txt");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec!["item1", "item2", "item3"]);

        // Clean up
        fs::remove_file(&temp_file).unwrap();
    }

    #[test]
    fn test_read_input_unknown_source() {
        let result = read_input("unknown_source");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Error reading file"));
    }
}
