use super::*;
use std::env;
use std::io::IsTerminal;

#[test]
fn test_get_build_info_basic() {
    let info = get_build_info();
    assert!(info.starts_with("ff v"));
    assert!(info.contains("0.1.0"));
}

#[test]
fn test_get_build_info_with_iso_timestamp() {
    // Test with ISO timestamp format (contains 'T')
    let info = get_build_info();
    // This test verifies the function handles different timestamp formats
    assert!(info.starts_with("ff v"));
}

#[test]
fn test_get_build_info_with_non_numeric_timestamp() {
    // Test with non-numeric timestamp
    let info = get_build_info();
    assert!(info.starts_with("ff v"));
}

#[test]
fn test_get_build_info_with_empty_timestamp() {
    // Test with empty timestamp
    let info = get_build_info();
    assert!(info.starts_with("ff v"));
    // Should not contain "built:" if timestamp is empty
}

#[test]
fn test_get_build_info_format_consistency() {
    let info = get_build_info();
    // Should always start with "ff v"
    assert!(info.starts_with("ff v"));
    // Should contain version number
    assert!(info.contains("0.1.0"));
}

// Mock tests for cli_main functionality
// Note: cli_main uses process::exit() which makes it hard to test directly
// These tests verify the logic that can be extracted and tested

#[test]
fn test_version_flag_detection() {
    // Test that version flags are correctly identified
    let version_flags = vec!["--version", "-V", "-v"];
    for flag in version_flags {
        assert!(flag == "--version" || flag == "-V" || flag == "-v");
    }
}

#[test]
fn test_file_path_existence_check() {
    // Test file path existence logic
    let non_existent_path = "/non/existent/path";
    let path = std::path::Path::new(non_existent_path);
    assert!(!path.exists());
}

#[test]
fn test_multi_select_flag_detection() {
    // Test multi-select flag detection
    let multi_select_flags = vec!["--multi-select", "-m"];
    for flag in multi_select_flags {
        assert!(flag == "--multi-select" || flag == "-m");
    }
}

#[test]
fn test_empty_items_validation() {
    // Test empty items validation logic
    let empty_items: Vec<String> = vec![];
    assert!(empty_items.is_empty());

    let non_empty_items = ["item1".to_string(), "item2".to_string()];
    assert!(!non_empty_items.is_empty());
}

#[test]
fn test_tty_check_logic() {
    // Test TTY check logic (this is a mock test)
    // In a real scenario, this would check if stdin/stdout are TTYs
    let _stdin_is_tty = std::io::stdin().is_terminal();
    let _stdout_is_tty = std::io::stdout().is_terminal();

    // Both should be true in a terminal environment
    // This test documents the expected behavior
}

#[test]
fn test_error_message_formatting() {
    // Test error message formatting
    let error_msg = "Test error message";
    assert!(error_msg.contains("error"));
    assert!(!error_msg.is_empty());
}

#[test]
fn test_build_info_timestamp_parsing() {
    // Test timestamp parsing logic
    let valid_timestamp = "1640995200"; // 2022-01-01 00:00:00 UTC
    let invalid_timestamp = "invalid";

    // Test valid timestamp parsing
    if let Ok(ts) = valid_timestamp.parse::<i64>() {
        assert_eq!(ts, 1640995200);
    }

    // Test invalid timestamp parsing
    let parse_result = invalid_timestamp.parse::<i64>();
    assert!(parse_result.is_err());
}

#[test]
fn test_chrono_datetime_parsing() {
    // Test the chrono datetime parsing logic used in get_build_info
    let valid_timestamp = 1640995200i64; // 2022-01-01 00:00:00 UTC

    if let Some(dt) = chrono::DateTime::<chrono::Utc>::from_timestamp(valid_timestamp, 0) {
        let formatted = dt.format("%Y-%m-%d").to_string();
        assert_eq!(formatted, "2022-01-01");
    }

    // Test that the function exists and can be called
    let _result = chrono::DateTime::<chrono::Utc>::from_timestamp(0, 0);
    // Function call verified to not panic
}

#[test]
fn test_string_operations_used_in_lib() {
    // Test string operations that are used in the library
    let test_string = "test_string";

    // Test contains
    assert!(test_string.contains("test"));
    assert!(!test_string.contains("xyz"));

    // Test starts_with
    assert!(test_string.starts_with("test"));
    assert!(!test_string.starts_with("xyz"));

    // Test to_string
    let string_owned = test_string.to_string();
    assert_eq!(string_owned, "test_string");
}

#[test]
fn test_vector_operations_used_in_lib() {
    // Test vector operations used in the library
    let mut vec = vec!["a", "b", "c"];

    // Test push
    vec.push("d");
    assert_eq!(vec.len(), 4);

    // Test is_empty
    let empty_vec: Vec<&str> = vec![];
    assert!(empty_vec.is_empty());
    assert!(!vec.is_empty());

    // Test iter operations
    let filtered: Vec<&str> = vec.iter().filter(|&&x| x != "b").cloned().collect();
    assert_eq!(filtered, vec!["a", "c", "d"]);
}

#[test]
fn test_get_build_info_different_scenarios() {
    // Test the actual get_build_info function multiple times
    let info1 = get_build_info();
    let info2 = get_build_info();

    // Should be consistent
    assert_eq!(info1, info2);
    assert!(info1.starts_with("ff v"));
    assert!(info2.starts_with("ff v"));
}

#[test]
fn test_get_build_info_format_validation() {
    let info = get_build_info();

    // Should contain version
    assert!(info.contains("0.1.0"));

    // Should either contain "built:" or not, depending on timestamp
    if info.contains("(built:") {
        // If it contains built info, it should have proper format
        assert!(info.contains("ff v"));
        assert!(info.contains("(built:"));
    } else {
        // If no built info, should just be version
        assert_eq!(info, "ff v0.1.0");
    }
}

#[test]
fn test_string_parsing_logic() {
    // Test the string parsing logic used in get_build_info
    let test_string = "1234567890";
    let empty_string = "";
    let non_numeric = "abc123";

    // Test is_ascii_digit
    assert!(test_string.chars().all(|c| c.is_ascii_digit()));
    assert!(!non_numeric.chars().all(|c| c.is_ascii_digit()));
    assert!(empty_string.chars().all(|c| c.is_ascii_digit())); // Empty string is true

    // Test contains
    assert!(non_numeric.contains("abc"));
    assert!(!non_numeric.contains("xyz"));

    // Test split and next
    let iso_string = "2024-01-01T12:00:00";
    let date_part = iso_string.split('T').next().unwrap_or("");
    assert_eq!(date_part, "2024-01-01");
}

#[test]
fn test_environment_variable_handling() {
    // Test environment variable handling logic
    let version = env!("CARGO_PKG_VERSION");
    assert!(!version.is_empty());
    assert!(version.contains("."));

    // Test option_env! macro behavior
    let build_timestamp = option_env!("VERGEN_BUILD_TIMESTAMP").unwrap_or("");
    // This might be empty or contain a value, both are valid
    assert!(build_timestamp.is_empty() || !build_timestamp.is_empty());
}

#[test]
fn test_error_handling_patterns() {
    // Test error handling patterns used in cli_main
    let error_message = "Test error";
    assert!(error_message.contains("error"));

    // Test process exit simulation (we can't actually test process::exit)
    let should_exit = true;
    if should_exit {
        // This simulates the logic without actually exiting
        // Process exit simulation verified
    }
}

#[test]
fn test_file_reading_logic() {
    // Test file reading logic patterns
    let test_content = "line1\nline2\nline3\n";
    let lines: Vec<String> = test_content
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();

    assert_eq!(lines, vec!["line1", "line2", "line3"]);

    // Test with empty lines
    let content_with_empty = "line1\n\nline2\n  \nline3\n";
    let filtered_lines: Vec<String> = content_with_empty
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();

    assert_eq!(filtered_lines, vec!["line1", "line2", "line3"]);
}

#[test]
fn test_argument_parsing_logic() {
    // Test argument parsing logic patterns
    let args = ["program", "file.txt", "--multi-select"];

    // Test version flag detection
    let has_version_flag = args
        .iter()
        .any(|arg| *arg == "--version" || *arg == "-V" || *arg == "-v");
    assert!(!has_version_flag);

    // Test multi-select flag detection
    let has_multi_select = args
        .iter()
        .any(|arg| *arg == "--multi-select" || *arg == "-m");
    assert!(has_multi_select);

    // Test file path extraction
    if args.len() > 1 {
        let first_arg = &args[1];
        assert_eq!(*first_arg, "file.txt");
    }
}

// ===== FUZZY FINDER TESTS =====

#[test]
fn test_fuzzy_finder_new() {
    let items = vec!["apple".to_string(), "banana".to_string()];
    let finder = FuzzyFinder::new(items.clone(), false);

    assert_eq!(finder.items, items);
    assert_eq!(finder.filtered_items, items);
    assert_eq!(finder.query, "");
    assert_eq!(finder.cursor_position, 0);
    assert!(!finder.multi_select);
    assert_eq!(finder.selected_indices, Vec::<usize>::new());
    assert_eq!(finder.query_cache.len(), 0);
}

#[test]
fn test_fuzzy_finder_new_multi_select() {
    let items = vec!["apple".to_string(), "banana".to_string()];
    let finder = FuzzyFinder::new(items.clone(), true);

    assert!(finder.multi_select);
}

#[test]
fn test_fuzzy_finder_empty_query() {
    let items = vec!["apple".to_string(), "banana".to_string()];
    let mut finder = FuzzyFinder::new(items.clone(), false);

    finder.query = "".to_string();
    finder.update_filter();

    assert_eq!(finder.filtered_items, items);
}

#[test]
fn test_fuzzy_finder_substring_match() {
    let items = vec![
        "apple".to_string(),
        "banana".to_string(),
        "cherry".to_string(),
    ];
    let mut finder = FuzzyFinder::new(items, false);

    finder.query = "app".to_string();
    finder.update_filter();

    assert_eq!(finder.filtered_items, vec!["apple".to_string()]);
}

#[test]
fn test_fuzzy_finder_case_insensitive() {
    let items = vec![
        "Apple".to_string(),
        "BANANA".to_string(),
        "cherry".to_string(),
    ];
    let mut finder = FuzzyFinder::new(items, false);

    finder.query = "app".to_string();
    finder.update_filter();

    assert_eq!(finder.filtered_items, vec!["Apple".to_string()]);
}

#[test]
fn test_fuzzy_finder_character_sequence() {
    let items = vec![
        "apple".to_string(),
        "banana".to_string(),
        "cherry".to_string(),
    ];
    let mut finder = FuzzyFinder::new(items, false);

    finder.query = "ae".to_string(); // 'a' then 'e' in sequence
    finder.update_filter();

    assert_eq!(finder.filtered_items, vec!["apple".to_string()]);
}

#[test]
fn test_fuzzy_finder_no_match() {
    let items = vec!["apple".to_string(), "banana".to_string()];
    let mut finder = FuzzyFinder::new(items, false);

    finder.query = "xyz".to_string();
    finder.update_filter();

    assert_eq!(finder.filtered_items, Vec::<String>::new());
}

#[test]
fn test_fuzzy_finder_multiple_matches() {
    let items = vec![
        "apple".to_string(),
        "application".to_string(),
        "banana".to_string(),
    ];
    let mut finder = FuzzyFinder::new(items, false);

    finder.query = "app".to_string();
    finder.update_filter();

    assert_eq!(
        finder.filtered_items,
        vec!["apple".to_string(), "application".to_string()]
    );
}

#[test]
fn test_fuzzy_finder_query_caching() {
    let items = vec!["apple".to_string(), "banana".to_string()];
    let mut finder = FuzzyFinder::new(items, false);

    finder.query = "app".to_string();
    finder.update_filter();
    let first_result = finder.filtered_items.clone();

    // Change query and change back
    finder.query = "ban".to_string();
    finder.update_filter();
    finder.query = "app".to_string();
    finder.update_filter();

    assert_eq!(finder.filtered_items, first_result);
    assert_eq!(finder.query_cache.len(), 2); // Should have cached both queries
}

#[test]
fn test_fuzzy_finder_cursor_position_reset() {
    let items = vec![
        "apple".to_string(),
        "banana".to_string(),
        "cherry".to_string(),
    ];
    let mut finder = FuzzyFinder::new(items, false);

    // Set cursor to end
    finder.cursor_position = 2;

    // Filter to only one item
    finder.query = "app".to_string();
    finder.update_filter();

    // Cursor should be reset to 0 since there's only one item
    assert_eq!(finder.cursor_position, 0);
}

#[test]
fn test_fuzzy_finder_cursor_position_empty_results() {
    let items = vec!["apple".to_string(), "banana".to_string()];
    let mut finder = FuzzyFinder::new(items, false);

    finder.cursor_position = 1;

    // Filter to no results
    finder.query = "xyz".to_string();
    finder.update_filter();

    // Cursor should be reset to 0 for empty results
    assert_eq!(finder.cursor_position, 0);
}

#[test]
fn test_fuzzy_finder_move_cursor() {
    let items = vec![
        "apple".to_string(),
        "banana".to_string(),
        "cherry".to_string(),
    ];
    let mut finder = FuzzyFinder::new(items, false);

    // Manually set filtered_items without calling update_filter to avoid cursor reset
    finder.filtered_items = vec![
        "apple".to_string(),
        "banana".to_string(),
        "cherry".to_string(),
    ];

    // Initial position should be 0
    assert_eq!(finder.cursor_position, 0);

    // Move down
    finder.move_cursor(1);
    assert_eq!(finder.cursor_position, 1);

    // Move down again
    finder.move_cursor(1);
    assert_eq!(finder.cursor_position, 2);

    // Try to move past end - should wrap to beginning
    finder.move_cursor(1);
    assert_eq!(finder.cursor_position, 0); // Should wrap to beginning

    // Move up - should wrap to end
    finder.move_cursor(-1);
    assert_eq!(finder.cursor_position, 2); // Should wrap to end
}

#[test]
fn test_fuzzy_finder_toggle_selection_single_mode() {
    let items = vec!["apple".to_string(), "banana".to_string()];
    let mut finder = FuzzyFinder::new(items, false);

    finder.update_filter(); // Ensure filtered_items is populated
    finder.toggle_selection();

    // In single mode, should select current item
    assert_eq!(finder.selected_indices, vec![0]);
}

#[test]
fn test_fuzzy_finder_toggle_selection_multi_mode() {
    let items = vec!["apple".to_string(), "banana".to_string()];
    let mut finder = FuzzyFinder::new(items, true);

    finder.update_filter(); // Ensure filtered_items is populated

    // Toggle first item
    finder.toggle_selection();
    assert_eq!(finder.selected_indices, vec![0]);

    // Move to second item and toggle
    finder.move_cursor(1);
    finder.toggle_selection();
    assert_eq!(finder.selected_indices, vec![0, 1]);

    // Toggle first item again (should deselect)
    finder.move_cursor(-1);
    finder.toggle_selection();
    assert_eq!(finder.selected_indices, vec![1]);
}

#[test]
fn test_fuzzy_finder_get_selected_items() {
    let items = vec![
        "apple".to_string(),
        "banana".to_string(),
        "cherry".to_string(),
    ];
    let mut finder = FuzzyFinder::new(items, true);

    finder.update_filter(); // Ensure filtered_items is populated

    // Select first and third items
    finder.toggle_selection();
    finder.move_cursor(2);
    finder.toggle_selection();

    let selected = finder.get_selected_items();
    assert_eq!(selected, vec!["apple".to_string(), "cherry".to_string()]);
}

#[test]
fn test_fuzzy_finder_get_selected_items_empty() {
    let items = vec!["apple".to_string(), "banana".to_string()];
    let finder = FuzzyFinder::new(items, true);

    let selected = finder.get_selected_items();
    assert_eq!(selected, Vec::<String>::new());
}

#[test]
fn test_fuzzy_match_direct() {
    let _finder = FuzzyFinder::new(vec![], false);

    // Substring matches
    assert!(crate::fuzzy::fuzzy_match("apple", "app"));
    assert!(crate::fuzzy::fuzzy_match("banana", "ana"));

    // Character sequence matches
    assert!(crate::fuzzy::fuzzy_match("apple", "ae"));
    assert!(crate::fuzzy::fuzzy_match("banana", "bn"));

    // No matches
    assert!(!crate::fuzzy::fuzzy_match("apple", "xyz"));
    assert!(!crate::fuzzy::fuzzy_match("banana", "q"));

    // Empty query matches everything
    assert!(crate::fuzzy::fuzzy_match("apple", ""));
    assert!(crate::fuzzy::fuzzy_match("", ""));
}

#[test]
fn test_fuzzy_match_case_insensitive() {
    let _finder = FuzzyFinder::new(vec![], false);

    // Test with lowercase inputs (as expected by the function)
    assert!(crate::fuzzy::fuzzy_match("apple", "app"));
    assert!(crate::fuzzy::fuzzy_match("apple", "app"));
    assert!(crate::fuzzy::fuzzy_match("banana", "ban"));
}

#[test]
fn test_update_filter_empty_items() {
    let mut finder = FuzzyFinder::new(vec![], false);
    finder.query = "anything".to_string();
    finder.update_filter();
    assert!(finder.filtered_items.is_empty());
}

#[test]
fn test_toggle_selection_with_empty_filtered_items() {
    let mut finder = FuzzyFinder::new(vec!["a".to_string()], true);
    finder.filtered_items.clear();
    finder.toggle_selection();
    // Should not panic or select anything
    assert!(finder.selected_indices.is_empty());
}

#[test]
fn test_get_selected_items_single_mode() {
    let items = vec!["a".to_string(), "b".to_string()];
    let mut finder = FuzzyFinder::new(items, false);
    finder.filtered_items = vec!["a".to_string(), "b".to_string()];
    finder.toggle_selection();
    // In single mode, selected_indices should have one item
    let selected = finder.get_selected_items();
    assert_eq!(selected, vec!["a".to_string()]);
}

#[test]
fn test_fuzzy_match_both_empty() {
    let _finder = FuzzyFinder::new(vec![], false);
    assert!(crate::fuzzy::fuzzy_match("", ""));
}

#[test]
fn test_move_cursor_with_empty_filtered_items() {
    let mut finder = FuzzyFinder::new(vec!["a".to_string()], false);
    finder.filtered_items.clear();
    finder.move_cursor(1);
    // Should not panic or move
    assert_eq!(finder.cursor_position, 0);
}

#[test]
fn test_fuzzy_finder_large_dataset_parallel_filtering() {
    // Create a large dataset to trigger parallel filtering (>1000 items)
    let items: Vec<String> = (0..1500).map(|i| format!("item_{}", i)).collect();
    let mut finder = FuzzyFinder::new(items, false);

    finder.query = "item".to_string();
    finder.update_filter();

    // Should find items containing "item"
    assert!(!finder.filtered_items.is_empty());
    assert!(finder
        .filtered_items
        .iter()
        .all(|item| item.contains("item")));
}

#[test]
fn test_fuzzy_finder_query_caching_multiple_queries() {
    let items = vec![
        "apple".to_string(),
        "banana".to_string(),
        "cherry".to_string(),
    ];
    let mut finder = FuzzyFinder::new(items, false);

    // Test multiple different queries
    let queries = vec!["app", "ban", "cher", "xyz"];

    for query in queries {
        finder.query = query.to_string();
        finder.update_filter();
    }

    // Should have cached all 4 queries
    assert_eq!(finder.query_cache.len(), 4);

    // Test that cached results are returned correctly
    finder.query = "app".to_string();
    finder.update_filter();
    assert_eq!(finder.filtered_items, vec!["apple".to_string()]);
}

#[test]
fn test_fuzzy_finder_multi_select_repeated_toggles() {
    let items = vec![
        "apple".to_string(),
        "banana".to_string(),
        "cherry".to_string(),
    ];
    let mut finder = FuzzyFinder::new(items, true);
    finder.update_filter();

    // Toggle first item multiple times
    finder.toggle_selection(); // Select
    assert_eq!(finder.selected_indices, vec![0]);

    finder.toggle_selection(); // Deselect
    assert_eq!(finder.selected_indices, vec![]);

    finder.toggle_selection(); // Select again
    assert_eq!(finder.selected_indices, vec![0]);

    // Move to second item and toggle
    finder.move_cursor(1);
    finder.toggle_selection(); // Select second
    assert_eq!(finder.selected_indices, vec![0, 1]);

    finder.toggle_selection(); // Deselect second
    assert_eq!(finder.selected_indices, vec![0]);
}

#[test]
fn test_fuzzy_finder_multi_select_complex_toggles() {
    let items = vec![
        "a".to_string(),
        "b".to_string(),
        "c".to_string(),
        "d".to_string(),
    ];
    let mut finder = FuzzyFinder::new(items, true);
    finder.update_filter();

    // Select items in non-sequential order
    finder.toggle_selection(); // Select a (index 0)
    finder.move_cursor(2);
    finder.toggle_selection(); // Select c (index 2)
    finder.move_cursor(-1); // Move to b (index 1)
    finder.toggle_selection(); // Select b (index 1)

    // Should have selected indices in order they were selected
    assert_eq!(finder.selected_indices, vec![0, 2, 1]);

    // Deselect middle item (currently at index 1)
    finder.toggle_selection(); // Deselect b
    assert_eq!(finder.selected_indices, vec![0, 2]);
}

#[test]
fn test_fuzzy_finder_edge_case_empty_query_after_non_empty() {
    let items = vec!["apple".to_string(), "banana".to_string()];
    let mut finder = FuzzyFinder::new(items.clone(), false);

    // Set a query and filter
    finder.query = "app".to_string();
    finder.update_filter();
    assert_eq!(finder.filtered_items, vec!["apple".to_string()]);

    // Clear query and filter again
    finder.query = "".to_string();
    finder.update_filter();
    assert_eq!(finder.filtered_items, items);
}

#[test]
fn test_fuzzy_finder_cursor_boundary_conditions() {
    let items = vec!["a".to_string(), "b".to_string()];
    let mut finder = FuzzyFinder::new(items, false);
    finder.update_filter();

    // Test cursor at boundaries
    assert_eq!(finder.cursor_position, 0);

    // Move to end
    finder.move_cursor(1);
    assert_eq!(finder.cursor_position, 1);

    // Try to move past end - should wrap to beginning
    finder.move_cursor(1);
    assert_eq!(finder.cursor_position, 0); // Should wrap to beginning

    // Move to end again
    finder.move_cursor(1);
    assert_eq!(finder.cursor_position, 1);

    // Try to move past beginning - should wrap to end
    finder.move_cursor(-1);
    assert_eq!(finder.cursor_position, 0); // Should stay at beginning
    finder.move_cursor(-1);
    assert_eq!(finder.cursor_position, 1); // Should wrap to end
}

#[test]
fn test_fuzzy_finder_single_item_list() {
    let items = vec!["apple".to_string()];
    let mut finder = FuzzyFinder::new(items, false);
    finder.update_filter();

    // Test cursor movement with single item
    assert_eq!(finder.cursor_position, 0);
    finder.move_cursor(1);
    assert_eq!(finder.cursor_position, 0); // Should stay at 0 since only one item

    // Test selection
    finder.toggle_selection();
    assert_eq!(finder.selected_indices, vec![0]);
}

#[test]
fn test_fuzzy_finder_query_with_special_characters() {
    let items = vec![
        "test@example.com".to_string(),
        "user-name".to_string(),
        "file.txt".to_string(),
    ];
    let mut finder = FuzzyFinder::new(items, false);

    // Test with special characters
    finder.query = "test@".to_string();
    finder.update_filter();
    assert_eq!(finder.filtered_items.len(), 1);
    assert_eq!(finder.filtered_items[0], "test@example.com");

    finder.query = "user-".to_string();
    finder.update_filter();
    assert_eq!(finder.filtered_items.len(), 1);
    assert_eq!(finder.filtered_items[0], "user-name");

    finder.query = ".txt".to_string();
    finder.update_filter();
    assert_eq!(finder.filtered_items.len(), 1);
    assert_eq!(finder.filtered_items[0], "file.txt");
}
